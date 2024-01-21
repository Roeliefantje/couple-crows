//! Example showing how to calculate boids data from compute shaders
//! For now they are stupid and just fly straight, need to fix this later on.
//! Reimplementation of https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/main.rs
//! 
//! Add a system to startup in order to have the Resources for the Render Device and the queue

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::world;
use bevy::{gizmos, math::*, prelude::*, pbr::CascadeShadowConfigBuilder, asset::AssetMetaCheck, core::Pod};

use bevy_app_compute::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bytemuck::Zeroable;

use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

// const NUM_BOIDS: u32 = 50000;

pub mod shared;
use shared::*;

mod compute_plugin;
use compute_plugin::ComputePlugin;




//Main, adding some useful plugins that allow for some easy logging.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ComputePlugin)
        .insert_resource(AssetMetaCheck::Never)
        // .add_plugins(AppComputePlugin)
        // .add_plugins(AppComputeWorkerPlugin::<BoidWorker>::default())
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        //.add_systems(Update, move_entities)
        .add_systems(Update, system)
//        .add_systems(Update, borders)
        .run()
}

#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);


pub fn run_animation(animations : Res<Animations>, mut players_query : Query<&mut AnimationPlayer, Added<AnimationPlayer>>){
    let mut rng = thread_rng();
    for mut player in &mut players_query{
        player.play(animations.0[0].clone()).repeat();
        player.seek_to(rng.gen_range(0..10000) as f32 / 10000.0);
        player.set_speed((rng.gen_range(0..5000) as f32 / 10000.0) + 1.0);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {

    // Flying Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
    ));


    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(BOX_SIZE*2.))),
        material: materials.add(Color::rgb(0.3, 0.9, 0.3).into()),
        transform: Transform::from_xyz(0., -BOX_SIZE*0.5, 0.),
        ..default()
    });

    // testing cube (delete later)
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 4. })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, 0., 0.0),
    //     ..default()
    // });
    
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    // direction light (sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.insert_resource(Animations(vec![asset_server.load("crow1.glb#Animation0")]));

    let mut crows: Vec<CrowBundle> = Vec::with_capacity(NUM_BOIDS as usize);
    for i in 0..NUM_BOIDS {
        crows.push(CrowBundle {
            pbr: SceneBundle {
                scene: asset_server.load("crow1.glb#Scene0"),
                transform: Transform::default().with_scale(Vec3::splat(0.02)),
                ..default()
            },
            boid_entity: BoidEntity(i as usize)
        });
    }

    commands.spawn_batch(crows);

}



//This part is to use the boids shader. I am using a plugin I found that helps with a lot of the boilerplate code
//It took quite some time to get this working, but it turns out the float3 does not align properly (which is why we are using vec4 for the boid information)
//https://github.com/Kjolnyr/bevy_app_compute
#[derive(TypeUuid)]
#[uuid = "2545ac14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct BoidsShader;

impl ComputeShader for BoidsShader {
    fn shader() -> ShaderRef {
        "shaders/boids_grid.wgsl".into()
    }
}

struct BoidWorker;

//This is what instantiates the compute shader and sets it up to be ran every fram.e
//We use 2 buffers for the boids in order to ensure behaviour is the same every time.
impl ComputeWorker for BoidWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {

        let asset_server: &AssetServer = world.resource();
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

        let mut crows: Vec<CrowBundle> = Vec::with_capacity(NUM_BOIDS as usize);
        let mut initial_boids_data: Vec<Boid> = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1., 1.);

        for i in 0..NUM_BOIDS {
            let x_coords =  unif.sample(&mut rng) as f32;
            let y_coords = unif.sample(&mut rng) as f32;
            let z_coords = unif.sample(&mut rng) as f32;
            let transform = Transform::from_xyz(x_coords, y_coords, z_coords).with_scale(Vec3::splat(0.02));
            grid.add_with_transform(&transform, &(i as usize));
            
            crows.push(CrowBundle {
                pbr: SceneBundle {
                    scene: asset_server.load("crow1.glb#Scene0"),
                    transform: transform,
                    ..default()
                },
                boid_entity: BoidEntity(i as usize)
            });

            initial_boids_data.push(Boid {
                pos: Vec4::new(
                    x_coords,
                    y_coords,
                    z_coords,
                    0.),
                vel: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0.)
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

        world.spawn_batch(crows);
        world.insert_resource(grid);

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_uniform("delta_time", &0.004f32)
            .add_staging("boids_src", &initial_boids_data)
            .add_staging("boids_dst", &initial_boids_data)
            .add_staging("amount_of_crows_vec", &amount_of_crows_vec)
            .add_staging("crow_idxs", &crow_idxs)
            .add_pass::<BoidsShader>(
                [NUM_BOIDS / 32 as u32, 1, 1],
                &["params", "delta_time", "boids_src", "boids_dst", "amount_of_crows_vec", "crow_idxs"],
            )
            .add_swap("boids_src", "boids_dst")
            .build()
    }
}



//This function reads the data from the compute shader and applies them to the crows rendered on the screen.
fn move_entities(
    time: Res<Time>,
    mut worker: ResMut<AppComputeWorker<BoidWorker>>,
    mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>,
    mut commands: Commands
) {
    if !worker.ready() {
        return;
    }
    let move_entities_span = info_span!("move_entities", name = "move_entities").entered();


    let mut grid = Grid::new(GRID_SIZE as usize, CELL_SIZE);
    let boids: Vec<Boid> = worker.read_vec::<Boid>("boids_dst");
    worker.write("delta_time", &time.delta_seconds());
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
            grid.add_with_transform(&Transform::from_translation(world_pos / (20 as f32)), &boid_entity.0)

        });
    
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
    worker.write_slice("amount_of_crows_vec", &amount_of_crows_vec);
    worker.write_slice("crow_idxs", &crow_idxs);
    commands.insert_resource(grid);
}

fn system(mut gizmos: Gizmos) {
    gizmos.cuboid(

        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(BOX_SIZE)),
        Color::GREEN,
    );
}
