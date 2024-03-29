//! Main file that is responsible for adding the other files and to run the application.
//! It also adds some default bevy features that allow for more easy debugging.
//! The last thing it does is setup the scene.

// Crate usages
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{math::*, prelude::*, pbr::CascadeShadowConfigBuilder, asset::AssetMetaCheck};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

//Own crate usage
pub mod shared;
use shared::*;

mod compute_plugin;
use compute_plugin::ComputePlugin;

mod instancing_plugin;
use instancing_plugin::Instancing_Plugin;


//Main, adding some useful plugins that allow for some easy logging.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync, // Doesn't limit framerate.
                fit_canvas_to_parent: true, //Webapp will turn full screen using this.
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ComputePlugin)
        .add_plugins(Instancing_Plugin)
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        .add_systems(Update, system)
        .run()
}

#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);

//Old animation function
pub fn _run_animation(animations : Res<Animations>, mut players_query : Query<&mut AnimationPlayer, Added<AnimationPlayer>>){
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

    // let glb_scene: Handle<Mesh> = asset_server.load("crow.glb#Scene0");
    


    // let mut crows: Vec<CrowBundle> = Vec::with_capacity(NUM_BOIDS as usize);
    // for i in 0..NUM_BOIDS {
    //     crows.push(CrowBundle {
    //         pbr: SceneBundle {
    //             scene: asset_server.load("crow1.glb#Scene0"),
    //             transform: Transform::default().with_scale(Vec3::splat(0.02)),
    //             ..default()
    //         },
    //         boid_entity: BoidEntity(i as usize)
    //     });
    // }

    // commands.spawn_batch(crows);

}

fn system(mut gizmos: Gizmos) {
    gizmos.cuboid(

        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(BOX_SIZE)),
        Color::GREEN,
    );
}
