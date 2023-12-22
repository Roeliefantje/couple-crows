use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{gizmos, math::*, prelude::*};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::{thread, time};
// use bevy_debug_camera::{DebugCamera, DebugCameraPlugin};

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
        //Set background color to white
        .insert_resource(ClearColor(Color::WHITE))
        .run();
}

#[derive(Resource)]

//Grid struct to store all crows in a grid, Grid is always centered at 0,0,0
struct Grid {
    grid: Vec<Vec<Vec<GridCell>>>,
    size: usize,
    cell_size: f32,
}

struct GridCell {
    crows: Vec<Transform>,
}

impl Default for Grid{
    fn default() -> Self {
        Self::new(20, 1.0)
    }
}

impl Grid {
    //Create new grid with size*size*size, size must be even
    fn new (size: usize, cell_size: f32) -> Self {
        let mut grid = Vec::with_capacity(size);
        for x in 0..size {
            let mut grid_x = Vec::with_capacity(size);
            for y in 0..size {
                let mut grid_y = Vec::with_capacity(size);
                for z in 0..size {
                    grid_y.push(GridCell{crows: Vec::new()});
                }
                grid_x.push(grid_y);
            }
            grid.push(grid_x);
        }
        Self {
            grid,
            size,
            cell_size,
        }
    }

    //Add a crow to the grid by its transform centered around (0,0,0)
    fn add_with_transform (&mut self, transform: &Transform) {
        //Convert the possible negative coordinates to positive 
        //meaning that negative coordinates are between 0 and size/2 
        //and positive coordinates are between size/2 and size 
        let x = self.cooridnate_to_grid_coordinate(transform.translation.x);
        let y = self.cooridnate_to_grid_coordinate(transform.translation.y);
        let z = self.cooridnate_to_grid_coordinate(transform.translation.z);
        self.grid[x][y][z].crows.push(*transform);
    }

    fn cooridnate_to_grid_coordinate (&self, coordinate: f32) -> usize {
         ((coordinate.abs() + if coordinate < 0.0 {0.0} else {self.size as f32 * self.cell_size / 2.0}) / self.cell_size) as usize % self.size
    }

    //Get all crows in a certain radius around a certain point
    fn get_in_radius (&self, point: Vec3, radius: f32) -> Vec<&Transform> {
        let mut crows = Vec::new();
        //Get grid coordinates of the potential affected cells
        let min_x = self.cooridnate_to_grid_coordinate(point.x - radius).max(0);
        let max_x = self.cooridnate_to_grid_coordinate(point.x + radius).min(self.size);
        let min_y = self.cooridnate_to_grid_coordinate(point.y - radius).max(0);
        let max_y = self.cooridnate_to_grid_coordinate(point.y + radius).min(self.size);
        let min_z = self.cooridnate_to_grid_coordinate(point.z - radius).max(0);
        let max_z = self.cooridnate_to_grid_coordinate(point.z + radius).min(self.size);
        //Iterate over all cells in the area grid
        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    //Iterate over all crows in the cell
                    for crow in &self.grid[x][y][z].crows {
                        //Check if the crow is in the radius
                        if crow.translation.distance(point) < radius {
                            crows.push(crow);
                        }
                    }
                }
            }
        }
        crows
    }
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

const BOUND_SIZE: f32 = 40.;

fn borders(mut query: Query<&mut Transform, With<Crow>>) {
    for mut transform in query.iter_mut() {
        if transform.translation.x < -BOUND_SIZE/2. {
            transform.translation.x = BOUND_SIZE/2.;
        }
        if transform.translation.x > BOUND_SIZE/2. {
            transform.translation.x = -BOUND_SIZE/2.;
        }

        if transform.translation.y < -BOUND_SIZE/2. {
            transform.translation.y = BOUND_SIZE/2.;
        }
        if transform.translation.y > BOUND_SIZE/2. {
            transform.translation.y = -BOUND_SIZE/2.;
        }
        if transform.translation.z < -BOUND_SIZE/2. {
            transform.translation.z = BOUND_SIZE/2.;
        }
        if transform.translation.z > BOUND_SIZE/2. {
            transform.translation.z = -BOUND_SIZE/2.;
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Crow)>, time: Res<Time>, grid: Res<Grid>, mut command : Commands) {
    
    let mut new_grid = Grid::new(grid.size, grid.cell_size);
    for (mut transform, crow) in query.iter_mut() {
        //transform.translation += crow.vel * time.delta_seconds();
        let new_pos = transform.translation + crow.vel * time.delta_seconds();
        transform.look_at(new_pos, Vec3::Y);
        transform.translation = new_pos;
        //println!("{}", crow.vel);
        new_grid.add_with_transform(&transform);
    }
    command.insert_resource(new_grid);
}

fn crow_behaviour(
    mut query: Query<(&Transform, &mut Crow)>,
    others: Query<&Transform, With<Crow>>,
    grid: Res<Grid>,
) {
    for (transform, mut crow) in query.iter_mut() {
        //Find other crows transforms in a seperation_radius
        let others_seperations = grid.get_in_radius(transform.translation, SEPERATION_RADIUS);

        //Find other crows in a vision_radius
        let others_vision = grid.get_in_radius(transform.translation, VISION_RADIUS);
        
        //Calculate the seperation, alignment and cohesion
        let seperation = calculate_seperation(&transform, &others_seperations);
        let alignment = calculate_alignment(&transform, &others_vision);
        let cohesion = calculate_cohesion(&transform, &others_vision);
        //for other_transform in others.iter(){
        //    let diff = transform.translation.distance(other_transform.translation);
        //    println!("Difference: {}", diff);
        //}
        //println!("Seperation {}, Alignment {}, Cohesion{}", seperation, alignment, cohesion);
        crow.vel += seperation + alignment + cohesion;

        crow.vel = crow.vel.normalize() * 2.0;
    }
}

const SEPERATION_RADIUS: f32 = 1.2;
const VISION_RADIUS: f32 = 3.0;
const COHESION_FACTOR: f32 = 0.01;

fn calculate_seperation(boid: &Transform, others: &Vec<&Transform>) -> Vec3 {
    let mut total_seperation: Vec3 = Vec3::ZERO;

    for other_crow in others.iter() {
        let diff: f32 = boid.translation.distance(other_crow.translation);

        if diff != 0.0 && diff < SEPERATION_RADIUS {
            let direction = (other_crow.translation - boid.translation).normalize() * -1.;

            total_seperation += direction * (1.0 / diff);
        }
    }

    total_seperation.normalize_or_zero()
}

fn calculate_alignment(boid: &Transform, others: &Vec<&Transform>) -> Vec3 {
    let mut total_alignment: Vec3 = Vec3::ZERO;

    for other_crows in others.iter() {
        let diff: f32 = boid.translation.distance(other_crows.translation);

        if diff != 0.0 && diff < VISION_RADIUS {
            let direction = other_crows.forward();
            total_alignment += direction;
        }
    }

    total_alignment.normalize_or_zero()
}

fn calculate_cohesion(boid: &Transform, others: &Vec<&Transform>) -> Vec3 {
    let mut average_position: Vec3 = Vec3::ZERO;
    let mut count: u16 = 0;
    for other_crows in others.iter() {
        let diff: f32 = boid.translation.distance(other_crows.translation);

        if diff != 0.0 && diff < VISION_RADIUS {
            count += 1;
            average_position += other_crows.translation;
        }
    }
    average_position /= count as f32;

    (average_position - boid.translation).normalize_or_zero()
}
