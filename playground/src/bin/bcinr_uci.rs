#![allow(unsafe_code)]
use std::io::{self, BufRead};
use std::time::Instant;
use std::sync::Arc;
use chess::{Board, ChessMove, MoveGen, BoardStatus, Color};
use pollster::FutureExt;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use std::str::FromStr;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Accumulator {
    hidden: [i32; 16],
}

struct GpuSearcher {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    nnue_buffer: wgpu::Buffer,
}

impl GpuSearcher {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).block_on().unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).block_on().unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("NNUE Pipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let nnue = playground::nnue::BranchTorchNNUE::new();
        let nnue_bytes = unsafe { std::slice::from_raw_parts(&nnue as *const _ as *const u8, std::mem::size_of_val(&nnue)) };
        let nnue_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NNUE Weights"),
            contents: nnue_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self { device, queue, pipeline, nnue_buffer }
    }

    fn evaluate_batch(&self, boards: &[Accumulator]) -> Vec<f32> {
        let max_chunk = 100_000; // Safe chunk size below 128MB binding limit
        let mut results = Vec::with_capacity(boards.len());
        for chunk in boards.chunks(max_chunk) {
            results.extend_from_slice(&self.evaluate_chunk(chunk));
        }
        results
    }

    fn evaluate_chunk(&self, boards: &[Accumulator]) -> Vec<f32> {
        if boards.is_empty() { return vec![]; }
        let count = boards.len();
        
        let mut padded_count = count;
        if padded_count % 64 != 0 {
            padded_count = ((count / 64) + 1) * 64;
        }

        let mut input_data = boards.to_vec();
        input_data.resize(padded_count, Accumulator { hidden: [0; 16] });

        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input"),
            contents: bytemuck::cast_slice(&input_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output"),
            size: (padded_count * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let policy_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Policy"),
            size: (padded_count * 4 * 64) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.nnue_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: input_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: output_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: policy_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((padded_count / 64) as u32, 1, 1);
        }

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (padded_count * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, (padded_count * 4) as u64);
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        self.device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let result: Vec<f32> = bytemuck::cast_slice(&data)[..count].to_vec();
        drop(data);
        staging_buffer.unmap();

        result
    }
}

struct SearchNode {
    board: Board,
    best_move: Option<ChessMove>,
    score: f32,
    children: Vec<(ChessMove, usize)>, // (move, index in tree)
    is_leaf: bool,
    gpu_idx: usize,
}

fn board_to_gpu(b: &Board, nnue: &playground::nnue::BranchTorchNNUE) -> Accumulator {
    let mut hidden = nnue.l1_biases;
    let pieces = [
        chess::Piece::Pawn, chess::Piece::Knight, chess::Piece::Bishop,
        chess::Piece::Rook, chess::Piece::Queen, chess::Piece::King
    ];
    let mut p_idx = 0;
    for &p in &pieces {
        let w_bb = *b.color_combined(Color::White) & *b.pieces(p);
        for sq in w_bb {
            let sq_idx = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][p_idx * 64 + sq_idx];
            }
        }
        let b_bb = *b.color_combined(Color::Black) & *b.pieces(p);
        for sq in b_bb {
            let sq_idx = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][(p_idx + 6) * 64 + sq_idx];
            }
        }
        p_idx += 1;
    }
    Accumulator { hidden }
}

fn expand_tree(board: Board, depth: usize, tree: &mut Vec<SearchNode>, gpu_leaves: &mut Vec<Accumulator>, nnue: &playground::nnue::BranchTorchNNUE) -> usize {
    let node_idx = tree.len();
    tree.push(SearchNode {
        board: board.clone(),
        best_move: None,
        score: 0.0,
        children: vec![],
        is_leaf: depth == 0 || board.status() != BoardStatus::Ongoing,
        gpu_idx: 0,
    });

    if tree[node_idx].is_leaf {
        tree[node_idx].gpu_idx = gpu_leaves.len();
        gpu_leaves.push(board_to_gpu(&board, nnue));
        return node_idx;
    }

    let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
    if moves.is_empty() {
        tree[node_idx].is_leaf = true;
        tree[node_idx].gpu_idx = gpu_leaves.len();
        gpu_leaves.push(board_to_gpu(&board, nnue));
        return node_idx;
    }

    for m in moves {
        let child_board = board.make_move_new(m);
        let child_idx = expand_tree(child_board, depth - 1, tree, gpu_leaves, nnue);
        tree[node_idx].children.push((m, child_idx));
    }
    node_idx
}

