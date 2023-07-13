pub mod camera;
pub mod minecraft_client;
pub mod client_chunk;
pub mod rendering;
pub mod gui;
pub mod models;
pub mod resource_manager;
pub mod textures;

use std::collections::HashMap;

use crate::client::rendering::wgpu_struct::WgpuData;
use camera::{Camera, CameraController, Projection};
use crate::math_helper::angle;
use rendering::{
    textures::{DepthTextureWrapper, DiffuseTextureWrapper},
};
use ultraviolet::Mat4;
use wgpu::BindGroupLayout;
use winit::window::CursorGrabMode;
use crate::minecraft::mc_resource_handler::CAMERA_BUFFER_NAME;

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

    pub textures: crate::minecraft::mc_resource_handler::TexMapType,
    pub depth_texture: DepthTextureWrapper,

    pipelines: HashMap<String, wgpu::RenderPipeline>,
    buffers: HashMap<String, wgpu::Buffer>,
    bind_group_layouts: HashMap<String, BindGroupLayout>,
    bind_groups: HashMap<String, wgpu::BindGroup>,
}

impl Client {
    pub fn new(
        window: winit::window::Window,
    ) -> Self {
        let gpu = WgpuData::new(&window);

        let (width, height) = window.inner_size().into();
        let camera = camera::Camera::new((0.0, 64.0, 10.0), (0.0, 1.0, 0.0), angle::Deg(-90.0), angle::Deg(-20.0));
        let projection = camera::Projection::new(width, height, angle::Deg(60.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(10.0, 1.0);

        let proj_view = projection.calc_matrix() * camera.calc_matrix();

        let depth_texture = DepthTextureWrapper::new(&gpu, wgpu::TextureFormat::Depth32Float, "DepthTexture");

        Self {
            window,
            gpu,
            camera,
            camera_controller,
            projection,
            proj_view,
            window_center: (width / 2, height / 2),

            swap_vsync: false,
            cursor_visible: true,

            textures: HashMap::new(),
            depth_texture,

            pipelines: HashMap::new(),
            buffers: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            bind_groups: HashMap::new(),
        }
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.gpu.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.gpu.queue
    }

    pub fn get_surface(&self) -> &wgpu::Surface {
        &self.gpu.surface
    }

    pub fn get_surface_configuration(&self) -> &wgpu::SurfaceConfiguration {
        &self.gpu.config
    }

    pub fn get_texture<T: AsRef<str>>(&self, id: T) -> &DiffuseTextureWrapper {
        self.textures.get(id.as_ref()).unwrap()
    }

    pub fn insert_texture<T: AsRef<str>>(&mut self, id: T, texture: DiffuseTextureWrapper) {
        self.textures.insert(id.as_ref().to_string(), texture);
    }

    pub fn get_layout<T: AsRef<str>>(&self, id: T) -> Option<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.get(id.as_ref())
    }

    pub fn get_bind_group<T: AsRef<str>>(&self, id: T) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(id.as_ref())
    }

    pub fn insert_bind_group<T: AsRef<str>>(&mut self, id: T, bind_group: wgpu::BindGroup) {
        self.bind_groups.insert(id.as_ref().to_string(), bind_group);
    }

    pub fn insert_layout<T: AsRef<str>>(&mut self, id: T, layout: wgpu::BindGroupLayout) {
        self.bind_group_layouts.insert(id.as_ref().to_string(), layout);
    }

    pub fn get_pipeline<T: AsRef<str>>(&self, id: T) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(id.as_ref())
    }

    pub fn insert_pipeline<T: AsRef<str>>(&mut self, id: T, pipeline: wgpu::RenderPipeline) {
        self.pipelines.insert(id.as_ref().to_string(), pipeline);
    }

    pub fn get_buffer<T: AsRef<str>>(&self, id: T) -> Option<&wgpu::Buffer> {
        self.buffers.get(id.as_ref())
    }

    pub fn insert_buffer<T: AsRef<str>>(&mut self, id: T, pipeline: wgpu::Buffer) {
        self.buffers.insert(id.as_ref().to_string(), pipeline);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.gpu.resize(new_size);
        self.projection.resize(new_size.0, new_size.1, 40);
        self.window_center = (new_size.0 / 2, new_size.1 / 2);
        self.depth_texture = DepthTextureWrapper::new(
            &self.gpu,
            self.depth_texture.get_texture_format(),
            "depth_texture",
        );
    }

    pub fn update(&mut self, dt: f32) {
        if self.swap_vsync {
            self.gpu.swap_vsync();
            self.swap_vsync = false;
        }
        if self.cursor_visible {
        } else {
            self.camera_controller.update_camera(&mut self.camera, dt);
        }
        self.proj_view = self.projection.calc_matrix() * self.camera.calc_matrix();

        let buffer = self.get_buffer(CAMERA_BUFFER_NAME).unwrap();
        self.get_queue().write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&[self.proj_view]),
        );
    }

    pub fn set_swap_vsync(&mut self, swap_vsync: bool) {
        self.swap_vsync = swap_vsync;
    }

    pub fn toggle_cursor_visible(&mut self) {
        self.cursor_visible = !self.cursor_visible;
        if self.cursor_visible {
            self.window
            .set_cursor_grab(CursorGrabMode::None)
            .unwrap();
        } else {
            self.window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_e| self.window.set_cursor_grab(CursorGrabMode::Confined))
                .unwrap();
        }

        self.window.set_cursor_visible(self.cursor_visible);
        self.camera_controller.reset_mouse();
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    } 
}
