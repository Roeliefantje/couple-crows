//! Example showing how to calculate boids data from compute shaders
//! For now they are stupid and just fly straight, need to fix this later on.
//! Reimplementation of https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/main.rs

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{
    asset::AssetMetaCheck,
    core::Pod,
    prelude::*,
};

use bevy_app_compute::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bytemuck::Zeroable;

use rand::distributions::{Distribution, Uniform};

const NUM_BOIDS: u32 = 10000;

pub const BOX_SIZE: f32 = 40.;

// Boid struct that gets transfered over to the compute shader which includes all the information needed for the computation.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Boid {
    pos: Vec4,
    vel: Vec4,
}

// Params we can set in order to change the behaviour of the compute shader.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Params {
    speed: f32,
    seperation_distance: f32,
    alignment_distance: f32,
    cohesion_distance: f32,
    seperation_scale: f32,
    alignment_scale: f32,
    cohesion_scale: f32,
}

//Identifier in order to link the boids data to a texture.
#[derive(Component)]
struct BoidEntity(pub usize);

//The bundle that gets spawned in with the texture / mesh of the boid
#[derive(Bundle)]
struct CrowBundle {
    pbr: SceneBundle,
    boid_entity: BoidEntity,
}

//Main, adding some useful plugins that allow for some easy logging.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(AppComputePlugin)
        .add_plugins(AppComputeWorkerPlugin::<BoidWorker>::default())
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_systems(Startup, setup)
        .add_systems(Update, move_entities)
        .add_systems(Update, system)
        .run()
}

//The main setup of the program. Basically just creates all the (World) boids and the setup for the camera.
//So it does not create the position etc, but it it creates the mesh for them and has a unique idx we use to transfer the boids data.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {

    //Flying Camera
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

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });


    let mut all_boids: Vec<CrowBundle> = Vec::with_capacity(NUM_BOIDS as usize);
    //let boid_mesh = meshes.add(Mesh::from(shape::Cube {size: 0.1})); 
    //let boid_material = materials.add(Color::ANTIQUE_WHITE.into());

    for i in 0..NUM_BOIDS {
        let crow = CrowBundle {
            pbr: SceneBundle {
                scene: asset_server.load("objects/crow1.glb#Scene0"),
                transform: Transform::from_xyz(0., 0., 0.)
                .with_scale(Vec3::splat(0.02)),
                ..default()
            },
            boid_entity: BoidEntity(i as usize)
        };
        all_boids.push(crow);
    }

    commands.spawn_batch(all_boids);

}



//This part is to use the boids shader. I am using a plugin I found that helps with a lot of the boilerplate code
//It took quite some time to get this working, but it turns out the float3 does not align properly (which is why we are using vec4 for the boid information)
//https://github.com/Kjolnyr/bevy_app_compute
#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct BoidsShader;

impl ComputeShader for BoidsShader {
    fn shader() -> ShaderRef {
        "shaders/boids.wgsl".into()
    }
}

struct BoidWorker;

//This is what instantiates the compute shader and sets it up to be ran every fram.e
//We use 2 buffers for the boids in order to ensure behaviour is the same every time.
impl ComputeWorker for BoidWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = Params {
            speed: 0.7,
            seperation_distance: 0.03,
            alignment_distance: 0.1,
            cohesion_distance: 0.1,
            seperation_scale: 0.4,
            alignment_scale: 1.,
            cohesion_scale: 1.,
        };

        let mut initial_boids_data: Vec<Boid> = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1., 1.);

        for _ in 0..NUM_BOIDS {
            initial_boids_data.push(Boid {
                pos: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0.),
                vel: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0.)
            });
        }

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_uniform("delta_time", &0.004f32)
            .add_staging("boids_src", &initial_boids_data)
            .add_staging("boids_dst", &initial_boids_data)
            .add_pass::<BoidsShader>(
                [NUM_BOIDS, 1, 1],
                &["params", "delta_time", "boids_src", "boids_dst"],
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
) {
    if !worker.ready() {
        return;
    }

    let boids: Vec<Boid> = worker.read_vec::<Boid>("boids_dst");
    worker.write("delta_time", &time.delta_seconds());
    q_boid
        .par_iter_mut()
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

fn system(mut gizmos: Gizmos) {
    gizmos.cuboid(

        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(BOX_SIZE)),
        Color::GREEN,
    );
}
