use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{gizmos, math::*, prelude::*};
use bevy::{pbr::CascadeShadowConfigBuilder};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use std::{thread, time};
// use bevy_debug_camera::{DebugCamera, DebugCameraPlugin};

//[MODULES]
use crate::{grid_architecture::Grid};
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
        //.add_system(Update, movement_system.system())
        .add_systems(Update, update_crow_lod)
    //    .add_system(Update, movement_system.system())
    //    .register_component::<Velocity>()
        //Set background color to white
        .insert_resource(ClearColor(Color::WHITE))
        .run();
}



#[derive(Component)]
struct Crow {
    vel: Vec3,
    lod: LOD
}

impl Default for Crow {
    fn default() -> Self {
        let mut rng = thread_rng();

        let x_coords = rng.gen_range(0..200) as f32 / 100.0;
        let y_coords = rng.gen_range(0..200) as f32 / 100.0;
        let z_coords = rng.gen_range(0..200) as f32 / 100.0;
        Self {
            vel: Vec3::new(x_coords, y_coords, z_coords).normalize(),
            lod : LOD::High
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

#[derive(PartialEq, Eq)]
enum LOD{
    High,
    Medium,
    Low
}

#[derive(Resource)]
struct CrowModels{
    high : Handle<Scene>,
    medium: Handle<Scene>,
    low: Handle<Scene>
}

#[derive(Component)]
struct LODCorner;

pub fn run_animation(
    animations: Res<Animations>,
    mut query: Query<(Entity, &mut AnimationPlayer)>,
) {
    // let entity_count = query.iter_mut().count();
    // println!("Number of entities with AnimationPlayer and Crow: {}", entity_count);
    // let mut rng = thread_rng();
    // for (entity, mut player) in query.iter_mut() {
    //     let animation_clips = &animations.0;
    //     let animation = match LOD::High {
    //         LOD::High => animation_clips[0].clone(),
    //         LOD::Medium => animation_clips[1].clone()
    //         LOD::Low => animation_clips[1].clone()
    //     };

    //     player.play(animation).repeat();
    //     player.seek_to(rng.gen_range(0..10000) as f32 / 10000.0);
    //     player.set_speed((rng.gen_range(0..5000) as f32 / 10000.0) + 1.0);
    // }
}

#[derive(Resource)]
struct FrameCounter(usize);

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

    //Marker to track the corner of the LOD area
    // let cube_material = materials.add(Color::rgb(0.7, 0.7, 0.7).into());
    // let cube_position = Vec3::new(0.0, 0.0, 0.0);
    // commands.spawn(PbrBundle {
    //     mesh:  meshes.add(Mesh::from(shape::Cube { size: 1. })),
    //     material: cube_material,
    //     transform: Transform::from_translation(cube_position),
    //     ..Default::default()
    // }).insert(LODCorner);
    
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.insert_resource(FrameCounter(0)); //initialize frame counter

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

    let crow_models = CrowModels {
        high: asset_server.load("crow1.glb#Scene0"),
        medium: asset_server.load("crow2.glb#Scene0"),
        low: asset_server.load("crow2.glb#Scene0"),
    };
    commands.insert_resource(crow_models);

    commands.insert_resource(Animations(vec![asset_server.load("crow1.glb#Animation0")]));
    commands.insert_resource(Animations(vec![asset_server.load("crow2.glb#Animation0")]));
    // Grid
    let mut grid = Grid::new(20, 1.0);

    //paddle
    let size: usize = 2000;
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

fn update_crow_lod(
    mut commands: Commands,
    camera_query: Query<&Transform, (With<Camera>, Without<LODCorner>)>,
    mut crow_query: Query<(&mut Handle<Scene>, &mut Crow, &Transform), Without<LODCorner>>,
    grid: Res<Grid>,
    models: Res<CrowModels>,
    mut marker_query: Query<(&mut Transform, &LODCorner)>
) {
    let camera_transform = camera_query.single();
    //let (mut marker_trans, _) = marker_query.single_mut();

    let affected_crows = grid.get_crows_in_lod_change_area(camera_transform, 50.0);
    let (x, y, z) = grid.get_lod_corner_cell(camera_transform, 30.0);
    //marker_trans.translation = Vec3::new(grid.grid_coordinate_to_coordinate(x), grid.grid_coordinate_to_coordinate(y), grid.grid_coordinate_to_coordinate(z));


    for (mut scene_handle, mut crow, transform) in crow_query.iter_mut() {
        let distance = camera_transform.translation.distance(transform.translation);
        let new_lod = if distance < 30.0 {
            LOD::High
        } else if distance < 50.0 {
            LOD::Medium
        } else {
            LOD::Low
        };

        if crow.lod != new_lod {
            *scene_handle = match new_lod {
                LOD::High => models.high.clone(),
                LOD::Medium => models.medium.clone(),
                LOD::Low => models.low.clone(),
            };
            crow.lod = new_lod;
        }
    }
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

//    struct Velocity(Vec3);
//    const CROW_SPEED: f32 = 2.0;
// struct Velocity(Vec3);
// const CROW_SPEED: f32 = 2.0;

//    fn movement_system(
//        mut frame_counter: ResMut<FrameCounter>,
//        time: Res<Time>,
//        mut query: Query<(&mut Transform, &Velocity), With<Crow>>,
//    ) {
//        if frame_counter.0 % 2 == 0 { //skip every other frame
//            for (mut transform, velocity) in query.iter_mut() {
//                transform.translation += velocity.0 * CROW_SPEED * time.delta_seconds();
//            }
//        }
//        frame_counter.0 += 1;
//    }

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

