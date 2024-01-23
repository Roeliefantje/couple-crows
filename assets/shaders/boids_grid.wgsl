// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/compute.wgsl

struct Params {
    speed: f32,
    seperationDistance : f32,
    alignmentDistance : f32,
    cohesionDistance : f32,
    seperationScale : f32,
    alignmentScale : f32,
    cohesionScale : f32,
    grid_size : f32,
    cell_size : f32,
}

struct Boid {
    pos: vec4<f32>,
    vel: vec4<f32>
}

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<uniform> delta_time: f32;

@group(0) @binding(2)
var<storage> boids_src: array<Boid>;
@group(0) @binding(3)
var<storage, read_write> boids_dst: array<Boid>;
@group(0) @binding(4)
var<storage> amount_of_crows_vec: array<u32>;
@group(0) @binding(5)
var<storage> crow_idxs: array<u32>;

fn wrap_around(coord: i32, max_value: i32) -> i32 {
    if (coord < 0) {
        return max_value - 1;
    } else if (coord >= max_value) {
        return 0;
    } else {
        return coord;
    }
}

@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let grid_size_x: i32 = 10; 
    let grid_size_y: i32 = 10; 
    let grid_size_z: i32 = 10; 

    //let total_grids = arrayLength(&amount_of_crows_vec);
    //let total_indices = arrayLength(&crow_idxs);

    let total_boids = arrayLength(&boids_src);
    let index = invocation_id.x;

    if (index >= total_boids) {
        return;
    }

    var vPos = boids_src[index].pos; // Boid Position
    var vVel = boids_src[index].vel; // Boid Velocity

    var grid_x = u32((vPos.x / params.cell_size) + (params.grid_size * params.cell_size * 0.5)) % u32(params.grid_size);
    var grid_y = u32((vPos.y / params.cell_size) + (params.grid_size * params.cell_size * 0.5)) % u32(params.grid_size);
    var grid_z = u32((vPos.z / params.cell_size) + (params.grid_size * params.cell_size * 0.5)) % u32(params.grid_size);

    var grid_idx: u32 = grid_x * u32(params.grid_size) * u32(params.grid_size) + grid_y * u32(params.grid_size) + grid_z;
    var start_idx: u32 = 0u;
    var end_idx: u32 = amount_of_crows_vec[grid_idx]; 
    if (grid_idx > 0u) {
        start_idx = amount_of_crows_vec[grid_idx - 1u];
    }

    var delta_x: i32 = -1;
    while (delta_x <= 1) {
        var delta_y: i32 = -1;
        while (delta_y <= 1) {
            var delta_z: i32 = -1;
            while (delta_z <= 1) {
                if (delta_x != 0 || delta_y != 0 || delta_z != 0) {
                    let neighbor_grid_x = wrap_around(i32(grid_x) + delta_x, grid_size_x);
                    let neighbor_grid_y = wrap_around(i32(grid_y) + delta_y, grid_size_y);
                    let neighbor_grid_z = wrap_around(i32(grid_z) + delta_z, grid_size_z);

                    let neighbor_grid_index = neighbor_grid_x +
                                              neighbor_grid_y * grid_size_x +
                                              neighbor_grid_z * grid_size_x * grid_size_y;
                }
                delta_z = delta_z + 1;
            }
            delta_y = delta_y + 1;
        }
        delta_x = delta_x + 1;
    }



    var total_seperation : vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total_alignment: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total_cohesion : vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var alignmentCount : i32 = 0;
    var cohesionCount: i32 = 0;

    var pos: vec4<f32>;
    var vel: vec4<f32>;

    var i: u32 = start_idx;

    loop {
        if (i >= end_idx) {
            break;
        }
        if (i == index) {
            continue;
        }

        pos = boids_src[crow_idxs[i]].pos;
        vel = boids_src[crow_idxs[i]].vel;

        let dst = distance(pos, vPos);

        if (0.0 < dst && dst < params.seperationDistance) {
            total_seperation += normalize(pos - vPos) * f32(-1) / dst;
        }
        if (dst < params.alignmentDistance) {
            total_alignment += vel;
            alignmentCount += 1;
        }
        if (dst < params.cohesionDistance) {
            total_cohesion += pos;
            cohesionCount += 1;
        }

        continuing {
            i = i + 1u;
        }
    }

    if (alignmentCount > 0) {
        total_alignment /= f32(alignmentCount);
    }

    if (cohesionCount > 0) {
        //Average position of the nearby crows
        total_cohesion /= f32(cohesionCount);
        //Turn that into a velocity vector from the Boid we are calculating.
        total_cohesion -= vPos;
    }

    vVel = vVel + (total_seperation * params.seperationScale) +
        (total_alignment * params.alignmentScale) +
        (total_cohesion * params.cohesionScale);

    // clamp velocity for a more pleasing simulation
    vVel = normalize(vVel) * params.speed;

    // kinematic update
    vPos = vPos + (vVel * delta_time);
    

    // Wrap around boundary
    if (vPos.x < -1.0) {
        vPos.x = 1.0 + (1.0 + vPos.x);
    }
    if (vPos.x > 1.0) {
        vPos.x = -1.0 + (vPos.x - 1.0);
    }
    if (vPos.y < -1.0) {
        vPos.y = 1.0 + (1.0 + vPos.y);
    }
    if (vPos.y > 1.0) {
        vPos.y = -1.0 + (vPos.y - 1.0);
    }
    if (vPos.z < -1.0) {
        vPos.z = 1.0 + (vPos.z + 1.0);
    }
    if (vPos.z > 1.0) {
        vPos.z = -1.0 + (vPos.z - 1.0);
    }

    // Write back
    boids_dst[index].pos = vPos;
    boids_dst[index].vel = vVel;
    
}
