use std::collections::VecDeque;

use client::{client_world::{ClientWorld, ClientChunkStorage}, rendering::{wgpu_struct::WgpuData, ElapsedTime, State, tessellator::TerrainTessellator}, camera::{self, CameraControllerMovement}, Client, mc_resource_handler};
use orange_rs::{math_helper::angle, registry::Registry, block::block_factory::BlockFactory, level::dimension::{Dimension, DimensionChunkDescriptor}, identifier::Identifier};
use winit::event::{DeviceEvent, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

fn main() {
    env_logger::init();

    // Eventually parse these for username and stuff
    let _args: Vec<String> = std::env::args().collect();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(orange_rs::get_app_icon("icon.png"))
        .build(&event_loop)
        .unwrap();

    let mut client: Client = Client::new(window);

    let mut render_time = ElapsedTime::new();
    let mut event_helper = WinitInputHelper::new();

    mc_resource_handler::mc_terrain_tex_layout(&mut client);
    mc_resource_handler::load_binary_resources(&mut client);

    // Layouts is created after state, but state should be part of client
    {
        let state = State::new(
            &client.gpu.device,
            &client.gpu.config,
            &client.projection,
            &client.camera,
            &client.layouts["mc_terrain_tex_layout"],
        );
        client.state.replace(state);
    }
    let mut registry = Registry::load_from(orange_rs::game_version::GameVersion::B173);

    let mut tessellator = TerrainTessellator::new();

    let chunk_generate_queue = VecDeque::<DimensionChunkDescriptor>::new();
    let chunk_tesselate_queue = VecDeque::<DimensionChunkDescriptor>::new();

    let mut client_world = ClientWorld::new();

    // Identifier, id, chunk height, chunk offset
    let mut level = Dimension::<ClientChunkStorage>::new(
        Identifier::from("overworld"),
        0,
        8,
        0,
        registry.get_block_register(),
    );

    client_world.add_dimension(level);

    use std::thread;
    let mut server_thread_handle = Some(thread::spawn(move || {
        
    }));
    let mut mesher_thread_handle = Some(thread::spawn(move || {

    }));

    event_loop.run(move |event, _, control_flow| {
        if let winit::event::Event::DeviceEvent {
            device_id: _,
            event,
        } = &event
        {
            if let DeviceEvent::MouseMotion { delta } = event {
                client.camera_controller.process_mouse(delta.0, -delta.1);
            }
        }

        if event_helper.update(&event) {
            if event_helper.quit() {
                control_flow.set_exit();
                // Rejoin thread to
                server_thread_handle
                    .take()
                    .unwrap()
                    .join()
                    .expect("Couldn't properly rejoin main thread");
                return;
            }
            if event_helper.key_held(VirtualKeyCode::W) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Forward, true);
            }
            if event_helper.key_held(VirtualKeyCode::S) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Backward, true);
            }
            if event_helper.key_held(VirtualKeyCode::A) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Left, true);
            }
            if event_helper.key_held(VirtualKeyCode::D) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Right, true);
            }
            if event_helper.key_held(VirtualKeyCode::Space) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Up, true);
            }
            if event_helper.key_held(VirtualKeyCode::LShift) {
                client
                    .camera_controller
                    .process_keyboard(CameraControllerMovement::Down, true);
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

            render_time.tick();

            client.update(render_time.elasped_time() as f32);

            let render_result: Result<(), wgpu::SurfaceError> = {
                let output = client.gpu.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder: wgpu::CommandEncoder =
                    client
                        .gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                client.draw_world(&client_world, &mut encoder, &view, client.camera.position, 0);

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
