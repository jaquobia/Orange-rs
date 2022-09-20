mod rendering;
mod camera;
mod math_helper;

use camera::CameraControllerMovement;
use egui::{Align2, Frame, Style};
use egui_wgpu::renderer::ScreenDescriptor;
use image::GenericImageView;
use lazy_static::lazy_static;
use wgpu::{RenderPass, PresentMode, util::DeviceExt};
use winit::{window::{WindowBuilder, Icon}, event_loop::{EventLoop}, event::{WindowEvent, Event, VirtualKeyCode}};
use rendering::{GpuStruct, WgpuData, RenderStates, ElapsedTime, Client};
use winit_input_helper::WinitInputHelper;
use crate::math_helper::angle;

type StateVecType = Vec<Box<dyn RenderStates>>;


fn handle_args(args: &Vec<String>) {
    std::mem::drop(args);
}

lazy_static!{
}

fn get_icon(name: &str) -> Option<Icon> {
    let icon = image::open(name).unwrap_or_else(|_err| { image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10)) });
    let (icon_width, icon_height) = icon.dimensions();
    return Some(Icon::from_rgba(icon.into_bytes(), icon_width, icon_height).unwrap());
}

fn main() {

    let args: Vec<String> = std::env::args().collect();
    handle_args(&args);

    let icon = get_icon("mangadex.png");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(icon)
        .build(&event_loop).unwrap();
    
    let gpu = WgpuData::new(&window);

    // window.set_inner_size(PhysicalSize::new(2560, 1080));
    let (width, height) = window.inner_size().into();
    let camera = camera::Camera::new((0.0, 0.0, 0.0), angle::Deg(-90.0), angle::Deg(-20.0));
    let projection = camera::Projection::new(width, height, angle::Deg(45.0), 0.1, 100.0);
    let camera_controller = camera::CameraController::new(4.0, 0.4);

    let mut client: Client = Client::new(window, gpu, camera, camera_controller, projection);

    let mut render_time = ElapsedTime::new();
    let mut event_helper = WinitInputHelper::new();

    let mut states: StateVecType = vec![Box::new(State::new(&client)), Box::new(EguiState::new(&client.gpu, &event_loop))];
    
    event_loop.run(move |event, _, control_flow| {

        if let Event::WindowEvent{ event, window_id: _ } = &event {
            for state in &mut states {
                if state.input(event) {
                    break;
                }
            }
        }

        if event_helper.update(&event) {
            if event_helper.quit() {
                control_flow.set_exit();
                return;
            }
            if event_helper.key_held(VirtualKeyCode::W) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Forward, true);
            }
            if event_helper.key_held(VirtualKeyCode::S) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Backward, true);
            }
            if event_helper.key_held(VirtualKeyCode::A) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Left, true);
            }
            if event_helper.key_held(VirtualKeyCode::D) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Right, true);
            }
            if event_helper.key_held(VirtualKeyCode::Space) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Up, true);
            }
            if event_helper.key_held(VirtualKeyCode::LShift) {
                client.camera_controller.process_keyboard(CameraControllerMovement::Down, true);
            }
            if event_helper.key_pressed(VirtualKeyCode::V) {
                client.set_swap_vsync(true);
            }
            if let Some(size) = event_helper.window_resized() {
                client.resize(size.into());
            }

            {
                client.camera_controller.process_scroll(event_helper.scroll_diff());
                let (mouse_dx, mouse_dy) = event_helper.mouse_diff();
                client.camera_controller.process_mouse(mouse_dx as f64, mouse_dy as f64);
            }

            render_time.tick();

            client.update(render_time.elasped_time() as f32);
            for state in &mut states {
                state.update();
            }

            let render_result: Result<(), wgpu::SurfaceError> = {
                let output = client.gpu.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder: wgpu::CommandEncoder = client.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

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
                    depth_stencil_attachment: None,
                });

                for state in &mut states {
                    state.render(&mut render_pass, &mut client, render_time.elasped_time());
                }
                
                std::mem::drop(render_pass);

                client.gpu.queue.submit(std::iter::once(encoder.finish()));
                output.present();
                Ok(())
            };

            match render_result {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => client.gpu.resize(client.gpu.size.into()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        } // event_helper update
    }); // event loop run

}

pub struct State {
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl State {
    pub fn new(client: &Client) -> Self {

        let gpu = &client.gpu;

        let device = &gpu.device;
        let config = &gpu.config;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        // let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { 
                label: Some("Camera Buffer"), 
                contents: bytemuck::cast_slice(&[client.projection.calc_matrix() * client.camera.calc_matrix()]), 
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST 
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
                

        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", 
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { 
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            }, 
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        Self {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
        }
    }
}

impl RenderStates for State {

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, client: &mut Client, _f_elapsed_time: f64) {
        client.gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[client.proj_view]));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct EguiState {
    state: egui_winit::State,
    context: egui_winit::egui::Context,
    renderpass: egui_wgpu::renderer::RenderPass, // Necessary for updating the egui pipeline
}

impl EguiState {
    pub fn new<>(gpu: &WgpuData, event_loop: &EventLoop<()>)
    -> Self {

        let state = egui_winit::State::new(&event_loop);
        let context = egui_winit::egui::Context::default();
        let renderpass = egui_wgpu::renderer::RenderPass::new(&gpu.device, gpu.config.format, 1);
    
        Self {
            state,
            context,
            renderpass,
        }   
    }
}

impl RenderStates for EguiState {

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.state.on_event(&self.context, event)
    }

    fn update(&mut self) {
        
    }

    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, client: &mut Client, f_elapsed_time: f64) {

        let input = self.state.take_egui_input(&client.window);
        
        self.context.begin_frame(input);
        let ctx = &mut self.context;

        let fps: i32 = (1.0_f64 / f_elapsed_time) as i32;
        let mut window_style = Style::default();
        window_style.animation_time = 0.0;
        // window_style.wrap = Some(false);
        let window_frame = Frame::window(&window_style);
        let test_window = egui::Window::new("Test Window")
        .anchor(Align2::LEFT_TOP, [0.0; 2])
        .frame(window_frame);
        test_window.show(ctx, |ui| {
            // ui.wrap_text();
            ui.label(format!("Draw Time: {f_elapsed_time}"));
            ui.label(format!("FPS: {fps}"));
            let mut vsync = client.gpu.vsync == PresentMode::AutoVsync;
            let resp = ui.checkbox(&mut vsync, "Vsync");
            if resp.changed() { //dirty vsync \
                client.set_swap_vsync(true);
            }
        });

        let full_output = self.context.end_frame();
        let paint_jobs = self.context.tessellate(full_output.shapes);
        self.state.handle_platform_output(&client.window, &self.context, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [ client.gpu.config.width, client.gpu.config.height ],
            pixels_per_point: self.state.pixels_per_point(),
        };
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderpass.update_texture(&client.gpu.device, &client.gpu.queue, *id, image_delta);
        }
        for id in &full_output.textures_delta.free {
            self.renderpass.free_texture(id);
        }
        self.renderpass.update_buffers(&client.gpu.device, &client.gpu.queue, &paint_jobs, &screen_descriptor);


        // Record all render passes.
        self.renderpass
        .execute_with_renderpass(render_pass, &paint_jobs, &screen_descriptor);
    }
}