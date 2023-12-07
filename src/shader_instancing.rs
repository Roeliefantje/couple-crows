//Plugins are used in order to organize your code basically, allows you to add plugins to main
//without having to add them in main.
//Maybe we can store all the positions of the boids in a Texture, and then use that texture to
//calculate a new Texture with the new positions.

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
};

use std::borrow::Cow;


fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0,0,0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);
    
    commands.insert_resource(BoidsImage(image));
}




pub struct BoidsComputePlugin;

const CROWS: u32 = 10000;
const WORKGROUP_SIZE: u32 = 16;

impl Plugin for BoidsComputePlugin {
    fn build(&self, app: &mut App){        
        app.add_plugins(ExtractResourcePlugin::<BoidsImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSet::PrepareBindGroups)
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("boids", BoidsNode::default());
        render_graph.add_node_edge(
            "boids",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }

    fn finish(&self, app: &mut App){
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<BoidsPipeline>();
        app.add_systems(Startup, setup);
    }
}

#[derive(Resource, Clone, Deref, ExtractResource)]
struct BoidsImage(Handle<Image>);

#[derive(Resource)]
struct BoidsImageBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<BoidsPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    boids_image: Res<BoidsImage>,
    render_device: Res<RenderDevice>,
) {
    let view = gpu_images.get(&boids_image.0).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::single(&view.texture_view),
    );
}

#[derive(Resource)]
pub struct BoidsPipeline {
    texture_bind_group_layout: BindGroupLayout,
    main_pipeline: CachedComputePipelineId
}

impl FromWorld for BoidsPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("boids_grou_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        format: TextureFormat::Rgba8Unorm,
                        access: StorageTextureAccess::ReadWrite,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }

            ],
        });
        

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/boids.wgsl");

        let main_pipeline = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor{
                label: None,
                layout: vec![texture_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("main"),

            });

        Self {
            texture_bind_group_layout,
            main_pipeline,
        }

    }
}

#[derive(Default)]
struct BoidsNode;

impl render_graph::Node for BoidsNode {
    fn run (
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<BoidsImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<BoidsPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        let main_pipeline = pipeline_cache.get_compute_pipeline(pipeline.main_pipeline).unwrap();

        pass.set_pipeline(main_pipeline);
        pass.dispatch_workgroups(CROWS / WORKGROUP_SIZE, 0, 0);

        Ok(())
    }
}

