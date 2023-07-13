pub struct WgpuData {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub vsync: wgpu::PresentMode,
}

impl WgpuData {
    pub fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        // let instance = wgpu::Instance::new(InstanceDescriptor {
        //     backends: Backends::GL,
        //     ..Default::default()
        // });
        let surface = unsafe { instance.create_surface(window).expect("No Surface") };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();
        log::info!("Selected Adapter: {:?}", adapter.get_info());

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ))
        .unwrap();

        let caps = surface.get_capabilities(&adapter); 
        let format = caps.formats[0].clone();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,

            vsync: wgpu::PresentMode::AutoVsync,
        }
    }

    pub fn swap_vsync(&mut self) {
        self.vsync = match self.vsync {
            wgpu::PresentMode::AutoNoVsync => wgpu::PresentMode::AutoVsync,
            _ => wgpu::PresentMode::AutoNoVsync,
        };
        self.config.present_mode = self.vsync;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size.into();
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.config.present_mode = self.vsync;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
