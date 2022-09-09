mod rendering;

use egui::{Align2, Frame, Style};
use egui_wgpu::renderer::ScreenDescriptor;
use image::GenericImageView;
use wgpu::{RenderPass};
use winit::{window::{WindowBuilder, Icon, Window}, event_loop::{EventLoop, ControlFlow}, event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState, DeviceEvent}};
use rendering::{GpuStruct, WgpuData, RenderStates, ElapsedTime};
use winit_input_helper::WinitInputHelper;

type StateVecType = Vec<Box<dyn RenderStates>>;

fn handle_args(args: &Vec<String>) {
    std::mem::drop(args);
}

fn do_updates(states: &mut StateVecType)
{
    for state in states {
        state.update();
    }
}

fn do_inputs(states: &mut StateVecType, event: &WindowEvent) -> bool
{
    for state in states {
        if state.input(event) {
            return true;
        }
    }
    return false;
}

fn do_render(gpu: &mut WgpuData, states: &mut StateVecType, window: &mut Window, f_elapsed_time: f64) -> Result<(), wgpu::SurfaceError> {
    let output = gpu.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder: wgpu::CommandEncoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

    for state in states {
        state.render(&mut render_pass, gpu, window, f_elapsed_time);
    }
    
    std::mem::drop(render_pass);

    gpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

fn main() {

    let args: Vec<String> = std::env::args().collect();
    handle_args(&args);

    let icon = image::open("mangadex.png").unwrap();
    let (icon_width, icon_height) = icon.dimensions();
    let icon: Option<Icon> = Some(Icon::from_rgba(icon.into_bytes(), icon_width, icon_height).unwrap());

    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(icon)
        .build(&event_loop).unwrap();
    let mut gpu = WgpuData::new(&window);

    let mut states: StateVecType = vec![Box::new(State::new(&gpu)), Box::new(EguiState::new(&gpu, &event_loop))];

    let mut render_time = ElapsedTime::new();

    let mut event_helper = WinitInputHelper::new();
    
    event_loop.run(move |event, _, control_flow| {

        if Event::MainEventsCleared == event {
            render_time.tick();
            do_updates(&mut states);
            match do_render(&mut gpu, &mut states, &mut window, render_time.elasped_time()) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size.into()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }

        if event_helper.update(&event) {
            if event_helper.quit() {
                control_flow.set_exit();
                return;
            }
            if event_helper.key_held(VirtualKeyCode::W) {
                println!("W HELD");
            }
            if let Some(size) = event_helper.window_resized() {
                gpu.resize(size.into());
            }
        }
        if let Event::WindowEvent{ event, window_id: _ } = &event {
            do_inputs(&mut states, &event);
        }
    });

}

pub struct State {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub fn new(gpu: &WgpuData) -> Self {

        let device = &gpu.device;
        let config = &gpu.config;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        // let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
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
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });
        Self {
            render_pipeline,
        }
    }
}

impl RenderStates for State {

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, _gpu: &mut WgpuData, _window: &Window, _f_elapsed_time: f64) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct EguiState {
    state: egui_winit::State,
    context: egui_winit::egui::Context,
    renderpass: egui_wgpu::renderer::RenderPass,

    scalar: f64,
}

impl EguiState {
    pub fn new(gpu: &WgpuData, event_loop: &EventLoop<()>) -> Self {

        let state = egui_winit::State::new(&event_loop);
        let context = egui_winit::egui::Context::default();
        let renderpass = egui_wgpu::renderer::RenderPass::new(&gpu.device, gpu.config.format, 1);
    
    
        Self {
            state,
            context,
            renderpass,

            scalar: 20.0,
        }   
    }
}

impl RenderStates for EguiState {

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.state.on_event(&self.context, event)
    }

    fn update(&mut self) {
        
    }

    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, gpu: &mut WgpuData, window: &Window, f_elapsed_time: f64) {
        let input = self.state.take_egui_input(&window);
        let fps: i32 = (1.0 / f_elapsed_time) as i32;
        self.context.begin_frame(input);

        let ctx = &mut self.context;
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
            ui.add(egui::Slider::new(&mut self.scalar, 0.0..=360.0).suffix("°"));
        });

        let full_output = self.context.end_frame();
        let paint_jobs = self.context.tessellate(full_output.shapes);
        self.state.handle_platform_output(window, &self.context, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [ gpu.config.width, gpu.config.height ],
            pixels_per_point: self.state.pixels_per_point(),
        };
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderpass.update_texture(&gpu.device, &gpu.queue, *id, image_delta);
        }
        for id in &full_output.textures_delta.free {
            self.renderpass.free_texture(id);
        }
        self.renderpass.update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);


        // Record all render passes.
        self.renderpass
        .execute_with_renderpass(render_pass, &paint_jobs, &screen_descriptor);
    }
}