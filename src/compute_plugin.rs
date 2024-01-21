use std::{borrow::Cow, str::FromStr};
// use wgpu::util::DeviceExt;
// use wgpu::util::DeviceExt::bin
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        renderer::RenderDevice,
        render_resource::*,
    },
    ecs::storage,
    core::Pod
};
// use env_logger::fmt::buffer;
use wgpu::Queue;
use bytemuck::Zeroable;
use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

const OVERFLOW: u32 = 0xffffffff;
// const NUM_BOIDS: u32 = 4000;

// pub mod shared;
use crate::shared::*;

pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_compute);

        let params = [
            0.7, //speed
            0.03, //seperation d
            0.1, // alignment d
            0.1, // cohesion d
            0.4, // seperation s
            1., // alignment s
            1., // cohesion s
            GRID_SIZE,
            CELL_SIZE
        ];

        //Init grid
        let mut grid = Grid::new(GRID_SIZE as usize, CELL_SIZE);
        let mut initial_boids_data: Vec<Boid> = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1., 1.);

        for i in 0..NUM_BOIDS {
            let x_coords =  unif.sample(&mut rng) as f32;
            let y_coords = unif.sample(&mut rng) as f32;
            let z_coords = unif.sample(&mut rng) as f32;
            let transform = Transform::from_xyz(x_coords, y_coords, z_coords).with_scale(Vec3::splat(0.02));
            grid.add_with_transform(&transform, &(i as usize));
            initial_boids_data.push(Boid {
                pos: Vec4::new(
                    x_coords,
                    y_coords,
                    z_coords,
                    0 as f32),
                vel: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0 as f32)
            });
        }
        
        let mut amount_of_crows_vec: Vec<u32> = Vec::with_capacity(grid.size.pow(3));
        let mut crow_idxs: Vec<u32> = Vec::with_capacity(NUM_BOIDS as usize);
        
        for x in 0..grid.size {
            for y in 0..grid.size {
                for z in 0..grid.size {
                    //let grid_idx = x * grid.size.pow(2) + y * grid.size + z;
                    let amount_of_crows = grid.grid[x][y][z].crows.len();
                    for crow in &grid.grid[x][y][z].crows {
                        //This should just be the idx ideally.
                        crow_idxs.push(crow.clone() as u32);
                    }
                    let current_amount = amount_of_crows_vec.last();
                    
                    let mut total_amount = amount_of_crows as u32;
                    match current_amount {
                        Some(val) => total_amount += val,
                        None => {},
                    }
                    if amount_of_crows > 0 as usize {
                        println!("total_amount: {}, amount_of_crows: {}", total_amount, amount_of_crows);
                    }
                    
                    
                    
                    amount_of_crows_vec.push(total_amount)
                }
            }
        }

        app.insert_resource(grid);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // env_logger::init();
            pollster::block_on(prepare_compute(app, &params, &initial_boids_data, &amount_of_crows_vec, &crow_idxs));
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
            wasm_bindgen_futures::spawn_local(async move {prepare_compute(app, &vec![1233, 22343, 3234234, 42234, 52423]).await});
        }
    }
}

#[derive(Resource)]
struct ComputeResources {
    device: RenderDevice,
    queue: Queue,
    dt_uniform: Buffer,
    staging_buffer_boids: Buffer,
    boid_buffers: Vec<Buffer>,
    storage_buffer_aoc: Buffer,
    storage_buffer_cidxs: Buffer,
    pipeline: ComputePipeline,
    bind_groups: Vec<BindGroup>,
    boids_buffer_size: u64,
    aoc_buffer_size: u64,
    cidxs_buffer_size: u64,
    current_frame: usize,
}


