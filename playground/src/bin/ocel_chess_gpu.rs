//! GPU (wgpu compute-shader) variant of the OCEL chess NNUE evaluation drill
//! — mirrors `ocel_chess`'s CPU evaluation loop but dispatches the NNUE
//! forward pass to the GPU for throughput comparison.
#![allow(unsafe_code)]
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use pollster::FutureExt;

#[repr(C)]
#[derive(Clone, Copy)]
struct NNUE {
    l1_weights: [i32; 12288],
    l1_biases: [i32; 16],
    l2_weights: [i32; 16],
    l2_bias: i32,
    _pad: [i32; 3], // Align 16 bytes
}
unsafe impl Pod for NNUE {}
unsafe impl Zeroable for NNUE {}

#[repr(C)]
#[derive(Clone, Copy)]
struct GameState {
    boards_low: [u32; 12],
    boards_high: [u32; 12],
}
unsafe impl Pod for GameState {}
unsafe impl Zeroable for GameState {}

async fn run() {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .unwrap();

    let (device, queue) = adapter.request_device(&Default::default(), None).await.unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
    });

    let nnue_data = vec![NNUE {
        l1_weights: [1; 12288],
        l1_biases: [0; 16],
        l2_weights: [1; 16],
        l2_bias: 0,
        _pad: [0; 3],
    }];

    let num_games = 1_048_576; // 2^20 massive game array
    let mut game_data = vec![GameState { boards_low: [0; 12], boards_high: [0; 12] }; num_games];
    // Seed the boards
    for i in 0..num_games {
        game_data[i].boards_low[0] = 0x0000_FFFF;
    }

    let nnue_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("NNUE Buffer"),
        size: std::mem::size_of::<NNUE>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&nnue_buffer, 0, bytemuck::cast_slice(&nnue_data));

    let games_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Games Buffer"),
        size: (std::mem::size_of::<GameState>() * num_games) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&games_buffer, 0, bytemuck::cast_slice(&game_data));

    let output_buffer_size = (num_games * 4) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: nnue_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: games_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: output_buffer.as_entire_binding() },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
        compilation_options: Default::default(),
    });

    let start = Instant::now();

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups((num_games as u32).div_ceil(64), 1, 1);
    }

    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait); // Block until GPU finishes all 1 million matrices!

    let duration = start.elapsed();

    println!("--- METAL GPU 40-CORE NNUE EXECUTION PASS ---");
    println!("Games Evaluated: {num_games}");
    println!("Total Execution Time: {duration:?}");
    println!("Nodes Per Second (NPS): {}", (num_games as f64 / duration.as_secs_f64()) as u64);
}

fn main() {
    run().block_on();
}
