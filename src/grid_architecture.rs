use std::fmt;

use bevy::{math::*, prelude::*};

fn main() {
    // Your main code here
}

#[derive(Resource)]

//Grid struct to store all crows in a grid, Grid is always centered at 0,0,0
pub struct Grid {
    pub grid: Vec<Vec<Vec<GridCell>>>,
    pub size: usize,
    pub cell_size: f32,
}

pub struct GridCell {
    crows: Vec<Transform>,
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
    pub fn add_with_transform (&mut self, transform: &Transform) {
        //Convert the possible negative coordinates to positive 
        //meaning that negative coordinates are between 0 and size/2 
        //and positive coordinates are between size/2 and size 
        let x = self.cooridnate_to_grid_coordinate(transform.translation.x) % self.size;
        let y = self.cooridnate_to_grid_coordinate(transform.translation.y) % self.size;
        let z = self.cooridnate_to_grid_coordinate(transform.translation.z) % self.size;
        self.grid[x][y][z].crows.push(*transform);
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
                        if crow.translation.distance(point) < radius {
                            crows.push(crow);
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

    fn gather_crows_from_corner<'a> (&'a self, crows: &mut Vec<&'a Transform>, c_x: usize, c_y: usize, c_z: usize, camera_transform: &Transform){
        let sign_x = if camera_transform.translation.x < 0.0 { -1.0 } else { 1.0 }; 
        let sign_y = if camera_transform.translation.y < 0.0 { -1.0 } else { 1.0 }; 
        let sign_z = if camera_transform.translation.z < 0.0 { -1.0 } else { 1.0 }; 
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

    pub fn get_crows_in_lod_change_area(&self, camera_transform: &Transform, lod_dist: f32) -> (Vec<&Transform>, Vec<&Transform>) {
        let mut close_crows = Vec::new();
        let mut far_crows = Vec::new();
        let sign_x = if camera_transform.translation.x < 0.0 { -1.0 } else { 1.0 }; 
        let sign_y = if camera_transform.translation.y < 0.0 { -1.0 } else { 1.0 }; 
        let sign_z = if camera_transform.translation.z < 0.0 { -1.0 } else { 1.0 }; 
        let (c_x, c_y, c_z) = self.get_lod_corner_cell(camera_transform, lod_dist);
        self.gather_crows_from_corner(&mut close_crows, c_x, c_y, c_z, camera_transform);
        self.gather_crows_from_corner(
            &mut far_crows, 
            (c_x as i32 + 1 * (-sign_x as i32)).max(0).min(self.size as i32 - 1) as usize, 
            (c_y as i32 + 1 * (-sign_y as i32)).max(0).min(self.size as i32 - 1) as usize, 
            (c_z as i32 + 1 * (-sign_z as i32)).max(0).min(self.size as i32 - 1) as usize, 
            camera_transform
        );
    
        info!("near layer: {}, far layer: {}", close_crows.len(), far_crows.len());
        (close_crows, far_crows)
    }    
}

impl fmt::Debug for Grid {
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