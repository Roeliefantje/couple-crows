use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{math::*, prelude::*};
use nalgebra::ComplexField;
use rand::{thread_rng, Rng};

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1080.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, apply_velocity)
        .add_systems(Update, crow_behaviour)
        .add_systems(Update, borders)
        .run();
}

#[derive(Resource)]

//Grid struct to store all crows in a grid, Grid is always centered at 0,0,0
struct Grid {
    grid: Vec<Vec<Vec<GridCell>>>,
    size: usize,
}

struct GridCell {
    crows: Vec<(Transform,Crow)>,
}

impl Grid {
    //Create new grid with size*size*size, size must be even
    fn New (size: usize) -> Self {
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
        }
    }
    //Add a crow to the grid by its transform centered around (0,0,0)
    fn Add (&mut self, transform: Transform, crow: Crow) {
        //Convert the possible negative coordinates to positive 
        //meaning that negative coordinates are between 0 and size/2 
        //and positive coordinates are between size/2 and size 
        let x = (transform.translation.x.abs() + if transform.translation.x < 0.0 {0.0} else {self.size as f32 / 2.0}) / self.size as usize;
        let y = (transform.translation.y.abs() + if transform.translation.y < 0.0 {0.0} else {self.size as f32 / 2.0}) / self.size as usize;
        let z = (transform.translation.z.abs() + if transform.translation.z < 0.0 {0.0} else {self.size as f32 / 2.0}) / self.size as usize;
        self.grid[x][y][z].crows.push((transform, crow));
    }
    //Update the grid by reevaluating the position of all crows
    fn Update (&mut self) {
        
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
    pbr: PbrBundle,
    crow: Crow,
}

impl Default for CrowBundle {
    fn default() -> Self {
        Self {
            pbr: PbrBundle::default(),
            crow: Crow::default(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    //paddle
    let size: usize = 1000;
    let mut all_cubes: Vec<CrowBundle> = Vec::with_capacity(size);
    let mut rng = thread_rng();

    for _ in 0..size {
        let x_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let y_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let z_coords = rng.gen_range(-10000..10000) as f32 / 1000.0;
        let cube = CrowBundle {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
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

fn rotate(mut query: Query<&mut Transform, With<Crow>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.)
    }
}

fn borders(mut query: Query<&mut Transform, With<Crow>>) {
    for mut transform in query.iter_mut() {
        if transform.translation.x < -10. {
            transform.translation.x = 10.;
        }
        if transform.translation.x > 10. {
            transform.translation.x = -10.;
        }

        if transform.translation.y < -10. {
            transform.translation.y = 10.;
        }
        if transform.translation.y > 10. {
            transform.translation.y = -10.;
        }
        if transform.translation.z < -10. {
            transform.translation.z = 10.;
        }
        if transform.translation.z > 10. {
            transform.translation.z = -10.;
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Crow)>, time: Res<Time>) {
    for (mut transform, crow) in query.iter_mut() {
        //transform.translation += crow.vel * time.delta_seconds();
        let new_pos = transform.translation + crow.vel * time.delta_seconds();
        transform.look_at(new_pos, Vec3::Y);
        transform.translation = new_pos;
        //println!("{}", crow.vel);
    }
}

fn crow_behaviour(
    mut query: Query<(&Transform, &mut Crow)>,
    others: Query<&Transform, With<Crow>>,
) {
    for (transform, mut crow) in query.iter_mut() {
        let seperation = calculate_seperation(&transform, &others);
        let alignment = calculate_alignment(&transform, &others);
        let cohesion = calculate_cohesion(&transform, &others);
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

fn calculate_seperation(boid: &Transform, others: &Query<&Transform, With<Crow>>) -> Vec3 {
    let mut total_seperation: Vec3 = Vec3::ZERO;

    for other_crows in others.iter() {
        let diff: f32 = boid.translation.distance(other_crows.translation);

        if diff != 0.0 && diff < SEPERATION_RADIUS {
            let direction = (other_crows.translation - boid.translation).normalize() * -1.;

            total_seperation += direction * (1.0 / diff);
        }
    }

    total_seperation
}

fn calculate_alignment(boid: &Transform, others: &Query<&Transform, With<Crow>>) -> Vec3 {
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

fn calculate_cohesion(boid: &Transform, others: &Query<&Transform, With<Crow>>) -> Vec3 {
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
