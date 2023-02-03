pub mod camera;
pub mod client_world;
pub mod client_chunk;
pub mod mc_resource_handler;
pub mod rendering;
pub mod gui;

use std::collections::HashMap;

use crate::client::rendering::wgpu_struct::WgpuData;
use camera::{Camera, CameraController, Projection};
use crate::math_helper::angle;
use rendering::{
    textures::{DepthTextureWrapper, DiffuseTextureWrapper},
    State,
};
use ultraviolet::Mat4;
use wgpu::BindGroupLayout;
use winit::window::CursorGrabMode;

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

    pub textures: crate::client::mc_resource_handler::TexMapType,
    pub layouts: HashMap<String, BindGroupLayout>,
    pub depth_texture: DepthTextureWrapper,

    pub state: Option<State>,
}

impl Client {
    pub fn new(
        window: winit::window::Window,
    ) -> Self {
        let gpu = WgpuData::new(&window);

        let (width, height) = window.inner_size().into();
        let camera = camera::Camera::new((0.0, 64.0, 10.0), angle::Deg(-90.0), angle::Deg(-20.0));
        let projection = camera::Projection::new(width, height, angle::Deg(45.0), 0.1, 100.0);
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
            layouts: HashMap::new(),
            depth_texture,

            state: None,
        }
    }

    pub fn get_texture(&self, id: &str) -> &DiffuseTextureWrapper {
        self.textures.get(id).unwrap()
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

        let state = self.state.as_mut().unwrap();
        self.gpu.queue.write_buffer(
            &state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.proj_view]),
        );
    }

    pub fn set_swap_vsync(&mut self, swap_vsync: bool) {
        self.swap_vsync = swap_vsync;
    }

    pub fn toggle_cursor_visible(&mut self) {
        self.cursor_visible = !self.cursor_visible;
        self.window
            .set_cursor_grab(if self.cursor_visible {
                CursorGrabMode::None
            } else {
                CursorGrabMode::Locked
            })
            .unwrap();
        self.window.set_cursor_visible(self.cursor_visible);
        self.camera_controller.reset_mouse();
    }

    pub fn is_cursor_visible(&mut self) -> bool {
        self.cursor_visible
    } 
}
