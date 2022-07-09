use crevice::std140::AsStd140;

use physics::wrach_glam::glam::vec2;
use wgpu::{util::DeviceExt, ShaderModule};
use wrach_physics_shaders as physics;

pub struct Builder<'a> {
    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self { device }
    }

    pub fn shader(device: &wgpu::Device, path: &str) -> ShaderModule {
        let shader_binary = rust_gpu_compiler::build(path);
        let spirv = wgpu::util::make_spirv(&shader_binary);
        device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: spirv,
        })
    }

    pub fn params_buffer(&mut self, params: physics::compute::Params) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Parameters buffer"),
                contents: bytemuck::bytes_of(&params.as_std140()),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            })
    }

    pub fn compute_bind_group_layout(&mut self) -> wgpu::BindGroupLayout {
        let sizeof_params = std::mem::size_of::<physics::compute::Params>() as u64;
        let sizeof_vec2 = std::mem::size_of::<physics::wrach_glam::glam::Vec2>();
        let sizeof_vec2s = (sizeof_vec2 * physics::world::NUM_PARTICLES as usize) as u64;
        let sizeof_propogation =
            std::mem::size_of::<physics::particle::Std140ParticlePropogation>() as u64;
        let sizeof_propogations = sizeof_propogation * physics::world::NUM_PARTICLES as u64;
        let neighbourhood_ids_size = (std::mem::size_of::<physics::neighbours::NeighbourhoodIDs>()
            * physics::world::NUM_PARTICLES) as u64;

        let bind_groups = [
            // Params
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_params),
                },
                count: None,
            },
            // Positions source
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    // TODO read_only should be true
                    // It causes this error:
                    // shader global ResourceBinding { group: 0, binding: 1 } is not available in the layout pipeline layout
                    // storage class Storage { access: LOAD } doesn't match the shader Storage { access: LOAD | STORE }
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_vec2s),
                },
                count: None,
            },
            // Positions destination
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_vec2s),
                },
                count: None,
            },
            // Velocities source
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_vec2s),
                },
                count: None,
            },
            // Velocities destination
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_vec2s),
                },
                count: None,
            },
            // Propogations
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_propogations),
                },
                count: None,
            },
            // Pixel grid map
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                        physics::neighbours::PixelMapBasic,
                    >() as u64),
                },
                count: None,
            },
            // Neighbourhood IDs
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(neighbourhood_ids_size),
                },
                count: None,
            },
        ];

        let bind_group_layout = wgpu::BindGroupLayoutDescriptor {
            entries: &bind_groups,
            label: None,
        };

        self.device.create_bind_group_layout(&bind_group_layout)
    }

    pub fn init_particle_buffer(
        &mut self,
    ) -> (
        Vec<physics::particle::ParticlePosition>,
        Vec<physics::particle::ParticleVelocity>,
        physics::neighbours::PixelMapBasic,
    ) {
        let mut initial_position_data: Vec<physics::particle::ParticlePosition> = Vec::new();
        let mut initial_velocity_data: Vec<physics::particle::ParticleVelocity> = Vec::new();
        let mut initial_grid_data: physics::neighbours::PixelMapBasic =
            [0; physics::neighbours::GRID_SIZE];
        let mut count = 0;
        let x_min = -0.0;
        let mut x = x_min;
        let mut y = -0.4;
        let spacing = 2.0 * physics::particle::PARTICLE_RADIUS;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter: f32 = 0.01;

        loop {
            loop {
                let position = vec2(x, y);
                initial_position_data.push(position);
                initial_velocity_data.push(vec2(
                    rng.gen_range(-jitter, jitter),
                    rng.gen_range(-jitter, jitter),
                ));
                physics::neighbours::NeighbouringParticles::place_particle_in_pixel(
                    count as physics::particle::ParticleID,
                    position,
                    &mut initial_grid_data,
                );
                count += 1;
                if count > physics::world::NUM_PARTICLES {
                    break;
                }
                x += spacing;
                if x > 0.9 {
                    break;
                }
            }
            y += spacing;
            x = x_min;
            if count > physics::world::NUM_PARTICLES {
                break;
            }
        }
        (
            initial_position_data,
            initial_velocity_data,
            initial_grid_data,
        )
    }
}
