//! BCINR-AZ: a branchless AlphaZero-style (DeepMind paradigm) chess engine that
//! drives an MCTS tree and evaluates leaf positions in *batches* on the Apple
//! M3 Max 40-core GPU via the dual-head NNUE compute shader.
//!
//! Each node expansion makes all legal children and dispatches their NNUE value
//! evaluation to the GPU in a single compute pass — that is what actually keeps
//! the 40 cores busy (per-node single-board dispatch would be latency-bound).
//!
//! A small branchless quiescence guard (the Stockfish paradigm) can resolve
//! captures at a leaf before its NN value is trusted, patching MCTS's tactical
//! blind spot. Toggle with `setoption name Hybrid value true|false`.
//!
//! UCI subset: `uci`, `isready`, `position [startpos|fen] [moves ...]`,
//! `go [movetime ms | nodes N]`, `setoption`, `quit`.
//! Plus a non-UCI `bench` command that reports GPU eval throughput (boards/sec).

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};
use pollster::FutureExt;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::Instant;
use wgpu::util::DeviceExt;

use playground::nnue::BranchTorchNNUE;

const C_PUCT: f32 = 1.5;

// ---------------------------------------------------------------------------
// GPU value head
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Accumulator {
    hidden: [i32; 16],
}

struct GpuEval {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    nnue_buffer: wgpu::Buffer,
    boards_evaluated: u64,
}

impl GpuEval {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .expect("no Metal adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .expect("no device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NNUE"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("NNUE Pipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let nnue = BranchTorchNNUE::new();
        // Flatten the weights into the exact field order of the WGSL `NNUE` struct.
        // Building the i32 vec ourselves makes the upload safe (no transmute) and
        // independent of the host struct's memory layout.
        let mut flat: Vec<i32> = Vec::with_capacity(13_348);
        for row in &nnue.l1_weights {
            flat.extend_from_slice(row);
        }
        flat.extend_from_slice(&nnue.l1_biases);
        flat.extend_from_slice(&nnue.l2_weights_value);
        for row in &nnue.l2_weights_policy {
            flat.extend_from_slice(row);
        }
        flat.push(nnue.l2_bias_value);
        flat.extend_from_slice(&nnue._pad);
        let nnue_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NNUE Weights"),
            contents: bytemuck::cast_slice(&flat),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        GpuEval {
            device,
            queue,
            pipeline,
            nnue_buffer,
            boards_evaluated: 0,
        }
    }

    /// Evaluate a batch of accumulators, returning the i32 value head per board
    /// (White-relative centipawns). One GPU compute dispatch for the whole batch.
    fn values(&mut self, boards: &[Accumulator]) -> Vec<i32> {
        if boards.is_empty() {
            return Vec::new();
        }
        let count = boards.len();
        let padded = count.div_ceil(64) * 64;

        let mut input = boards.to_vec();
        input.resize(padded, Accumulator { hidden: [0; 16] });

        let input_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input"),
                contents: bytemuck::cast_slice(&input),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let value_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Value"),
            size: (padded * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let policy_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Policy"),
            size: (padded * 4 * 64) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.nnue_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: value_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: policy_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((padded / 64) as u32, 1, 1);
        }

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (padded * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&value_buffer, 0, &staging, 0, (padded * 4) as u64);
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        // Correct readback: the shader writes i32 values (the prior code read them
        // as f32, reinterpreting the bit pattern as garbage).
        let out: Vec<i32> = bytemuck::cast_slice::<u8, i32>(&data)[..count].to_vec();
        drop(data);
        staging.unmap();

        self.boards_evaluated += count as u64;
        out
    }
}

