use crevice::std140::AsStd140;
use shaders::wrach_glam::glam::{vec2, vec4};
use wgpu::util::DeviceExt;

pub struct Builder<'a> {
    config: &'a wgpu::SurfaceConfiguration,
    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> {
    pub fn new(config: &'a wgpu::SurfaceConfiguration, device: &'a wgpu::Device) -> Self {
        Self { config, device }
    }

    pub fn params_buffer(&mut self, params: shaders::compute::Params) -> wgpu::Buffer {
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
        let sizeof_particle = std::mem::size_of::<shaders::particle::Std140ParticleBasic>();
        let sizeof_particles = (sizeof_particle * shaders::world::NUM_PARTICLES as usize) as u64;
        let sizeof_params = std::mem::size_of::<shaders::compute::Params>() as u64;

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
                        shaders::neighbours::PixelMapBasic,
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

    pub fn render_pipeline(&mut self, shader_module: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let particle_array_stride =
            std::mem::size_of::<shaders::particle::Std140ParticleBasic>() as u64;

        let fragment = wgpu::FragmentState {
            module: shader_module,
            entry_point: "main_fs",
            targets: &[self.config.format.into()],
        };

        let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: particle_array_stride,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x4, 1 => Float32x2, 2 => Float32x2, 3 => Float32x2
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 1 * 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![4 => Float32x2],
                    },
                ],
            },
            fragment: Some(fragment),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        };

        self.device.create_render_pipeline(&pipeline_descriptor)
    }

    pub fn init_vertices_buffer(&mut self) -> wgpu::Buffer {
        // A square made of 2 triangles
        #[rustfmt::skip]
        let vertex_buffer_data: Vec<f32> = [
            // First triangle ----------------------
            -1, -1, -1, 1, 1, 1,
            // Second triangle ----------------------
            -1, -1, 1, 1, 1, -1,
        ]
        .iter()
        .map(|x| 0.5 * shaders::particle::PIXEL_SIZE * (*x as f32))
        .collect();
        let mut square = [0.0; 12];
        for i in 0..12 {
            square[i] = vertex_buffer_data[i];
        }
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::bytes_of(&square),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn init_particle_buffer(&mut self) -> Vec<shaders::particle::Std140ParticleBasic> {
        let mut initial_particle_data: Vec<shaders::particle::Std140ParticleBasic> = Vec::new();
        let mut count = 0;
        let x_min = -0.0;
        let mut x = x_min;
        let mut y = -0.4;
        let spacing = 2.0 * shaders::particle::PARTICLE_RADIUS;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter: f32 = 0.01;

        loop {
            loop {
                let position = vec2(x, y); // * (1.0 + rng.gen_range(-jitter, jitter));
                let particle = shaders::particle::ParticleBasic {
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
                if count > shaders::world::NUM_PARTICLES {
                    break;
                }
                x += spacing;
                if x > 0.9 {
                    break;
                }
            }
            y += spacing;
            x = x_min;
            if count > shaders::world::NUM_PARTICLES {
                break;
            }
        }
        initial_particle_data
    }
}
