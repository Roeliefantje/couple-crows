use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{gizmos, math::*, prelude::*};
use bevy::{pbr::CascadeShadowConfigBuilder};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use std::{thread, time};
// use bevy_debug_camera::{DebugCamera, DebugCameraPlugin};

//[MODULES]
use crate::grid_architecture::*;
use crate::boid_movement::*;
mod boid_movement;
mod grid_architecture;


pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1080.0;
pub const BOX_SIZE: f32 = 20.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, run_animation)
        .add_systems(Update, system)
        .add_systems(Update, apply_velocity)
        .add_systems(Update, crow_behaviour)
        .add_systems(Update, borders)
        .add_system(Update, movement_system.system())
        .register_component::<Velocity>()
        //Set background color to white
        .insert_resource(ClearColor(Color::WHITE))
        .run();
}



#[derive(Component)]
struct Crow {
    vel: Vec3,
}

impl Default for Crow {
    fn default() -> Self {
        let mut rng = thread_rng();

        let x_coords = rng.gen_range(0..200) as f32 / 100.0;
        let y_coords = rng.gen_range(0..200) as f32 / 100.0;
        let z_coords = rng.gen_range(0..200) as f32 / 100.0;
        Self {
            vel: Vec3::new(x_coords, y_coords, z_coords).normalize(),
        }
    }
}

#[derive(Bundle)]
struct CrowBundle {
    pbr: SceneBundle,
    crow: Crow,
}

impl Default for CrowBundle {
    fn default() -> Self {
        Self {
            pbr: SceneBundle::default(),
            crow: Crow::default(),
        }
    }
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

struct FrameCounter(usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    commands.insert_resource(FrameCounter(0)), //initialize frame counter
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

    // Grid
    let mut grid = Grid::new(20, 1.0);

    //paddle
    let size: usize = 1000;
    let mut crows = Vec::with_capacity(size);
    let mut rng = thread_rng();

    for _ in 0..size {
        let x_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let y_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let z_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let transform = Transform::from_xyz(x_coords, y_coords, z_coords).with_scale(Vec3::splat(0.02));
        let crow = CrowBundle {
            pbr : SceneBundle {
            scene: asset_server.load("crow1.glb#Scene0"),
            transform,
            ..default()
            },
        ..default()
        };
        crows.push(crow);
        grid.add_with_transform(&transform);
    }
    commands.spawn_batch(crows);
    commands.insert_resource(grid);
}



fn system(mut gizmos: Gizmos) {
    gizmos.cuboid(

        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(BOX_SIZE)),
        Color::GREEN,
    );
}

// fn rotate(mut query: Query<&mut Transform, With<Crow>>, time: Res<Time>) {
//     for mut transform in &mut query {
//         transform.rotate_y(time.delta_seconds() / 2.)
//     }
// }

struct Velocity(Vec3);
const CROW_SPEED: f32 = 2.0;

fn movement_system(
    mut frame_counter: ResMut<FrameCounter>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Crow>>,
) {
    if frame_counter.0 % 2 == 0 { //skip every other frame
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0 * CROW_SPEED * time.delta_seconds();
        }
    }
    frame_counter.0 += 1;
}

fn borders(mut query: Query<&mut Transform, With<Crow>>) {
    for mut transform in query.iter_mut() {
        if transform.translation.x < -BOX_SIZE/2. {
            transform.translation.x = BOX_SIZE/2.;
        }
        if transform.translation.x > BOX_SIZE/2. {
            transform.translation.x = -BOX_SIZE/2.;
        }

        if transform.translation.y < -BOX_SIZE/2. {
            transform.translation.y = BOX_SIZE/2.;
        }
        if transform.translation.y > BOX_SIZE/2. {
            transform.translation.y = -BOX_SIZE/2.;
        }
        if transform.translation.z < -BOX_SIZE/2. {
            transform.translation.z = BOX_SIZE/2.;
        }
        if transform.translation.z > BOX_SIZE/2. {
            transform.translation.z = -BOX_SIZE/2.;
        }
    }
}