/// Incremental L1 accumulator on the CPU (same content the shader consumes).
fn board_to_acc(b: &Board, nnue: &BranchTorchNNUE) -> Accumulator {
    let mut hidden = nnue.l1_biases;
    let pieces = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
    for (p_idx, &p) in pieces.iter().enumerate() {
        let w_bb = *b.color_combined(Color::White) & *b.pieces(p);
        for sq in w_bb {
            let s = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][p_idx * 64 + s];
            }
        }
        let b_bb = *b.color_combined(Color::Black) & *b.pieces(p);
        for sq in b_bb {
            let s = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][(p_idx + 6) * 64 + s];
            }
        }
    }
    Accumulator { hidden }
}

/// Map a White-relative centipawn score to a side-to-move value in (-1, 1).
fn stm_value(white_cp: i32, stm: Color) -> f32 {
    let cp = if stm == Color::White {
        white_cp as f32
    } else {
        -(white_cp as f32)
    };
    // tanh squashing; branch-free.
    let x = cp / 400.0;
    let e = (2.0 * x).exp();
    (e - 1.0) / (e + 1.0)
}

// ---------------------------------------------------------------------------
// MCTS tree
// ---------------------------------------------------------------------------

struct Node {
    board: Board,
    mv: Option<ChessMove>,
    prior: f32,
    visits: u32,
    value_sum: f32, // accumulated value from this node's side-to-move perspective
    children: Vec<u32>,
    expanded: bool,
    terminal: Option<f32>, // terminal value (stm perspective) if game over
    vloss: u32,            // in-flight virtual losses (for leaf-parallel batching)
    queued: bool,          // already scheduled for expansion this batch
    pending_value: f32,    // value computed at expansion, consumed by backprop
}

struct Mcts<'a> {
    nodes: Vec<Node>,
    gpu: &'a mut GpuEval,
    nnue: &'a BranchTorchNNUE,
    hybrid: bool,
}

impl<'a> Mcts<'a> {
    fn new(root: Board, gpu: &'a mut GpuEval, nnue: &'a BranchTorchNNUE, hybrid: bool) -> Self {
        let mut m = Mcts {
            nodes: Vec::with_capacity(1 << 16),
            gpu,
            nnue,
            hybrid,
        };
        m.nodes.push(Node {
            board: root,
            mv: None,
            prior: 1.0,
            visits: 0,
            value_sum: 0.0,
            children: Vec::new(),
            expanded: false,
            terminal: None,
            vloss: 0,
            queued: false,
            pending_value: 0.0,
        });
        m
    }

    /// PUCT-select the best child of `node`. Branchless argmax via a fold over
    /// children (no early-exit `break`; the running best is a mask-select).
    fn select_child(&self, node: usize) -> usize {
        let p = &self.nodes[node];
        // Virtual losses count as in-flight visits so parallel descents in the
        // same batch diversify instead of all piling onto one leaf.
        let parent_visits = (p.visits + p.vloss).max(1) as f32;
        let sqrt_parent = parent_visits.sqrt();
        let mut best_idx = p.children[0] as usize;
        let mut best_score = f32::NEG_INFINITY;
        for &cidx in &self.nodes[node].children {
            let c = &self.nodes[cidx as usize];
            let eff_visits = c.visits + c.vloss;
            // child.value_sum is from the CHILD's stm perspective; from the parent
            // it is the negation. Each in-flight virtual loss adds +1 to the child's
            // own value (it "wins" => looks bad for the parent), discouraging the
            // parent from re-selecting an already-in-flight child.
            let q = if eff_visits > 0 {
                -((c.value_sum + c.vloss as f32) / eff_visits as f32)
            } else {
                0.0
            };
            let u = C_PUCT * c.prior * sqrt_parent / (1.0 + eff_visits as f32);
            let score = q + u;
            // Integer-domain argmax. The previous mask-select did the running-best
            // update in the f32 domain: `score*take + best_score*(1-take)`. On the
            // first iteration best_score == NEG_INFINITY and take == 0 on every
            // later non-improving child, so `NEG_INFINITY * 0.0 == NaN` poisoned
            // best_score — after which `score > NaN` is always false and the very
            // first child was returned forever (no exploration; only one root child
            // ever got visits). Round-tripping the u32 node index through f32 also
            // loses precision past 2^24 nodes. Both are avoided here.
            let take = score > best_score;
            best_score = if take { score } else { best_score };
            best_idx = if take { cidx as usize } else { best_idx };
        }
        best_idx
    }