async fn prepare_compute(
    app: &mut App,
    params: &[f32],
    boids: &[Boid],
    amount_of_crows: &[u32],
    crow_idxs: &[u32]) {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    let (wgpu_device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let device: RenderDevice = wgpu_device.into();

    let cs_module = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../assets/shaders/boids_grid.wgsl"))),
    });

    //Create Uniform Buffer for Params
    let param_buffer = create_uniform_buffer(&device, params);

    //Create Uniform Buffer for DeltaTiem
    let dt_buffer = create_uniform_buffer(&device, &vec![0.004f32]);
    

    
    //Create buffer src and dst buffers
    let (_, _, storage_buffer_boids_src) = create_buffers(&device, boids);
    let (boids_size, staging_buffer_boids, storage_buffer_boids_dst) = create_buffers(&device, boids);

    //Create buffers for grid values
    let (grid_aoc_size, _, storage_buffer_aoc) = create_buffers(&device, amount_of_crows);
    let (crowd_idxs_size, _, crow_idx_buffer) = create_buffers(&device, crow_idxs);

    let boids_storage_buffers = vec![storage_buffer_boids_src, storage_buffer_boids_dst];

    let compute_pipeline = device.create_compute_pipeline(&RawComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main",
    });

    // Instantiates the bind group, once again specifying the binding of buffers.
    // This will throw an error in case the wgsl does not use all the binds I think.
    let bind_group_layout: BindGroupLayout = compute_pipeline.get_bind_group_layout(0).into();

    // We create 2 bind groups in order to swap the src and dst of the boids, this allows us to stay consistent
    // We only need one staging buffer for both, as they are the same size.
    let mut bindgroups: Vec<BindGroup> = Vec::with_capacity(2);
    for i in 0..2 {
        bindgroups.push(device.create_bind_group(
            None, 
            &bind_group_layout, 
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: param_buffer.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 1,
                    resource: dt_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: boids_storage_buffers[i].as_entire_binding()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: boids_storage_buffers[(i + 1) % 2].as_entire_binding()
                },
                BindGroupEntry {
                    binding: 4,
                    resource: storage_buffer_aoc.as_entire_binding()
                },
                BindGroupEntry {
                    binding: 5,
                    resource: crow_idx_buffer.as_entire_binding()
                }
            ]
        ))
    }

    let compute_resources: ComputeResources = ComputeResources {
        device: device,
        queue: queue,
        dt_uniform: dt_buffer,
        staging_buffer_boids: staging_buffer_boids,
        boid_buffers: boids_storage_buffers,
        storage_buffer_aoc: storage_buffer_aoc,
        storage_buffer_cidxs: crow_idx_buffer,
        pipeline: compute_pipeline,
        bind_groups: bindgroups,
        boids_buffer_size: boids_size,
        aoc_buffer_size: grid_aoc_size,
        cidxs_buffer_size: crowd_idxs_size,
        current_frame: 0
    };

    app.insert_resource(compute_resources);

}

fn create_uniform_buffer<T: Pod>(device: &RenderDevice, data: &[T]) -> Buffer{
    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Simulation Parameter Buffer"),
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
    })
}

fn create_buffers<T: Pod>(device: &RenderDevice, data: &[T]) -> (u64, Buffer, Buffer) {
    let size = std::mem::size_of_val(data) as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer(&BufferDescriptor { 
        label: None,
        size: size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST, 
        mapped_at_creation: false,
    });

    let storage_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Storage Buffer Boids"),
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::STORAGE
            | BufferUsages::COPY_DST
            | BufferUsages::COPY_SRC,
    });

    (size, staging_buffer, storage_buffer)
}


fn run_compute(
    mut cr: ResMut<ComputeResources>, 
    mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>
) {
    //println!("Running compute!");

    cr.current_frame = (cr.current_frame + 1) % 2;

    #[cfg(not(target_arch = "wasm32"))]
    {
        // env_logger::init();
        let boids = pollster::block_on(run_compute_inner(&cr));

        q_boid
        .iter_mut()
        .for_each(|(mut transform, boid_entity)| {
            let world_pos = Vec3::new(
                20. * (boids[boid_entity.0].pos.x),
                20. * (boids[boid_entity.0].pos.y),
                20. * (boids[boid_entity.0].pos.z)
            );
            
            transform.look_at(world_pos, Vec3::Y);
            transform.translation = world_pos;

        });
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        let boids = wasm_bindgen_futures::spawn_local(run_compute_inner(&cr));
    }

    
}

