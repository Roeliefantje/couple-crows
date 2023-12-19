//! Example showing how to calculate boids data from compute shaders
//! For now they are stupid and just fly straight, need to fix this later on.
//! Reimplementation of https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/main.rs

//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

//If ID3D12CommandQueue error, in terminal on windows: set WGPU_BACKEND=VULKAN
use bevy::{
    core::Pod,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, NoFrustumCulling},
        Render, RenderApp, RenderSet,
    },
    diagnostic::{
        FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin
    }
};

use bevy_app_compute::prelude::*;
use bytemuck::Zeroable;

use rand::distributions::{Distribution, Uniform};

const NUM_BOIDS: u32 = 10000;

// Boid struct that gets transfered over to the compute shader which includes all the information needed for the computation.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Boid {
    pos: Vec4,
    vel: Vec4,
}

// Params we can set in order to change the behaviour of the compute shader.
#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Params {
    speed: f32,
    seperation_distance: f32,
    alignment_distance: f32,
    cohesion_distance: f32,
    seperation_scale: f32,
    alignment_scale: f32,
    cohesion_scale: f32,
}

//Identifier in order to link the boids data to a texture.
#[derive(Component)]
struct BoidEntity(pub usize);

//The bundle that gets spawned in with the texture / mesh of the boid
#[derive(Bundle)]
struct CrowBundle {
    pbr: PbrBundle,
    boid_entity: BoidEntity,
}

//Main, adding some useful plugins that allow for some easy logging.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(AppComputePlugin)
        .add_plugins(AppComputeWorkerPlugin::<BoidWorker>::default())
        .add_plugins(CustomMaterialPlugin)
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_systems(Startup, setup)
        .add_systems(Update, move_entities_instance)
        .run()
}

//The main setup of the program. Basically just creates all the (World) boids and the setup for the camera.
//So it does not create the position etc, but it it creates the mesh for them and has a unique idx we use to transfer the boids data.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {


    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    let mut all_boids: Vec<CrowBundle> = Vec::with_capacity(NUM_BOIDS as usize);
    let boid_mesh = meshes.add(Mesh::from(shape::Cube {size: 0.1})); 
    let boid_material = materials.add(Color::ANTIQUE_WHITE.into());

    // for i in 0..NUM_BOIDS {
    //     let crow = CrowBundle {
    //         pbr: PbrBundle {
    //             mesh: boid_mesh.clone(),
    //             material: boid_material.clone(),
    //             ..Default::default()
    //         },
    //         boid_entity: BoidEntity(i as usize)
    //     };
    //     all_boids.push(crow);
    // }

    // commands.spawn_batch(all_boids);


    commands.spawn((
        meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
        SpatialBundle::INHERITED_IDENTITY,
        InstanceMaterialData(
            (1..=317)
                .flat_map(|x| (1..=315).map(move |y| (x as f32 / 317.0, y as f32 / 317.0)))
                .map(|(x, y)| InstanceData {
                    position: Vec3::new(x * 31.7 - 16.5, y * 31.7 - 16.5, 0.0),
                    scale: 1.0,
                    color: Color::hsla(x * 360., y, 0.5, 1.0).as_rgba_f32(),
                })
                .collect(),
        ),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    ));

}



//This part is to use the boids shader. I am using a plugin I found that helps with a lot of the boilerplate code
//It took quite some time to get this working, but it turns out the float3 does not align properly (which is why we are using vec4 for the boid information)
//https://github.com/Kjolnyr/bevy_app_compute
#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct BoidsShader;

impl ComputeShader for BoidsShader {
    fn shader() -> ShaderRef {
        "shaders/boids.wgsl".into()
    }
}

struct BoidWorker;

//This is what instantiates the compute shader and sets it up to be ran every fram.e
//We use 2 buffers for the boids in order to ensure behaviour is the same every time.
impl ComputeWorker for BoidWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = Params {
            speed: 0.7,
            seperation_distance: 0.03,
            alignment_distance: 0.1,
            cohesion_distance: 0.1,
            seperation_scale: 1.,
            alignment_scale: 1.,
            cohesion_scale: 1.,
        };

        let mut initial_boids_data: Vec<Boid> = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1., 1.);

        for _ in 0..NUM_BOIDS {
            initial_boids_data.push(Boid {
                pos: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0.),
                vel: Vec4::new(
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    unif.sample(&mut rng) as f32,
                    0.)
            });
        }

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_uniform("delta_time", &0.004f32)
            .add_staging("boids_src", &initial_boids_data)
            .add_staging("boids_dst", &initial_boids_data)
            .add_pass::<BoidsShader>(
                [NUM_BOIDS, 1, 1],
                &["params", "delta_time", "boids_src", "boids_dst"],
            )
            .add_swap("boids_src", "boids_dst")
            .build()
    }
}



