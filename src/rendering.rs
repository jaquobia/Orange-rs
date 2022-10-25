use instant::Duration;
use ultraviolet::Mat4;
use winit::{window::CursorGrabMode};

use crate::camera::{CameraController, Projection, Camera};

pub trait GpuStruct {
    fn resize(&mut self, new_size: (u32, u32));
}

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
        { // Debug window height
            let width = size.width;
            let height = size.height;
            println!("Window Size: {width} {height}");
        }

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        // #[cfg(not(target_arch = "wasm32"))]
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        // #[cfg(target_arch = "wasm32")]
        // let adapter = instance
        //     .enumerate_adapters(wgpu::Backends::all())
        //     .filter(|adapter| {
        //         // Check if this adapter supports our surface
        //         !surface.get_supported_formats(&adapter).is_empty()
        //     })
        //     .next()
        //     .unwrap();


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
            // alpha_mode: wgpu::CompositeAlphaMode::Auto // 0.14.0
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
}

impl GpuStruct for WgpuData {
    fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size.into();
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.config.present_mode = self.vsync;
            self.surface.configure(&self.device, &self.config);

        }
    }
}

impl WgpuData {
    pub fn swap_vsync(&mut self) {
        self.vsync = match self.vsync {
            wgpu::PresentMode::AutoNoVsync => { wgpu::PresentMode::AutoVsync }
            _ => { wgpu::PresentMode::AutoNoVsync },
        };
        self.config.present_mode = self.vsync;
        self.surface.configure(&self.device, &self.config);
    }
}

pub trait RenderStates {
    fn input(&mut self, event: &winit::event::WindowEvent) -> bool;
    fn update(&mut self);
    fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, client: &mut Client, f_elapsed_time: f64);
}

pub struct ElapsedTime {
    time_now: instant::Instant,
    time_last: instant::Instant,
    dur: instant::Duration,
}

impl ElapsedTime {
    pub fn new() -> Self {
        let time_now = instant::Instant::now();
        let time_last = time_now;
        Self {
            time_now,
            time_last,
            dur: Duration::from_secs(0),
        }
    }

    pub fn tick(&mut self) {
        self.time_last = self.time_now;
        self.time_now = instant::Instant::now();
        self.dur = self.time_now - self.time_last;
    }

    pub fn duration(&self) -> instant::Duration {
        self.dur
    }

    pub fn elasped_time(&self) -> f64 {
        self.dur.as_secs_f64()
    }
}

pub struct Client {
    pub window: winit::window::Window,
    pub gpu: WgpuData,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub projection: Projection,
    pub proj_view: Mat4,
    pub window_center: (u32, u32),

    swap_vsync: bool,
    cursor_visible: bool,
}

impl Client {
    pub fn new(window: winit::window::Window, gpu: WgpuData, camera: Camera, camera_controller: CameraController, projection: Projection) -> Self {
        let size: (u32, u32) = window.inner_size().into();
        let proj_view = projection.calc_matrix() * camera.calc_matrix();
        Self {
            window,
            gpu,
            camera,
            camera_controller,
            projection,
            proj_view: proj_view,
            window_center: (size.0 / 2, size.1 / 2),

            swap_vsync: false,
            cursor_visible: true,
        }
    }
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.gpu.resize(new_size);
        self.projection.resize(new_size.0, new_size.1);
        self.window_center = (new_size.0 / 2, new_size.1 / 2);
    }

    pub fn update(&mut self, dt: f32) {

        if self.swap_vsync {
            self.gpu.swap_vsync();
            self.swap_vsync = false;
        }
        if self.cursor_visible {

        }
        else {
            self.camera_controller.update_camera(&mut self.camera, dt);
            self.proj_view = self.projection.calc_matrix() * self.camera.calc_matrix();
        }
    }

    pub fn set_swap_vsync(&mut self, swap_vsync: bool) {
        self.swap_vsync = swap_vsync;
    }
    pub fn toggle_cursor_visible(&mut self) {
        self.cursor_visible = !self.cursor_visible;
        self.window.set_cursor_grab(if self.cursor_visible { CursorGrabMode::None } else { CursorGrabMode::Locked }).unwrap();
        self.window.set_cursor_visible(self.cursor_visible);
        self.camera_controller.reset_mouse();
    }
    pub fn is_cursor_visible(&mut self) -> bool {
        self.cursor_visible
    }
}
