mod rendering;
mod camera;
mod mc_resource_handler;
mod math_helper;
mod direction;
mod world;

use std::{path::PathBuf};

use camera::CameraControllerMovement;
use image::GenericImageView;
use ultraviolet::Vec3;
use wgpu::{RenderPass, util::DeviceExt};
use winit::{window::{Icon, Window}, event_loop::{EventLoop}, event::{WindowEvent, Event, VirtualKeyCode, DeviceEvent}};
use rendering::{GpuStruct, WgpuData, RenderStates, ElapsedTime, Client, tessellator, mesh::Mesh, verticies::TerrainVertex};
use winit_input_helper::WinitInputHelper;
use crate::{math_helper::angle};


type StateVecType = Vec<Box<dyn RenderStates>>;


pub fn handle_args(args: &Vec<String>) {
    std::mem::drop(args);
}

lazy_static::lazy_static!{
    pub static ref MC_HOME : PathBuf = {
        PathBuf::from("./")
    };
}

pub fn get_app_icon(name: &str) -> Option<Icon> {
    let icon = image::open(name).unwrap_or_else(|_err| { println!("Failed to load {}", name); image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10)) });
    let (icon_width, icon_height) = icon.dimensions();
    return Some(Icon::from_rgba(icon.into_bytes(), icon_width, icon_height).unwrap());
}

pub fn main_loop(event_loop: EventLoop<()>, window: Window) {

    let gpu = WgpuData::new(&window);

    let (width, height) = window.inner_size().into();
    let camera = camera::Camera::new((0.0, 0.0, 10.0), angle::Deg(-90.0), angle::Deg(-20.0));
    let projection = camera::Projection::new(width, height, angle::Deg(45.0), 0.1, 100.0);
    let camera_controller = camera::CameraController::new(4.0, 1.0);

    let mut client: Client = Client::new(window, gpu, camera, camera_controller, projection);

    let mut render_time = ElapsedTime::new();
    let mut event_helper = WinitInputHelper::new();

    mc_resource_handler::mc_terrain_tex_layout(&mut client);
    mc_resource_handler::load_binary_resources(&mut client);

    let mut states: StateVecType = vec![Box::new(State::new(&client))];

    event_loop.run(move |event, _, control_flow| {

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
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: client.depth_texture.get_view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
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
        {
            let test_color = Vec3::new(0.0, 1.0, 0.0);
            let test_tex_indicies = [4, 4, 4, 4, 4, 4];
            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        tess.cuboid(Vec3::new(x as f32, y as f32, z as f32), test_color, test_tex_indicies);
                    }
                }
            }
        }

        let test_mesh = tess.build(device);



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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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
        render_pass.set_bind_group(1, client.get_texture("terrain.png").bind_group(), &[]);
        self.test_mesh.draw(render_pass);
    }
}