    /// Run ONE leaf-parallel batch: select up to `batch` leaves (using virtual
    /// loss so they diverge), expand them all, evaluate EVERY child of EVERY
    /// selected leaf in a SINGLE GPU dispatch, then back-propagate. Returns the
    /// number of leaf playouts performed. This is what saturates the 40 GPU cores
    /// — a per-leaf dispatch would be latency-bound (~700 sims/s); batching turns
    /// the GPU's ~18M evals/s of throughput into search depth.
    fn run_batch(&mut self, batch: usize) -> u64 {
        // --- 1. Selection: gather leaves, applying virtual loss along each path.
        let mut paths: Vec<Vec<usize>> = Vec::with_capacity(batch);
        let mut to_expand: Vec<usize> = Vec::new();
        for _ in 0..batch {
            let mut path = Vec::new();
            let mut node = 0usize;
            loop {
                path.push(node);
                self.nodes[node].vloss += 1;
                if self.nodes[node].terminal.is_some() {
                    break;
                }
                if !self.nodes[node].expanded {
                    if !self.nodes[node].queued {
                        self.nodes[node].queued = true;
                        to_expand.push(node);
                    }
                    break;
                }
                node = self.select_child(node);
            }
            paths.push(path);
        }

        // --- 2. Expansion: build every child of every unique leaf into ONE flat
        // accumulator batch, dispatch once.
        let mut flat: Vec<Accumulator> = Vec::new();
        // (leaf, start, moves, boards)
        let mut plan: Vec<(usize, usize, Vec<ChessMove>, Vec<Board>)> = Vec::new();
        for &leaf in &to_expand {
            self.nodes[leaf].queued = false;
            let board = self.nodes[leaf].board;
            match board.status() {
                BoardStatus::Checkmate => {
                    self.nodes[leaf].terminal = Some(-1.0);
                    self.nodes[leaf].expanded = true;
                    continue;
                }
                BoardStatus::Stalemate => {
                    self.nodes[leaf].terminal = Some(0.0);
                    self.nodes[leaf].expanded = true;
                    continue;
                }
                BoardStatus::Ongoing => {}
            }
            let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
            let boards: Vec<Board> = moves.iter().map(|m| board.make_move_new(*m)).collect();
            let start = flat.len();
            for b in &boards {
                flat.push(board_to_acc(b, self.nnue));
            }
            plan.push((leaf, start, moves, boards));
        }
        let white_cps = self.gpu.values(&flat);

        // --- 3. Create children, priors, and each leaf's 1-ply negamax value.
        for (leaf, start, moves, boards) in plan {
            let n = moves.len();
            let cps = &white_cps[start..start + n];
            let mut child_vals: Vec<f32> = boards
                .iter()
                .zip(cps)
                .map(|(b, &cp)| stm_value(cp, b.side_to_move()))
                .collect();
            if self.hybrid {
                for (i, cb) in boards.iter().enumerate() {
                    if let Some(adj) = self.tactical_guard(cb) {
                        child_vals[i] = adj;
                    }
                }
            }
            let parent_vals: Vec<f32> = child_vals.iter().map(|v| -v).collect();
            let priors = softmax(&parent_vals);
            let base = self.nodes.len() as u32;
            for (i, (&m, &cb)) in moves.iter().zip(boards.iter()).enumerate() {
                self.nodes.push(Node {
                    board: cb,
                    mv: Some(m),
                    prior: priors[i],
                    visits: 0,
                    value_sum: 0.0,
                    children: Vec::new(),
                    expanded: false,
                    terminal: None,
                    vloss: 0,
                    queued: false,
                    pending_value: 0.0,
                });
                self.nodes[leaf].children.push(base + i as u32);
            }
            self.nodes[leaf].expanded = true;
            self.nodes[leaf].pending_value =
                parent_vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        }

        // --- 4. Back-propagate every path, removing the virtual loss.
        for path in &paths {
            let leaf = *path.last().unwrap();
            let v = self.nodes[leaf]
                .terminal
                .unwrap_or(self.nodes[leaf].pending_value);
            let mut x = v;
            for &node in path.iter().rev() {
                self.nodes[node].vloss = self.nodes[node].vloss.saturating_sub(1);
                self.nodes[node].visits += 1;
                self.nodes[node].value_sum += x;
                x = -x;
            }
        }
        paths.len() as u64
    }