fn minimax(idx: usize, tree: &mut Vec<SearchNode>, gpu_scores: &[f32], maximizing: bool) -> f32 {
    if tree[idx].is_leaf {
        // Evaluate natively based on random weights (just returning GPU output)
        let score = gpu_scores[tree[idx].gpu_idx];
        tree[idx].score = score;
        return score;
    }

    let mut best_val = if maximizing { -1000000.0 } else { 1000000.0 };
    let mut best_move = None;

    let children = tree[idx].children.clone();
    for (m, child_idx) in children {
        let val = minimax(child_idx, tree, gpu_scores, !maximizing);
        if maximizing {
            if val > best_val {
                best_val = val;
                best_move = Some(m);
            }
        } else {
            if val < best_val {
                best_val = val;
                best_move = Some(m);
            }
        }
    }

    tree[idx].score = best_val;
    tree[idx].best_move = best_move;
    best_val
}

fn main() {
    let mut searcher = GpuSearcher::new();
    let mut board = Board::default();

    println!("id name BCINR GPU 40-Core");
    println!("id author AG");
    println!("uciok");

    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let line = line_result.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() { continue; }

        match tokens[0] {
            "uci" => {
                println!("id name BCINR GPU 40-Core");
                println!("id author AG");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "position" => {
                if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = Board::default();
                    if tokens.len() > 2 && tokens[2] == "moves" {
                        for m_str in &tokens[3..] {
                            if let Ok(m) = ChessMove::from_str(m_str) {
                                board = board.make_move_new(m);
                            }
                        }
                    }
                } else if tokens.len() > 1 && tokens[1] == "fen" {
                    let mut fen = String::new();
                    let mut start = 2;
                    while start < tokens.len() && tokens[start] != "moves" {
                        fen.push_str(tokens[start]);
                        fen.push(' ');
                        start += 1;
                    }
                    if let Ok(b) = Board::from_str(fen.trim()) {
                        board = b;
                    }
                    if start < tokens.len() && tokens[start] == "moves" {
                        for m_str in &tokens[start + 1..] {
                            if let Ok(m) = ChessMove::from_str(m_str) {
                                board = board.make_move_new(m);
                            }
                        }
                    }
                }
            }
            "go" => {
                // Find a movetime if specified
                let mut max_time_ms = 1000;
                for i in 1..tokens.len() {
                    if tokens[i] == "movetime" && i + 1 < tokens.len() {
                        max_time_ms = tokens[i+1].parse::<u128>().unwrap_or(1000);
                    }
                }

                let start_time = Instant::now();
                let mut best_move = None;
                let mut depth = 1;
                
                // Iterative Deepening
                loop {
                    let mut tree = Vec::new();
                    let mut gpu_leaves = Vec::new();
                    let nnue_inst = playground::nnue::BranchTorchNNUE::new();
                    
                    let root_idx = expand_tree(board.clone(), depth, &mut tree, &mut gpu_leaves, &nnue_inst);
                    let scores = searcher.evaluate_batch(&gpu_leaves);
                    
                    minimax(root_idx, &mut tree, &scores, board.side_to_move() == Color::White);
                    
                    if let Some(m) = tree[root_idx].best_move {
                        best_move = Some(m);
                    }
                    
                    let elapsed = start_time.elapsed().as_millis();
                    println!("info depth {} nodes {} time {} nps {}", depth, tree.len(), elapsed, (tree.len() as u128 * 1000) / elapsed.max(1));
                    
                    if elapsed >= max_time_ms {
                        break;
                    }
                    
                    if depth >= 5 {
                        // Hard cap at depth 5 for the demo to prevent blowing RAM (could be 20M+ nodes)
                        break; 
                    }
                    depth += 1;
                }

                if let Some(m) = best_move {
                    println!("bestmove {}", m);
                } else {
                    let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
                    if !moves.is_empty() {
                        println!("bestmove {}", moves[0]);
                    } else {
                        println!("bestmove 0000");
                    }
                }
            }
            "quit" => {
                break;
            }
            _ => {}
        }
    }
}
