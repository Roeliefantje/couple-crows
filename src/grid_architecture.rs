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
        let x = self.cooridnate_to_grid_coordinate(transform.translation.x);
        let y = self.cooridnate_to_grid_coordinate(transform.translation.y);
        let z = self.cooridnate_to_grid_coordinate(transform.translation.z);
        self.grid[x][y][z].crows.push(*transform);
    }

    fn cooridnate_to_grid_coordinate (&self, coordinate: f32) -> usize {
         ((coordinate.abs() + if coordinate < 0.0 {0.0} else {self.size as f32 * self.cell_size / 2.0}) / self.cell_size) as usize % self.size
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