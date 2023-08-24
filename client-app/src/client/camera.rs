use ultraviolet::{projection::perspective_wgpu_dx, Mat4, Vec3};

use orange_rs::{world::chunk::CHUNK_SECTION_AXIS_SIZE, math_helper::angle::Rad};

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    front: Vec3,
    right: Vec3,
    up: Vec3,
    world_up: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new<V: Into<Vec3>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        up: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let rad_yaw: Rad<f32> = yaw.into();
        let rad_pitch: Rad<f32> = pitch.into();
        Self::raw_new(position.into(), up.into(), rad_yaw.0, rad_pitch.0)
    }

    pub fn raw_new(position: Vec3, up: Vec3, yaw: f32, pitch: f32) -> Self {

        let mut ret = Self {
            position,
            front: Vec3::new(0.0, 0.0, 0.0),
            right: Vec3::new(0.0, 0.0, 0.0),
            up,
            world_up: up,
            yaw,
            pitch,
        };
        Self::update_vectors(&mut ret);
        ret
    }

    pub fn yaw_pitch(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    pub fn vectors(&self) -> (Vec3, Vec3, Vec3) {
        (self.front, self.right, self.up)
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::look_at(
            self.position,
            self.position + self.front,
            self.up,
        )
    }

    pub fn update_vectors(&mut self) {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        self.front = Vec3::new(
            cos_yaw * cos_pitch,
            sin_pitch,
            sin_yaw * cos_pitch,
        ).normalized();
        self.right = self.front.cross(self.world_up).normalized();
        self.up = self.right.cross(self.front).normalized();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        let rad_fov: Rad<f32> = fovy.into();
        Self {
            aspect: width as f32 / height as f32,
            fovy: rad_fov.0,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, render_distance: u32) {
        self.aspect = width as f32 / height as f32;
        self.zfar = (render_distance as usize * CHUNK_SECTION_AXIS_SIZE) as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        perspective_wgpu_dx(self.fovy, self.aspect, self.znear, self.zfar)
        // projection::perspective_gl(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub enum CameraControllerMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, movement: CameraControllerMovement, pressed: bool) -> bool {
        let amount = if pressed { 1.0 } else { 0.0 };
        match movement {
            CameraControllerMovement::Forward => {
                self.amount_forward = amount;
                true
            }
            CameraControllerMovement::Backward => {
                self.amount_backward = amount;
                true
            }
            CameraControllerMovement::Left => {
                self.amount_left = amount;
                true
            }
            CameraControllerMovement::Right => {
                self.amount_right = amount;
                true
            }
            CameraControllerMovement::Up => {
                self.amount_up = amount;
                true
            }
            CameraControllerMovement::Down => {
                self.amount_down = amount;
                true
            }
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal += mouse_dx as f32;
        self.rotate_vertical += mouse_dy as f32;
    }

    pub fn reset_mouse(&mut self) {
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalized();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalized();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        const SENSITIVITY_CORRECTION: f32 = 0.005_f32;

        // Rotate
        camera.yaw += self.rotate_horizontal * self.sensitivity * SENSITIVITY_CORRECTION;
        camera.pitch += self.rotate_vertical * self.sensitivity * SENSITIVITY_CORRECTION;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }

        camera.update_vectors();

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        self.amount_left = 0.0;
        self.amount_right = 0.0;
        self.amount_forward = 0.0;
        self.amount_backward = 0.0;
        self.amount_up = 0.0;
        self.amount_down = 0.0;
    }
}
