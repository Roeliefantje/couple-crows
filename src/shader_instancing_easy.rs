use bevy_app_compute::*;
use bevy::prelude::*;
use bytemuck::Zeroable;
use rand::distributions::{Distribution, Uniform};

pub struct BoidsComputePlugin;

impl Plugin for BoidsComputePlugin {
    fn build(&self, app: &mut App){        
        app.add_plugins(ExtractResourcePlugin::<BoidsImage>::default());
        app.add_plugin(AppComputePlugin);
        app.add_plugin(AppComputeWorkerPlugin::<BoidsComputeWorker>::default());
    }

    fn finish(&self, app: &mut App){
        app.add_system(my_system);
    }
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Boid {
    pos: Vec3,
    vel: Vec3
}

const NUM_BOIDS: u32 = 500; 

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct BoidsShader;

impl ComputeShader for BoidsShader {
    fn shader() -> ShaderRef {
        "shaders/boids.wgsl".into()
    }
}

#[derive(Resource)]
struct BoidsComputeWorker;


impl ComputeWorker for BoidsComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {

        let mut initial_boids_data = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::thread_rng();
        let unif = Uniform::new_inclusive(-1, 1);

        for _ in 0..NUM_BOIDS {
            initial_boids_data.push(Boid {
                pos: Vec3::new(unif.sample(&mut rng), unif.sample(&mut rng), unif.sample(&mut rng)),
                vel: Vec3::new(unif.sample(&mut rng), unif.sample(&mut rng), unif.sample(&mut rng)).normalize(),
            })
        }
        

        let worker = AppComputeWorkerBuilder::new(world)
            .add_staging("boids_src", &initial_boids_data)
            .add_staging("boids_dst", &initial_boids_data)
            .add_pass::<BoidsShader>([NUM_BOIDS, 1, 1], &["boids_src", "boids_dst"])
            .add_swap("boids_src", "boids_dst")
            .build();

        worker
    }
}


fn my_system(
    mut compute_worker: ResMut<AppComputeWorker<BoidsComputeWorker>>
) {
    if !compute_worker.available() {
        return;
    };

    let boids = worker.read_vec::<Boid>("boids_dst");

    println!("got {:?}", boids)
}
