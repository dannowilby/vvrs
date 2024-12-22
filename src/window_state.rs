use std::sync::Arc;

use wgpu::{
    Adapter, CompositeAlphaMode, DeviceDescriptor, Features, Instance, InstanceDescriptor,
    PresentMode, RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::window::Window;

pub struct WindowState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub instance: Instance,
    pub adapter: Adapter,

    pub window: Arc<Window>,
}

impl WindowState {
    pub async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();

        // Set up surface
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();

        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::default()
                .union(Features::MULTI_DRAW_INDIRECT)
                .union(Features::POLYGON_MODE_LINE)
                .union(Features::INDIRECT_FIRST_INSTANCE),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        log::info!(
            "Maximum buffer size: {}MiB",
            device.limits().max_buffer_size / (2u64.pow(20))
        );
        log::info!(
            "Maximum number of vertex buffers: {}",
            device.limits().max_vertex_buffers
        );

        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface");
        let swapchain_format = TextureFormat::Bgra8UnormSrgb;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: PresentMode::AutoNoVsync, // Set Vsync/No Vsync here
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
            window,
        }
    }
}
