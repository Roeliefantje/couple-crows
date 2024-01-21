use std::fmt;
use std::collections::HashMap;

use crate::LOD;
use crate::GridZone;
use bevy::{math::*, prelude::*};

fn main() {
    // Your main code here
}

#[derive(Resource)]

//Grid struct to store all crows in a grid, Grid is always centered at 0,0,0
pub struct Grid{
    pub grid: Vec<Vec<Vec<GridCell>>>,
    pub size: usize,
    pub cell_size: f32,
}

pub struct GridCell{
    crows: Vec<GridItem>
}

pub struct GridItem{
    transform: Transform,
    id: usize
}

impl Default for Grid{
    fn default() -> Self {
        Self::new(20, 1.0)
    }
}

impl Grid{
    //Create new grid with size*size*size, size must be even
    pub fn new (size: usize, cell_size: f32) -> Self {
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
    pub fn add_with_transform_and_id (&mut self, transform: &Transform, id: usize) {
        //Convert the possible negative coordinates to positive 
        //meaning that negative coordinates are between 0 and size/2 
        //and positive coordinates are between size/2 and size 
        let x = self.cooridnate_to_grid_coordinate(transform.translation.x) % self.size;
        let y = self.cooridnate_to_grid_coordinate(transform.translation.y) % self.size;
        let z = self.cooridnate_to_grid_coordinate(transform.translation.z) % self.size;
        self.grid[x][y][z].crows.push(GridItem{transform: *transform, id: id});
    }

    pub fn cooridnate_to_grid_coordinate(&self, coordinate: f32) -> usize {
        ((coordinate / self.cell_size) as f32  + (self.size / 2) as f32).max(0.0).min((self.size - 1) as f32) as usize
    }

    pub fn grid_coordinate_to_coordinate(&self, grid_coordinate: usize) -> f32 {
        (grid_coordinate as f32 - (self.size / 2) as f32) * self.cell_size
    }    

    //Get all crows in a certain radius around a certain point
    pub fn get_in_radius (&self, point: Vec3, radius: f32) -> Vec<&Transform> {
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
                        if crow.transform.translation.distance(point) < radius {
                            crows.push(&crow.transform);
                        }
                    }
                }

            }
        }
        crows
    }

    pub fn get_lod_corner_cell (&self, camera_transform: &Transform, lod_dist: f32) -> (usize, usize, usize) {
        let p = camera_transform.translation;
        let d = f32::sqrt(p.x.powi(2) + p.y.powi(2) + p.z.powi(2));
        let t = lod_dist / d;

        let cell_x = self.cooridnate_to_grid_coordinate((1.0 - t) * p.x);
        let cell_y = self.cooridnate_to_grid_coordinate((1.0 - t) * p.y);
        let cell_z = self.cooridnate_to_grid_coordinate((1.0 - t) * p.z);
    
        return (cell_x, cell_y, cell_z);
    }

    pub fn get_grid_far_corner (&self, camera_transform: &Transform)  -> (usize, usize, usize){
        return (
            if camera_transform.translation.x < 0.0 { self.size } else { 0 },
            if camera_transform.translation.y < 0.0 { self.size } else { 0 }, 
            if camera_transform.translation.z < 0.0 { self.size } else { 0 }
        )
    }

    pub fn get_crows_in_transition(&self, new_corner: &GridZone, camera_transform: &Transform) -> Vec<(usize, LOD)> {
        let mut crows = Vec::new();
        let mut crows_vec = Vec::new();
        let (sign_x, sign_y, sign_z) = Grid::get_camera_signs(camera_transform);
        self.gather_crows_from_corner(&mut crows_vec, new_corner.near_corner, camera_transform);
        for crw in crows_vec {
            crows.push((crw.id, LOD::High));
        }
        crows_vec = Vec::new();
        self.gather_crows_from_corner(&mut crows_vec, ((new_corner.near_corner.0 as f32 - sign_x) as usize, (new_corner.near_corner.1 as f32 - sign_x) as usize, (new_corner.near_corner.2 as f32 - sign_x) as usize), camera_transform);
        for crw in crows_vec {
            crows.push((crw.id, LOD::Low));
        }
        crows
    }

    fn get_camera_signs(camera_transform: &Transform) -> (f32, f32, f32){
        return(
            if camera_transform.translation.x < 0.0 { -1.0 } else { 1.0 },
            if camera_transform.translation.y < 0.0 { -1.0 } else { 1.0 }, 
            if camera_transform.translation.z < 0.0 { -1.0 } else { 1.0 }
        )
    }

    fn gather_crows_from_corner<'a> (&'a self, crows: &mut Vec<&'a GridItem>, corner: (usize, usize, usize), camera_transform: &Transform, ){
        let (c_x, c_y, c_z) = corner;
        let (sign_x, sign_y, sign_z) = Grid::get_camera_signs(camera_transform);
        let mut x = c_x;
        let mut y = c_y;
        let mut z = c_z;
        while(x >= 0 && x < self.size){
            while(y >= 0 && y < self.size){
                if(!(x == c_z && y == c_y && z == c_z)){
                    for crow in &self.grid[x][y][c_z].crows {
                        crows.push(crow);
                    }
                }
                y = (y as i32 + 1 * (-sign_y as i32)) as usize;
            }
            x = (x as i32 + 1 * (-sign_x as i32)) as usize;
        }
        x = c_x;
        y = c_y;
        z = c_z;
        while(x >= 0 && x < self.size){
            while(z >= 0 && z < self.size){
                if(!(x == c_z && y == c_y && z == c_z)){
                    for crow in &self.grid[x][c_y][z].crows {
                        crows.push(crow);
                    }
                }
                z = (z as i32 + 1 * (-sign_z as i32)) as usize;
            }
            x = (x as i32 + 1 * (-sign_x as i32)) as usize;
        }
        x = c_x;
        y = c_y;
        z = c_z;
        while(z >= 0 && z < self.size){
            while(y >= 0 && y < self.size){
                if(!(x == c_z && y == c_y && z == c_z)){
                    for crow in &self.grid[c_x][y][z].crows {
                        crows.push(crow);
                    }
                }
                y = (y as i32 + 1 * (-sign_y as i32)) as usize;
            }
            z = (z as i32 + 1 * (-sign_z as i32)) as usize;
        }
    }

    pub fn get_all_lod(&self, camera_transform: &Transform, lod_dist: f32) -> HashMap<usize, LOD> {
        let mut close_crows = Vec::new();
        let mut far_crows = Vec::new();
        let (sign_x, sign_y, sign_z) = Grid::get_camera_signs(camera_transform);
        let (c_x, c_y, c_z) = self.get_lod_corner_cell(camera_transform, lod_dist);
        for x in 0..self.size {
            for y in 0..self.size {
                for z in 0..self.size {
                    for crow in &self.grid[x][y][z].crows {
                        if
                            ((sign_x < 0.0 && c_x < x) || (sign_x > 0.0 && c_x > x)) &&
                            ((sign_y < 0.0 && c_y < y) || (sign_y > 0.0 && c_y > y)) &&
                            ((sign_z < 0.0 && c_z < z) || (sign_z > 0.0 && c_z > z))
                        {
                            far_crows.push(crow.id);
                        } else{
                            close_crows.push(crow.id);
                        }
                    }
                }
            }
        }
        let mut map = HashMap::new();
        for crow_id in close_crows{
            map.insert(crow_id, LOD::High);
        }
        for crow_id in far_crows{
            map.insert(crow_id, LOD::Low);
        }
        map
    }
}

impl fmt::Debug for GridItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(id: {})", self.id)
    }
}

impl fmt::Debug for Grid{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Grid {{ size: {}, cell_size: {}, grid: [", self.size, self.cell_size)?;
        
        for x in &self.grid {
            write!(f, "[")?;
            for y in x {
                write!(f, "[")?;
                for z in y {
                    write!(f, "{{ crows: {:?} }},", z.crows)?;
                }
                write!(f, "],")?;
            }
            write!(f, "],")?;
        }

        write!(f, "] }}")
    }
}