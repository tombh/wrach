use bytemuck;
use wgpu::util::DeviceExt;

use super::builder;

use wrach_physics_shaders as physics;

const NUM_PARTICLES: u32 = physics::world::NUM_PARTICLES as u32;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 128;

pub struct Pipeline {
    pub particle_bind_groups: Vec<wgpu::BindGroup>,
    pub particle_buffers: Vec<wgpu::Buffer>,
    pub params_buffer: wgpu::Buffer,
    pub grid_buffer: wgpu::Buffer,
    pub pre_compute_pipeline: wgpu::ComputePipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub work_group_count: u32,
    pub bind_group: usize,
}

impl Pipeline {
    pub fn init(device: &wgpu::Device) -> Self {
        let mut builder = builder::Builder::new(device);

        let shader_module = builder::Builder::shader(device, "shaders/physics");

        let params_buffer = builder.params_buffer(physics::compute::Params { stage: 0 });

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

        let initial_particle_data = builder.init_particle_buffer();

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

        let grid: physics::neighbours::PixelMapBasic = [0; physics::neighbours::GRID_SIZE];
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
                        resource: params_buffer.as_entire_binding(),
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

        Self {
            particle_bind_groups,
            particle_buffers,
            params_buffer,
            grid_buffer,
            pre_compute_pipeline,
            compute_pipeline,
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