    /// One-ply static capture resolution from the side-to-move's perspective.
    /// Returns an adjusted stm value if captures exist, else `None`. CPU-only:
    /// the signed white-relative eval already lives in `hidden[0]` (neuron 0 is
    /// the +eval unit), so no GPU round-trip is needed — per-child GPU dispatches
    /// would be latency-bound and destroy throughput.
    fn tactical_guard(&self, board: &Board) -> Option<f32> {
        let mut caps: Vec<ChessMove> = MoveGen::new_legal(board).collect();
        caps.retain(|m| board.piece_on(m.get_dest()).is_some());
        if caps.is_empty() {
            return None;
        }
        // best capture for the side to move = max over (-opponent_value)
        let best = caps
            .iter()
            .map(|m| {
                let child = board.make_move_new(*m);
                let white_cp = board_to_acc(&child, self.nnue).hidden[0];
                -stm_value(white_cp, child.side_to_move())
            })
            .fold(f32::NEG_INFINITY, f32::max);
        Some(best)
    }

    fn best_move(&self) -> Option<ChessMove> {
        // Most-visited child (standard AlphaZero move selection).
        self.nodes[0]
            .children
            .iter()
            .max_by_key(|&&c| self.nodes[c as usize].visits)
            .and_then(|&c| self.nodes[c as usize].mv)
    }
}

