use bevy::{gizmos, math::*, prelude::*};

use crate::{Crow, grid_architecture::Grid};

const SEPERATION_RADIUS: f32 = 1.2;
const VISION_RADIUS: f32 = 3.0;
const COHESION_FACTOR: f32 = 0.01;
const SEPERATION_FACTOR: f32 = 1.0;
const ALIGNMENT_FACTOR: f32 = 0.125;

pub fn apply_velocity(mut query: Query<(&mut Transform, &Crow)>, time: Res<Time>, grid: Res<Grid>, mut command : Commands) {
    
    let mut new_grid = Grid::new(grid.size, grid.cell_size);
    for (mut transform, crow) in query.iter_mut() {
        //transform.translation += crow.vel * time.delta_seconds();
        let new_pos = transform.translation + crow.vel * time.delta_seconds();
        transform.look_at(new_pos, Vec3::Y);
        transform.translation = new_pos;
        //println!("{}", crow.vel);
        new_grid.add_with_transform(&transform, crow);
        //println!("New pos: {}", transform.translation);
    }
    command.insert_resource(new_grid);
    
}

pub fn crow_behaviour(
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

pub fn calculate_seperation(boid: &Transform, others: &Vec<&(Transform,Crow)>) -> Vec3 {
    let mut total_seperation: Vec3 = Vec3::ZERO;

    for other_crow in others.iter() {

        let diff: f32 = boid.translation.distance(other_crow.0.translation);

        if diff != 0.0 {
            total_seperation -= (other_crow.0.translation - boid.translation);
        }
    }
    total_seperation * SEPERATION_FACTOR
}

pub fn calculate_alignment(boid: &Transform, others: &Vec<&(Transform,Crow)>) -> Vec3 {
    let mut total_alignment: Vec3 = Vec3::ZERO;
    if !others.is_empty() {

        for other_crow in others.iter() {
            let diff: f32 = boid.translation.distance(other_crow.0.translation);
    
            if diff != 0.0 {
                total_alignment += other_crow.1.vel.normalize();
            }
        }
        total_alignment /= others.len() as f32;
    }
    //println!("Total alignment: {}", total_alignment);
    total_alignment * ALIGNMENT_FACTOR
}

pub fn calculate_cohesion(boid: &Transform, others: &Vec<&(Transform,Crow)>) -> Vec3 {
    let mut average_position: Vec3 = Vec3::ZERO;

    if !others.is_empty() {
        
        let mut count: u16 = 0;
        for other_crows in others.iter() {
            let diff: f32 = boid.translation.distance(other_crows.0.translation);

            if diff != 0.0 {
                count += 1;
                average_position += other_crows.0.translation;
            }
        }
        average_position /= count as f32;
    }
    (average_position - boid.translation) * COHESION_FACTOR
}
