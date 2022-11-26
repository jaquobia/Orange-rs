mod rendering;
mod camera;
mod mc_assets;
mod mc_resource_handler;
mod math_helper;
mod mc_constants;
mod utils;

use std::path::PathBuf;

use camera::CameraControllerMovement;
// use egui::{Align2, Frame, Style};
// use egui_wgpu::renderer::ScreenDescriptor;
use image::GenericImageView;
use ultraviolet::Vec3;
use wgpu::{RenderPass, util::DeviceExt};
use winit::{window::{WindowBuilder, Icon}, event_loop::{EventLoop}, event::{WindowEvent, Event, VirtualKeyCode, DeviceEvent}};
use rendering::{GpuStruct, WgpuData, RenderStates, ElapsedTime, Client, tessellator, mesh::Mesh, verticies::TerrainVertex};
use winit_input_helper::WinitInputHelper;
use crate::{math_helper::angle};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

type StateVecType = Vec<Box<dyn RenderStates>>;


pub fn handle_args(args: &Vec<String>) {
    std::mem::drop(args);
}

lazy_static::lazy_static!{
    static ref MC_HOME : PathBuf = {
        let win_appdata = std::env::var("APPDATA");
        let mut dir = std::env::home_dir().unwrap_or_default();
        println!("Home directory is {:?}, if this is incorrect, please make an issue on the github!", dir);
        // let mut dir = if cfg!(windows) && win_appdata.is_ok() {
        //     PathBuf::from(win_appdata.unwrap())
        // } else {
        //     home::home_dir().unwrap().to_path_buf()
        // };
        dir.push(".minecraft");
        dir
    };
}

fn get_icon(name: &str) -> Option<Icon> {
    let icon = image::open(name).unwrap_or_else(|_err| { println!("Failed to load {}", name); image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10)) });
    let (icon_width, icon_height) = icon.dimensions();
    return Some(Icon::from_rgba(icon.into_bytes(), icon_width, icon_height).unwrap());
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    #[cfg(target_arch = "wasm32")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }
    #[cfg(not(target_arch = "wasm32"))] {
        env_logger::init();
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(get_icon("icon.png"))
        .build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let gpu = WgpuData::new(&window);

    let (width, height) = window.inner_size().into();
    let camera = camera::Camera::new((0.0, 0.0, 10.0), angle::Deg(-90.0), angle::Deg(-20.0));
    let projection = camera::Projection::new(width, height, angle::Deg(45.0), 0.1, 100.0);
    let camera_controller = camera::CameraController::new(4.0, 1.0);

    let mut client: Client = Client::new(window, gpu, camera, camera_controller, projection);

    let mut render_time = ElapsedTime::new();
    let mut event_helper = WinitInputHelper::new();

    mc_assets::check_assets();
    mc_resource_handler::mc_terrain_tex_layout(&mut client);
    mc_resource_handler::load_resources(&mut client);

    let mut states: StateVecType = vec![Box::new(State::new(&client))];


    // states.push(Box::new(EguiState::new(&client.gpu, &event_loop)));

    event_loop.run(move |event, _, control_flow| {

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Event::WindowEvent{ event, window_id: _ } = &event {
                for state in &mut states {
                    if state.input(event) {
                        break;
                    }
                }
            }
        }

        if let Event::DeviceEvent { device_id: _, event } = &event {
            if let DeviceEvent::MouseMotion { delta } = event {
                client.camera_controller.process_mouse(delta.0, -delta.1);
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
            if event_helper.key_pressed(VirtualKeyCode::Escape) {
                client.toggle_cursor_visible();
            }
            if let Some(size) = event_helper.window_resized() {
                client.resize(size.into());
            }

            if !client.is_cursor_visible() {
                client.camera_controller.process_scroll(event_helper.scroll_diff());
            }

            render_time.tick();

            client.update(render_time.elasped_time() as f32);
            for state in &mut states {
                state.update(&mut client);
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
                    state.render(&mut render_pass, &client, render_time.elasped_time());
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

    pub test_mesh: Mesh,
    // pub vertex_buffer: wgpu::Buffer,
    // pub num_verticies: u32,

    // pub index_buffer: wgpu::Buffer,
    // pub num_indicies: u32,
}

impl State {
    pub fn new(client: &Client) -> Self {

        let gpu = &client.gpu;

        let device = &gpu.device;
        let config = &gpu.config;

        let mut tess = tessellator::TerrainTessellator::new();
        let test_mesh = tess
        .cuboid(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), [0, 1, 2, 3, 4, 5]).build(device);


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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
                &client.layouts["mc_terrain_tex_layout"],
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    TerrainVertex::desc(),
                ],
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

            test_mesh,
            // vertex_buffer,
            // num_verticies,
            // index_buffer,
            // num_indicies,
        }
    }
}

