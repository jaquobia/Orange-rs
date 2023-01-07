use std::collections::HashMap;

use instant::Duration;
use lazy_static::__Deref;
use ultraviolet::{Mat4, IVec3, IVec2};
use wgpu::{BindGroupLayout, CommandEncoder};
use winit::{window::CursorGrabMode};

use crate::{camera::{CameraController, Projection, Camera}, level::{Level, chunk}, State};

use self::textures::{DepthTextureWrapper, DiffuseTextureWrapper};

pub mod tessellator;
pub mod mesh;
pub mod verticies;
pub mod textures;

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

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();
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
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto
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
    fn update(&mut self, client: &mut Client);
    fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, client: &'a Client, f_elapsed_time: f64);
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

    pub fn elasped_time(&self) -> f64 {
        self.dur.as_secs_f64()
    }
}

type StateVecType = Vec<Box<dyn RenderStates>>;

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

    pub textures: crate::mc_resource_handler::TexMapType,
    pub layouts: HashMap<String, BindGroupLayout>,
    pub depth_texture: DepthTextureWrapper,
    
    // pub states: StateVecType,
    pub state: Option<State>,
}

impl Client {
    pub fn new(window: winit::window::Window, gpu: WgpuData, camera: Camera, camera_controller: CameraController, projection: Projection) -> Self {
        let size: (u32, u32) = window.inner_size().into();
        let proj_view = projection.calc_matrix() * camera.calc_matrix();

        let depth_texture = DepthTextureWrapper::new(&gpu, wgpu::TextureFormat::Depth32Float, "DepthTexture");

        // let states = vec![];

        let ret = Self {
            window,
            gpu,
            camera,
            camera_controller,
            projection,
            proj_view: proj_view,
            window_center: (size.0 / 2, size.1 / 2),

            swap_vsync: false,
            cursor_visible: true,

            textures: HashMap::new(),
            layouts: HashMap::new(),
            depth_texture,

            state: None,
        };
        // ret.states.push();
        ret
    }

    pub fn get_texture(&self, id: &str) -> &DiffuseTextureWrapper {
        self.textures.get(id).unwrap()
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.gpu.resize(new_size);
        self.projection.resize(new_size.0, new_size.1);
        self.window_center = (new_size.0 / 2, new_size.1 / 2);
        self.depth_texture = DepthTextureWrapper::new(&self.gpu, self.depth_texture.get_texture_format(), "depth_texture");
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

        // for state in self.states {
        //     state.update(self);
        // }
        
        let state = self.state.as_mut().unwrap();
        self.gpu.queue.write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[self.proj_view]));
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

    pub fn draw_level(&mut self, level: &Level, encoder: &mut CommandEncoder, view: &wgpu::TextureView) {

        let render_distance = 4;
        let player_pos = IVec3::new(0,0,0);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth_texture.get_view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });


        let render_distance_as_vec = IVec2::new(render_distance, render_distance);
        let player_chunk_pos: IVec2 = Level::get_chunk_pos(player_pos.x, player_pos.z).into();
        let min_extent = player_chunk_pos - render_distance_as_vec;
        let max_extent = player_chunk_pos + render_distance_as_vec;

        let state = self.state.as_ref().unwrap();

        render_pass.set_pipeline(&state.render_pipeline);
        render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
        render_pass.set_bind_group(1, self.get_texture("terrain.png").bind_group(), &[]);

        for x in min_extent.x..=max_extent.x {
            for z in min_extent.y..=max_extent.y {
                if let Some(chunk) = level.get_chunk_at(x, z) {
                    chunk.draw(&mut render_pass);
                }
            }
        }

        std::mem::drop(render_pass);
    }
}
