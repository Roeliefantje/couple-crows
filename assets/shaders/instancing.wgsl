// Instancing shader from the bevy example:
// https://github.com/bevyengine/bevy/blob/release-0.12.1/assets/shaders/instancing.wgsl
// Modified to be able to orient the birds towards the velocity
// Also has a commented line that would modify the color of the crow based on the velocity.
#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_vel: vec4<f32>,
    @location(5) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) vel: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    //Flip the velocity, so the crows front is at the front.
    let i_vel = vertex.i_vel * -1.0f;
    let yaw = atan2(i_vel.x, i_vel.z);
    let pitch = atan2(i_vel.y, length(i_vel.xz));

    // Create rotation matrix
    let rotation = mat3x3<f32>(
        vec3<f32>(cos(yaw), 0.0, -sin(yaw)),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(sin(yaw), 0.0, cos(yaw))
    ) * mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, cos(pitch), -sin(pitch)),
        vec3<f32>(0.0, sin(pitch), cos(pitch))
    );

    // Rotate the position
    let pos = rotation * vertex.position;
    
    let position = pos * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    var out: VertexOutput;



    // NOTE: Passing 0 as the instance_index to get_model_matrix() is a hack
    // for this example as the instance_index builtin would map to the wrong
    // index in the Mesh array. This index could be passed in via another
    // uniform instead but it's unnecessary for the example.
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(0u),
        vec4<f32>(position, 1.0)
    );
    out.color = vertex.i_color;
    out.vel = i_vel;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

    let vel_n = normalize(in.vel);

    //return vec4<f32>(abs(vel_n.x), abs(vel_n.y), abs(vel_n.z), 0.3);
    return in.color;
}
