use crevice::std140::AsStd140;

use physics::wrach_glam::glam::{vec2, vec4};
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
        let sizeof_particle = std::mem::size_of::<physics::particle::Std140ParticleBasic>();
        let sizeof_particles = (sizeof_particle * physics::world::NUM_PARTICLES as usize) as u64;
        let sizeof_params = std::mem::size_of::<physics::compute::Params>() as u64;

        let bind_groups = [
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
                    min_binding_size: wgpu::BufferSize::new(sizeof_particles),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(sizeof_particles),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
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
        ];

        let bind_group_layout = wgpu::BindGroupLayoutDescriptor {
            entries: &bind_groups,
            label: None,
        };

        self.device.create_bind_group_layout(&bind_group_layout)
    }

    pub fn init_particle_buffer(&mut self) -> Vec<physics::particle::Std140ParticleBasic> {
        let mut initial_particle_data: Vec<physics::particle::Std140ParticleBasic> = Vec::new();
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
                let position = vec2(x, y); // * (1.0 + rng.gen_range(-jitter, jitter));
                let particle = physics::particle::ParticleBasic {
                    color: vec4(1.0, 1.0, 1.0, 1.0),
                    position,
                    previous: position,
                    pre_fluid_position: position,
                    velocity: vec2(
                        rng.gen_range(-jitter, jitter),
                        rng.gen_range(-jitter, jitter),
                    ),
                    ..Default::default()
                };
                initial_particle_data.push(particle.as_std140());
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
        initial_particle_data
    }
}
