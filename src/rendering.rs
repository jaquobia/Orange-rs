pub trait GpuStruct {
    fn resize(&mut self, new_size: (u32, u32));
}

pub struct WgpuData {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl WgpuData {
    pub fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

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
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&device, &config);

        

        Self {
            surface,
            device, 
            queue,
            config,
            size,

        }
    }
}

impl GpuStruct for WgpuData {
    fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size.into();
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }
}


pub trait RenderStates {
    fn input(&mut self, event: &winit::event::WindowEvent) -> bool;
    fn update(&mut self);
    fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, gpu: &mut WgpuData, window: &winit::window::Window, f_elapsed_time: f64);
}

pub struct ElapsedTime {
    time_now: std::time::Instant,
    time_last: std::time::Instant,
    elasped_time: f64,
}

impl ElapsedTime {
    pub fn new() -> Self {
        let time_now = std::time::Instant::now();
        let time_last = time_now;
        Self {
            time_now,
            time_last,
            elasped_time: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.time_last = self.time_now;
        self.time_now = std::time::Instant::now();
        let diff = self.time_now - self.time_last;
        self.elasped_time = diff.as_secs_f64();
    }

    pub fn elasped_time(&self) -> f64 {
        self.elasped_time
    }
}

