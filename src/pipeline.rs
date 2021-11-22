use bytemuck;
use crevice::std140::AsStd140;
use wgpu::util::DeviceExt;

use shaders::particle::ParticleBasic;
use shaders::particle::ParticleGridStartID;
use shaders::wrach_glam::glam::{vec2, vec4};

const NUM_PARTICLES: u32 = shaders::world::NUM_PARTICLES as u32;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 64;

pub struct Pipeline {
    particle_bind_groups: Vec<wgpu::BindGroup>,
    particle_buffers: Vec<wgpu::Buffer>,
    vertices_buffer: wgpu::Buffer,
    grid_buffer: wgpu::Buffer,
    pre_compute_pipeline: wgpu::ComputePipeline,
    predict_compute_pipeline: wgpu::ComputePipeline,
    compute_pipeline: wgpu::ComputePipeline,
    post_compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    work_group_count: u32,
    frame_num: usize,
    bind_group: usize,
    is_paused: bool,
}

impl Pipeline {
    pub fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    pub fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_defaults()
    }

    pub fn required_features() -> wgpu::Features {
        wgpu::Features::CLEAR_COMMANDS
    }

    pub fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn init(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let shader_binary = wgpu::include_spirv!(env!("shaders.spv"));
        let shader_module = device.create_shader_module(&shader_binary);

        let sim_param_data = [
            0.0f32, // stage
            0.0,    // stage
            0.0,    // stage
            0.0,    // stage
            0.0,    // stage
        ]
        .to_vec();
        let sim_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&sim_param_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sizeof_particle = std::mem::size_of::<shaders::particle::Std140ParticleBasic>();
        let sizeof_particle_buffer = (sizeof_particle * NUM_PARTICLES as usize) as u64;

        // create compute bind layout group and compute pipeline layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (sim_param_data.len() * std::mem::size_of::<f32>()) as _,
                            ),
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
                            min_binding_size: wgpu::BufferSize::new(sizeof_particle_buffer),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(sizeof_particle_buffer),
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
                                shaders::neighbours::GridBasic,
                            >()
                                as u64),
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        // create render pipeline with empty bind group layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<shaders::particle::Std140ParticleBasic>()
                            as u64,
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
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "main_fs",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        // Create pre-compute pipeline
        let pre_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Pre-compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader_module,
                entry_point: "pre_main_cs",
            });

        // Create compute pipeline
        let predict_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Predict compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader_module,
                entry_point: "predict_main_cs",
            });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader_module,
            entry_point: "main_cs",
        });

        // Create post-compute pipeline
        let post_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Post-compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader_module,
                entry_point: "post_main_cs",
            });

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
        let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&square),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Buffer for all particles data of type [(posx,posy,velx,vely),...]
        let mut initial_particle_data: Vec<shaders::particle::Std140ParticleBasic> = Vec::new();
        let mut count = 0;
        let x_min = -0.0;
        let mut x = x_min;
        let mut y = -0.4;
        let spacing = 2.0 * shaders::particle::PARTICLE_RADIUS;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = 0.01;

        loop {
            loop {
                let position = vec2(x, y); // * (1.0 + rng.gen_range(-jitter, jitter));
                let mut particle = ParticleBasic {
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
                particle.grid_start_index = particle.grid_start_index();
                initial_particle_data.push(particle.as_std140());
                count += 1;
                if count > NUM_PARTICLES {
                    break;
                }
                x += spacing;
                if x > 0.9 {
                    break;
                }
            }
            y += spacing;
            x = x_min;
            if count > NUM_PARTICLES {
                break;
            }
        }

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_particle_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        let grid: shaders::neighbours::GridBasic =
            [0; shaders::neighbours::TOTAL_GRID_STORAGE_SIZE];
        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Pixel Map")),
            contents: bytemuck::cast_slice(&grid),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sim_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: grid_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        Pipeline {
            particle_bind_groups,
            particle_buffers,
            vertices_buffer,
            grid_buffer,
            pre_compute_pipeline,
            predict_compute_pipeline,
            compute_pipeline,
            post_compute_pipeline,
            render_pipeline,
            work_group_count,
            frame_num: 0,
            bind_group: 0,
            is_paused: false,
        }
    }

    /// update is called for any WindowEvent not handled by the framework
    pub fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    /// resize is called on WindowEvent::Resized events
    pub fn resize(
        &mut self,
        _sc_desc: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        //empty
    }

    /// render is called each frame, dispatching compute groups proportional
    ///   a TriangleList draw call for all NUM_PARTICLES at 3 vertices each
    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        // create render pass descriptor and its color attachments
        let color_attachments = [wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        }];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        // get command encoder
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        command_encoder.clear_buffer(&self.grid_buffer, 0, None);

        command_encoder.push_debug_group("compute");
        if !self.is_paused {
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            // neighbours
            if self.bind_group == 0 {
                self.bind_group = 1;
            } else {
                self.bind_group = 0;
            }
            cpass.set_bind_group(0, &self.particle_bind_groups[self.bind_group], &[]);
            cpass.set_pipeline(&self.pre_compute_pipeline);
            cpass.dispatch(
                (shaders::neighbours::GRID_COUNT as f32 / 64.0) as u32 + 1,
                1,
                1,
            );

            for _ in 0..shaders::particle::DEFAULT_NUM_SOLVER_SUBSTEPS {
                // predict pass
                if self.bind_group == 0 {
                    self.bind_group = 1;
                } else {
                    self.bind_group = 0;
                }
                cpass.set_bind_group(0, &self.particle_bind_groups[self.bind_group], &[]);
                cpass.set_pipeline(&self.predict_compute_pipeline);
                cpass.dispatch(self.work_group_count, 1, 1);

                // compute pass
                if self.bind_group == 0 {
                    self.bind_group = 1;
                } else {
                    self.bind_group = 0;
                }
                cpass.set_bind_group(0, &self.particle_bind_groups[self.bind_group], &[]);
                cpass.set_pipeline(&self.compute_pipeline);
                cpass.dispatch(self.work_group_count, 1, 1);

                // post compute
                if self.bind_group == 0 {
                    self.bind_group = 1;
                } else {
                    self.bind_group = 0;
                }
                cpass.set_bind_group(0, &self.particle_bind_groups[self.bind_group], &[]);
                cpass.set_pipeline(&self.post_compute_pipeline);
                cpass.dispatch(self.work_group_count, 1, 1);
            }
        }
        command_encoder.pop_debug_group();

        command_encoder.push_debug_group("render pixels");
        {
            if self.bind_group == 0 {
                self.bind_group = 1;
            } else {
                self.bind_group = 0;
            }
            // render pass
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pipeline);
            // render dst particles
            rpass.set_vertex_buffer(0, self.particle_buffers[self.bind_group].slice(..));
            // the three instance-local vertices
            rpass.set_vertex_buffer(1, self.vertices_buffer.slice(..));
            rpass.draw(0..6, 0..NUM_PARTICLES);
        }
        command_encoder.pop_debug_group();

        // update frame count
        self.frame_num += 1;

        // done
        queue.submit(Some(command_encoder.finish()));
    }
}
