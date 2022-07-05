use bytemuck;
use crevice::std140::AsStd140;
use wgpu::util::DeviceExt;

use super::builder;

use wrach_physics_shaders as physics;

const NUM_PARTICLES: u32 = physics::world::NUM_PARTICLES as u32;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 128;

pub struct Pipeline {
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub position_buffers: Vec<wgpu::Buffer>,
    pub velocity_buffers: Vec<wgpu::Buffer>,
    pub propogations_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub grid_buffer: wgpu::Buffer,
    pub pre_compute_pipeline: wgpu::ComputePipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub post_compute_pipeline: wgpu::ComputePipeline,
    pub work_group_count: u32,
    pub bind_group: usize,
}

impl Pipeline {
    pub fn init(device: &wgpu::Device) -> Self {
        let mut builder = builder::Builder::new(device);

        let shader_module = builder::Builder::shader(device, "shaders/physics");

        let params_buffer = builder.params_buffer(physics::compute::Params::default());

        let compute_bind_group_layout = builder.compute_bind_group_layout();
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<physics::compute::Std140Params>() as u32,
                }],
            });

        let pre_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Pre-compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader_module,
                entry_point: "pre_main_cs",
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader_module,
            entry_point: "main_cs",
        });

        let post_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Post compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader_module,
                entry_point: "post_main_cs",
            });

        let (initial_position_data, initial_velocity_data) = builder.init_particle_buffer();

        let mut bind_groups = Vec::<wgpu::BindGroup>::new();

        let mut position_buffers = Vec::<wgpu::Buffer>::new();
        for i in 0..2 {
            position_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Position Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_position_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        let mut velocity_buffers = Vec::<wgpu::Buffer>::new();
        for i in 0..2 {
            velocity_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Velocity Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_velocity_data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        let propogation = physics::particle::ParticlePropogation::default();
        let propogations: Vec<physics::particle::Std140ParticlePropogation> =
            vec![propogation.as_std140(); physics::world::NUM_PARTICLES];
        let propogations_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Propogations buffer")),
            contents: bytemuck::cast_slice(&propogations),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let grid: physics::neighbours::PixelMapBasic = [0; physics::neighbours::GRID_SIZE];
        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Pixel Map")),
            contents: bytemuck::cast_slice(&grid),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let neighbourhood_ids: physics::neighbours::NeighbourhoodIDsBuffer =
            [[0; physics::neighbours::MAX_NEIGHBOURS_WITH_COUNT]; physics::world::NUM_PARTICLES];
        let neighbourhood_ids_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Neighbourhood IDs")),
                contents: bytemuck::cast_slice(&neighbourhood_ids),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        for i in 0..2 {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: position_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: position_buffers[(i + 1) % 2].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: velocity_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: velocity_buffers[(i + 1) % 2].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: propogations_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: grid_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: neighbourhood_ids_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        Self {
            bind_groups,
            position_buffers,
            velocity_buffers,
            propogations_buffer,
            params_buffer,
            grid_buffer,
            pre_compute_pipeline,
            compute_pipeline,
            post_compute_pipeline,
            work_group_count,
            bind_group: 0,
        }
    }

    pub fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    pub fn required_limits() -> wgpu::Limits {
        wgpu::Limits {
            max_push_constant_size: 128,
            max_storage_buffers_per_shader_stage: 7,
            ..wgpu::Limits::downlevel_defaults()
        }
    }

    pub fn required_features() -> wgpu::Features {
        wgpu::Features::CLEAR_COMMANDS | wgpu::Features::PUSH_CONSTANTS
    }

    pub fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
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
}
