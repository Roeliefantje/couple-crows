//! This file holds some of the structs and values we have used in multiple files or to easily modify it.
//! ie. the NUM_BOIDS value is responsible for the amount of crows that will be rendered.
//! It also holds the grid struct we use in order to speed up the computation of the boid algorithm.

use bevy::{
    prelude::*,
    core::Pod,
    render::render_resource::ShaderType,
    render::extract_component::{ExtractComponent, ExtractComponentPlugin},
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    }
};
use bytemuck::Zeroable;

pub const NUM_BOIDS: u32 = 128000;
pub const BOX_SIZE: f32 = 40.;
//Grid_size * cell_size should be 2.0
pub const GRID_SIZE: f32 = 20.0;
pub const CELL_SIZE: f32 = 0.1;

// Boid struct that gets transfered over to the compute shader which includes all the information needed for the computation.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Boid {
    pub pos: Vec4,
    pub vel: Vec4,
}

// Params we can set in order to change the behaviour of the compute shader.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Params {
    pub speed: f32,
    pub seperation_distance: f32,
    pub alignment_distance: f32,
    pub cohesion_distance: f32,
    pub seperation_scale: f32,
    pub alignment_scale: f32,
    pub cohesion_scale: f32,
    pub grid_size: f32,
    pub cell_size: f32,
}



//Identifier in order to link the boids data to a texture.
#[derive(Component)]
pub struct BoidEntity(pub usize);

//The bundle that gets spawned in with the texture / mesh of the boid
#[derive(Bundle)]
pub struct CrowBundle {
    pub pbr: SceneBundle,
    pub boid_entity: BoidEntity,
}


#[derive(Resource)]

//Grid struct to store all crows in a grid, Grid is always centered at 0,0,0
pub struct Grid {
    pub grid: Vec<Vec<Vec<GridCell>>>,
    pub size: usize,
    pub cell_size: f32,
}

pub struct GridCell {
    pub crows: Vec<usize>,
}

impl Default for Grid{
    fn default() -> Self {
        Self::new(20, 1.0)
    }
}

impl Grid {
    //Create new grid with size*size*size, size must be even
    pub fn new (size: usize, cell_size: f32) -> Self {
        let mut grid = Vec::with_capacity(size);
        for _x in 0..size {
            let mut grid_x = Vec::with_capacity(size);
            for _y in 0..size {
                let mut grid_y = Vec::with_capacity(size);
                for _z in 0..size {
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
    pub fn add_with_transform (&mut self, transform: &Transform, idx: &usize) {
        let x = self.cooridnate_to_grid_coordinate(transform.translation.x);
        let y = self.cooridnate_to_grid_coordinate(transform.translation.y);
        let z = self.cooridnate_to_grid_coordinate(transform.translation.z);
        self.grid[x][y][z].crows.push(*idx);
    }

    pub fn cooridnate_to_grid_coordinate (&self, coordinate: f32) -> usize {
        //negative value would be -1 + 2 if size is say 4, resulting in 1, positive values will be 1 + 2 = 3, so i
        let val = ((coordinate / self.cell_size) + (self.size as f32 * 0.5)) as usize % self.size;
        val
        
    }
}


#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub vel: Vec4,
    pub color: [f32; 4],
}

#[derive(Component, Deref)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}