impl RenderStates for State {

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self, client: &mut Client) {
        client.gpu.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[client.proj_view]));
    }

    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, client: &'a Client, _f_elapsed_time: f64) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &client.textures.get("terrain.png").unwrap().bind_group, &[]);
        self.test_mesh.draw(render_pass);
    }
}

// pub struct EguiState {
//     state: egui_winit::State,
//     context: egui_winit::egui::Context,
//     renderpass: egui_wgpu::renderer::RenderPass, // Necessary for updating the egui pipeline
// }

// impl EguiState {
//     pub fn new<>(gpu: &WgpuData, event_loop: &EventLoop<()>)
//     -> Self {

//         let state = egui_winit::State::new(&event_loop);
//         let context = egui_winit::egui::Context::default();
//         let renderpass = egui_wgpu::renderer::RenderPass::new(&gpu.device, gpu.config.format, 1);

//         Self {
//             state,
//             context,
//             renderpass,
//         }
//     }
// }

// impl RenderStates for EguiState {

//     fn input(&mut self, event: &WindowEvent) -> bool {
//         self.state.on_event(&self.context, event)
//     }

//     fn update(&mut self) {

//     }

//     fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, client: &mut Client, f_elapsed_time: f64) {
//         if client.is_cursor_visible() {
//         let input = self.state.take_egui_input(&client.window);

//         self.context.begin_frame(input);
//         let ctx = &mut self.context;

//         let fps: i32 = (1.0_f64 / f_elapsed_time) as i32;
//         let mut window_style = Style::default();
//         window_style.animation_time = 0.0;
//         // window_style.wrap = Some(false);
//         let window_frame = Frame::window(&window_style);
//         let test_window = egui::Window::new("Test Window")
//         .anchor(Align2::LEFT_TOP, [0.0; 2])
//         .frame(window_frame);
//         test_window.show(ctx, |ui| {
//             // ui.wrap_text();
//             ui.label(format!("Draw Time: {f_elapsed_time}"));
//             ui.label(format!("FPS: {fps}"));
//             let mut vsync = client.gpu.vsync == PresentMode::AutoVsync;
//             let resp = ui.checkbox(&mut vsync, "Vsync");
//             if resp.changed() {
//                 client.set_swap_vsync(true);
//             }
//             let mut cursor = client.is_cursor_visible();
//             let resp = ui.checkbox(&mut cursor, "Cursor Visible?");
//             if resp.changed() {
//                 client.toggle_cursor_visible();
//             }
//         });

//         let full_output = self.context.end_frame();
//         let paint_jobs = self.context.tessellate(full_output.shapes);
//         self.state.handle_platform_output(&client.window, &self.context, full_output.platform_output);

//         let screen_descriptor = ScreenDescriptor {
//             size_in_pixels: [ client.gpu.config.width, client.gpu.config.height ],
//             pixels_per_point: self.state.pixels_per_point(),
//         };

//         for (id, image_delta) in &full_output.textures_delta.set {
//             self.renderpass.update_texture(&client.gpu.device, &client.gpu.queue, *id, image_delta);
//         }
//         for id in &full_output.textures_delta.free {
//             self.renderpass.free_texture(id);
//         }
//         self.renderpass.update_buffers(&client.gpu.device, &client.gpu.queue, &paint_jobs, &screen_descriptor);


//         // Record all render passes.
//         self.renderpass
//         .execute_with_renderpass(render_pass, &paint_jobs, &screen_descriptor);
//     }
//     }
// }
