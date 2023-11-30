use bevy::{prelude::*, math::*};
use rand::{thread_rng, Rng};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1080.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Crow;

#[derive(Bundle)]
struct CrowBundle {
    pbr: PbrBundle,
    crow: Crow
}

impl Default for CrowBundle {
    fn default() -> Self {
        Self {
            pbr: PbrBundle::default(),
            crow: Crow
        }
    }
}


fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    //paddle
    let size: usize = 100000;
    let mut all_cubes: Vec<CrowBundle> = Vec::with_capacity(size);
    let mut rng = thread_rng();

    for _ in 0..size {
        let x_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let y_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let z_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let cube = CrowBundle {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube {size: 0.1})),
                material: materials.add(Color::rgb_u8(124, 144, 166).into()),
                transform: Transform::from_xyz(x_coords, y_coords, z_coords),
                ..default()
            },
            ..default()
        };
        all_cubes.push(cube)
    }
    commands.spawn_batch(all_cubes);
}