//This function reads the data from the compute shader and applies them to the crows rendered on the screen.
fn move_entities(
    time: Res<Time>,
    mut worker: ResMut<AppComputeWorker<BoidWorker>>,
    mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>,
) {
    if !worker.ready() {
        return;
    }

    let boids: Vec<Boid> = worker.read_vec::<Boid>("boids_dst");
    worker.write("delta_time", &time.delta_seconds());
    q_boid
        .par_iter_mut()
        .for_each(|(mut transform, boid_entity)| {
            let world_pos = Vec3::new(
                20. * (boids[boid_entity.0].pos.x),
                20. * (boids[boid_entity.0].pos.y),
                20. * (boids[boid_entity.0].pos.z)
            );
            
            transform.look_at(world_pos, Vec3::Y);
            transform.translation = world_pos;

        });
}




//START INSTANCE SHADER

//The plugin that handles all the app adds etc that are necessary for shader instancing.
pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

//This is the data that gets copied over to the GPU for the shader instancing.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct InstanceData {
    position: Vec3,
    scale: f32,
    color: [f32; 4],
}

#[derive(Component, Deref)]
struct InstanceMaterialData(Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}


//The custom queue necessary to allow us to use the shader for instancing of the boids.
//I think this makes sure it uses our Draw Custom Function, which in turn uses the custom pipeline
#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<Entity, With<InstanceMaterialData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for entity in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.get(&entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key = view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            transparent_phase.add(Transparent3d {
                entity,
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder
                    .distance_translation(&mesh_instance.transforms.transform.translation),
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}

//The Buffer instances used for the shader instancing, this is the data that lives on the GPU.
#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}



//The custom Pipeline we use for the instance shader. I think it basically inherits the Mesh Pipeline.
//But uses a different shader, which is shaders/instancing.wgsl
#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}
// I have no clue what this exactly does, it seems to add some information to a vertex?
// It should then be to allow for i_pos_scale and i_color
// It also seems to pass the meshes to bind group 1? I have no clue how this works at all.
impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        // meshes typically live in bind group 2. because we are using bindgroup 1
        // we need to add MESH_BINDGROUP_1 shader def so that the bindings are correctly
        // linked in the shader
        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}


//The drawCustom Type that is used to 
type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

pub struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderMeshInstances>);
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: &'w InstanceBuffer,
        (meshes, render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(mesh_instance) = render_mesh_instances.get(&item.entity()) else {
            return RenderCommandResult::Failure;
        };
        let gpu_mesh = match meshes.into_inner().get(mesh_instance.mesh_asset_id) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}


fn move_entities_instance(
    time: Res<Time>,
    mut worker: ResMut<AppComputeWorker<BoidWorker>>,
    mut query: Query<(Entity, &mut InstanceMaterialData)>,
    //mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>
) {
    if !worker.ready() {
        return;
    }

    let boids: Vec<Boid> = worker.read_vec::<Boid>("boids_dst");
    worker.write("delta_time", &time.delta_seconds());
    for (_, mut instance_data) in &mut query {
        for (index, instance) in instance_data.0
            .iter_mut()
            .enumerate() {

                if index < NUM_BOIDS as usize {
                    instance.position = Vec3::new(
                        20. * (boids[index].pos.x),
                        20. * (boids[index].pos.y),
                        20. * (boids[index].pos.z)
                    );
                }
                
            }


    }
    // q_boid
    //     .par_iter_mut()
    //     .for_each(|(mut transform, boid_entity)| {
    //         let world_pos = Vec3::new(
    //             20. * (boids[boid_entity.0].pos.x),
    //             20. * (boids[boid_entity.0].pos.y),
    //             20. * (boids[boid_entity.0].pos.z)
    //         );
            
    //         transform.look_at(world_pos, Vec3::Y);
    //         transform.translation = world_pos;

    //     });
}


// fn change_instance_pos (
//     mut query: Query<(Entity, &mut InstanceMaterialData)>
// ) {
//     for (_, mut instance_data) in &mut query {
//         for data in &mut instance_data.0 {
//             data.position.x += 0.001;
//         }
//     }
// }