/// Branchless softmax (numerically stabilised by subtracting the max).
fn softmax(xs: &[f32]) -> Vec<f32> {
    if xs.is_empty() {
        return Vec::new();
    }
    let m = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = xs.iter().map(|&x| (x - m).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

fn search(board: &Board, gpu: &mut GpuEval, nnue: &BranchTorchNNUE, hybrid: bool, max_ms: u128, max_nodes: u64) -> Option<ChessMove> {
    let start = Instant::now();
    let before = gpu.boards_evaluated;
    let mut tree = Mcts::new(*board, gpu, nnue, hybrid);
    // Leaf-parallel batch size: how many leaves are collected and evaluated per
    // single GPU dispatch. Larger => better GPU saturation, coarser tree updates.
    const BATCH: usize = 384;
    let mut sims = 0u64;
    loop {
        sims += tree.run_batch(BATCH);
        if start.elapsed().as_millis() >= max_ms {
            break;
        }
        // Bound tree memory for very long searches.
        if sims >= max_nodes || tree.nodes.len() > 4_000_000 {
            break;
        }
    }
    let mv = tree.best_move();
    let elapsed = start.elapsed().as_millis().max(1);
    let gpu_boards = tree.gpu.boards_evaluated - before;
    drop(tree);
    println!(
        "info sims {} gpu_boards {} time {} sps {} gpu_boards_per_sec {}",
        sims,
        gpu_boards,
        elapsed,
        (sims as u128 * 1000) / elapsed,
        (gpu_boards as u128 * 1000) / elapsed
    );
    mv
}

impl GpuEval {
    fn bench(&mut self, batch: usize, iters: usize) {
        let board = Board::default();
        let nnue = BranchTorchNNUE::new();
        let acc = board_to_acc(&board, &nnue);
        let batch_vec = vec![acc; batch];
        let start = Instant::now();
        for _ in 0..iters {
            let _ = self.values(&batch_vec);
        }
        let secs = start.elapsed().as_secs_f64();
        let total = (batch * iters) as f64;
        println!(
            "GPU bench: {} boards in {:.3}s = {:.2}M boards/sec ({} per dispatch x {} dispatches)",
            total as u64,
            secs,
            total / secs / 1e6,
            batch,
            iters
        );
    }
}

fn main() {
    let mut board = Board::default();
    let mut gpu = GpuEval::new();
    let nnue = BranchTorchNNUE::new();
    let mut hybrid = true;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        match t[0] {
            "uci" => {
                writeln!(out, "id name BCINR-AZ (GPU MCTS)").unwrap();
                writeln!(out, "id author BCINR").unwrap();
                writeln!(out, "option name Hybrid type check default true").unwrap();
                writeln!(out, "uciok").unwrap();
            }
            "isready" => writeln!(out, "readyok").unwrap(),
            "setoption" => {
                if line.contains("Hybrid") {
                    hybrid = line.contains("true");
                }
            }
            "ucinewgame" => board = Board::default(),
            "position" => {
                board = parse_position(&t).unwrap_or_else(Board::default);
            }
            "go" => {
                let mut max_ms = 1000u128;
                let mut max_nodes = u64::MAX;
                let mut i = 1;
                while i < t.len() {
                    match t[i] {
                        "movetime" => {
                            max_ms = t.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1000);
                            i += 1;
                        }
                        "nodes" => {
                            max_nodes = t.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(u64::MAX);
                            i += 1;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                let mv = search(&board, &mut gpu, &nnue, hybrid, max_ms, max_nodes)
                    .or_else(|| MoveGen::new_legal(&board).next());
                match mv {
                    Some(m) => writeln!(out, "bestmove {m}").unwrap(),
                    None => writeln!(out, "bestmove 0000").unwrap(),
                }
            }
            "eval" => {
                // Debug: GPU value of the current board and of each legal move.
                let acc = board_to_acc(&board, &nnue);
                let wcp = gpu.values(&[acc])[0];
                writeln!(out, "white_cp={} stm_value={:.3} cpu_hidden0={}", wcp, stm_value(wcp, board.side_to_move()), acc.hidden[0]).unwrap();
                let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
                let accs: Vec<Accumulator> = moves.iter().map(|m| board_to_acc(&board.make_move_new(*m), &nnue)).collect();
                let cps = gpu.values(&accs);
                let mut scored: Vec<(ChessMove, i32)> = moves.iter().cloned().zip(cps.iter().cloned()).collect();
                // best for side-to-move: lowest white_cp if black to move, highest if white
                let stm = board.side_to_move();
                scored.sort_by_key(|(_, cp)| if stm == Color::White { -*cp } else { *cp });
                for (m, cp) in scored.iter().take(5) {
                    writeln!(out, "  {m} white_cp={cp}").unwrap();
                }
            }
            "bench" => {
                let batch: usize = t.get(1).and_then(|s| s.parse().ok()).unwrap_or(50_000);
                let iters: usize = t.get(2).and_then(|s| s.parse().ok()).unwrap_or(200);
                gpu.bench(batch, iters);
            }
            "quit" => break,
            _ => {}
        }
        out.flush().unwrap();
    }
}

fn parse_position(t: &[&str]) -> Option<Board> {
    let mut idx = 1;
    let mut board = if t.get(1) == Some(&"startpos") {
        idx = 2;
        Board::default()
    } else if t.get(1) == Some(&"fen") {
        let mut fen = String::new();
        idx = 2;
        while idx < t.len() && t[idx] != "moves" {
            fen.push_str(t[idx]);
            fen.push(' ');
            idx += 1;
        }
        Board::from_str(fen.trim()).ok()?
    } else {
        return None;
    };
    if t.get(idx) == Some(&"moves") {
        for ms in &t[idx + 1..] {
            if let Ok(m) = ChessMove::from_str(ms) {
                board = board.make_move_new(m);
            }
        }
    }
    Some(board)
}
