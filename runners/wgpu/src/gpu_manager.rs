use crate::pipeline::pipeline;
use winit;

use wrach_physics_shaders as physics;

pub struct GPUManager {
    pub window: winit::window::Window,
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: pipeline::Pipeline,
}

impl GPUManager {
    pub async fn setup() -> (Self, winit::event_loop::EventLoop<()>) {
        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder
            .with_title("Wrach")
            .with_inner_size(winit::dpi::PhysicalSize::new(
                physics::world::MAP_WIDTH * physics::world::WINDOW_ZOOM,
                physics::world::MAP_HEIGHT * physics::world::WINDOW_ZOOM,
            ));
        let window = builder.build(&event_loop).unwrap();

        log::info!("Initializing the surface...");

        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

        let instance = wgpu::Instance::new(backend);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
                .await
                .expect("No suitable GPU adapters found on the system!");

        {
            let adapter_info = adapter.get_info();
            println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        }

        let optional_features = pipeline::Pipeline::optional_features();
        let required_features = pipeline::Pipeline::required_features();
        let adapter_features = adapter.features();
        assert!(
            adapter_features.contains(required_features),
            "Adapter does not support required features for this example: {:?}",
            required_features - adapter_features
        );

        let required_downlevel_capabilities = pipeline::Pipeline::required_downlevel_capabilities();
        let downlevel_capabilities = adapter.get_downlevel_properties();
        assert!(
            downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
            "Adapter does not support the minimum shader model required to run this example: {:?}",
            required_downlevel_capabilities.shader_model
        );
        assert!(
            downlevel_capabilities
            .flags
            .contains(required_downlevel_capabilities.flags),
            "Adapter does not support the downlevel capabilities required to run this example: {:?}",
            required_downlevel_capabilities.flags - downlevel_capabilities.flags
            );

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
        let needed_limits =
            pipeline::Pipeline::required_limits().using_resolution(adapter.limits());

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: (optional_features & adapter_features) | required_features,
                    limits: needed_limits,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &config);
        let pipeline = pipeline::Pipeline::init(&device);

        let manager = Self {
            window,
            instance,
            size,
            surface,
            adapter,
            device,
            queue,
            config,
            pipeline,
        };

        (
            manager,
            // `event_loop` has to be passed seperately because of the `event_loop.run()` closure
            event_loop,
        )
    }
}