async fn run_compute_inner(cr: &ComputeResources) -> Vec<Boid>{
    let boids = run_compute_shader(cr).await;

    //Update the Rendered items with the positions and rotations and create a new grid
    let mut grid = Grid::new(GRID_SIZE as usize, CELL_SIZE);
    for (i, boid) in boids.iter().enumerate() {
        let transform = Transform::from_xyz(boid.pos.x, boid.pos.y, boid.pos.z);
        grid.add_with_transform(&transform, &i);

    }

    let mut amount_of_crows_vec: Vec<u32> = Vec::with_capacity(grid.size.pow(3));
    let mut crow_idxs: Vec<u32> = Vec::with_capacity(NUM_BOIDS as usize);

    for x in 0..grid.size {
        for y in 0..grid.size {
            for z in 0..grid.size {
                    //let grid_idx = x * grid.size.pow(2) + y * grid.size + z;
                let amount_of_crows = grid.grid[x][y][z].crows.len();
                for crow in &grid.grid[x][y][z].crows {
                    //This should just be the idx ideally.
                    crow_idxs.push(crow.clone() as u32);
                }
                let current_amount = amount_of_crows_vec.last();
                    
                let mut total_amount = amount_of_crows as u32;
                match current_amount {
                    Some(val) => total_amount += val,
                    None => {},
                }
                // if amount_of_crows > 0 as usize {
                //     println!("total_amount: {}, amount_of_crows: {}", total_amount, amount_of_crows);
                // }            
                amount_of_crows_vec.push(total_amount)
            }
        }
    }


    cr.queue.write_buffer(&cr.storage_buffer_aoc, 0, bytemuck::cast_slice(&amount_of_crows_vec));
    cr.queue.write_buffer(&cr.storage_buffer_cidxs, 0, bytemuck::cast_slice(&crow_idxs));

    return boids

}

async fn run_compute_shader(cr: &ComputeResources) -> Vec<Boid>{
    let mut encoder =
        cr.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
        });
        cpass.set_pipeline(&cr.pipeline);
        cpass.set_bind_group(0, &cr.bind_groups[cr.current_frame], &[]);
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(NUM_BOIDS / 32 as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }

    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    //encoder.copy_buffer_to_buffer(&cr.storage_buffer, 0, &cr.staging_buffer, 0, cr.buffer_size);
    encoder.copy_buffer_to_buffer(&cr.boid_buffers[(cr.current_frame + 1) % 2], 0, &cr.staging_buffer_boids, 0, cr.boids_buffer_size);

    // Submits command encoder for processing
    cr.queue.submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    // let buffer_slice = cr.staging_buffer.slice(..);
    let buffer_boids_slice = cr.staging_buffer_boids.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    // let (sender, receiver) = flume::bounded(1);
    let (sender_boids, receiver_boids) = flume::bounded(1);
    // buffer_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
    buffer_boids_slice.map_async(MapMode::Read, move |v| sender_boids.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    cr.device.wgpu_device().poll(wgpu::MaintainBase::Wait);
    // device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    // Awaits until `buffer_future` can be read from
    if let Ok(Ok(())) = receiver_boids.recv_async().await {
        // Gets contents of buffer
        let data = buffer_boids_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result: Vec<Boid> = bytemuck::cast_slice(&data).to_vec();
        
        //cr.queue.write_buffer(&cr.storage_buffer, 0, bytemuck::cast_slice(&vec![1, 2, 3, 4, 5]));
        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        cr.staging_buffer_boids.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory
        
        // Returns data from buffer
        //Some(result)
        //println!("Size of Boids buffer: [{}]", result[0].pos.x);
        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}

