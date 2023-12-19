// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/compute.wgsl

struct Params {
    speed: f32,
    seperationDistance : f32,
    alignmentDistance : f32,
    cohesionDistance : f32,
    seperationScale : f32,
    alignmentScale : f32,
    cohesionScale : f32
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


@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let total_boids = arrayLength(&boids_src);
    let index = invocation_id.x;

    if (index >= total_boids) {
        return;
    }

    var vPos = boids_src[index].pos; // Boid Position
    var vVel = boids_src[index].vel; // Boid Velocity

    var total_seperation : vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total_alignment: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total_cohesion : vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var alignmentCount : i32 = 0;
    var cohesionCount: i32 = 0;

    var pos: vec4<f32>;
    var vel: vec4<f32>;

    var i: u32 = 0u;

    loop {
        if (i >= total_boids) {
            break;
        }
        if (i == index) {
            continue;
        }

        pos = boids_src[i].pos;
        vel = boids_src[i].vel;

        let dst = distance(pos, vPos);

        if (dst < params.seperationDistance) {
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
        vPos.x = 1.0;
    }
    if (vPos.x > 1.0) {
        vPos.x = -1.0 + (vPos.x - 1.0);
        vPos.x = -1.0;
    }
    if (vPos.y < -1.0) {
        vPos.y = 1.0 + (1.0 + vPos.y);
        vPos.y = 1.0;
    }
    if (vPos.y > 1.0) {
        vPos.y = -1.0 + (vPos.y - 1.0);
        vPos.y = -1.0;
    }
    if (vPos.z < -1.0) {
        vPos.z = 1.0 + (vPos.z + 1.0);
        vPos.z = 1.0;
    }
    if (vPos.z > 1.0) {
        //vPos.z = -1.0 + (vPos.z - 1.0);
        vPos.z = -1.0;
    }

    // Write back
    boids_dst[index].pos = vPos;
    boids_dst[index].vel = vVel;
    
}
