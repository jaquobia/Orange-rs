use std::{collections::VecDeque, sync::{Arc, RwLock, mpsc}};

use client::{client_world::{ClientWorld, ClientChunkStorage}, rendering::{wgpu_struct::WgpuData, ElapsedTime, State, tessellator::{TerrainTessellator, self}}, camera::{self, CameraControllerMovement}, Client, mc_resource_handler};
use orange_rs::{math_helper::angle, registry::Registry, block::block_factory::BlockFactory, level::{dimension::{Dimension, DimensionChunkDescriptor}, chunk::Chunk}, identifier::Identifier, MCThread};
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
    let mut registry = Arc::new(RwLock::new(Registry::load_from(orange_rs::game_version::GameVersion::B173)));

    let mut shared_tessellator = Arc::new(RwLock::new(TerrainTessellator::new()));

    let mut chunk_generate_queue = VecDeque::<DimensionChunkDescriptor>::new();
    let mut chunk_tesselate_queue = VecDeque::<DimensionChunkDescriptor>::new();

    // let (tx_gen, rx_gen) = mpsc::channel();
    // let (tx_main_to_tes, rx_main_to_tes) = mpsc::channel();
    // let (tx_tes_to_main, rx_tes_to_main) = mpsc::channel();

    let mut client_world = Arc::new(RwLock::new(ClientWorld::new(8)));

    // Identifier, id, chunk height, chunk offset
    let chunk_height = 8;
    let mut level = Dimension::new(
        Identifier::from("overworld"),
        0,
        chunk_height,
        0,
        registry.read().unwrap().get_block_register(),
        );

    client_world.write().unwrap().add_dimension(level);

    use std::thread;
    let mut server_thread_handle = Some(thread::spawn(move || {

    }));
    // let mut mesher_thread_handle = Some(thread::spawn(move || {
    //     let tessellator = shared_tessellator.clone();
    //     let client_world = client_world.clone();
    //
    //     loop {
    //         let some_job = rx_main_to_tes.recv();
    //         match some_job {
    //             Ok(job) => {
    //                 match job {
    //                     MCThread::Shutdown => { break; },
    //                     MCThread::Work(chunk_pos) => {
    //                         let (section_index, chunk_pos) = chunk_pos;
    //                         tessellator.tesselate_chunk_section(section, section_position, blocks);
    //
    //                     },
    //                 } 
    //             },
    //             Err(_) => {
    //                 break;
    //             }
    //         }
    //     }
    //
    // }));

    // client_world.tesselate_chunks(tesselator, tesselation_queue, device, blocks);

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
                    .expect("Couldn't properly rejoin server to main thread");
                // mesher_thread_handle.take().unwrap().join().expect("Couldn't properly rejoin mesher to main thread");
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
                {
                    let client_world = client_world.read().unwrap();
                    let render_distance = 10;
                    client.draw_world(&client_world, &mut encoder, &view, client.camera.position, render_distance, &mut chunk_tesselate_queue); 
                }

                {
                    let mut client_world = client_world.write().unwrap();
                    client_world.process_chunks();

                    while let Some(pos) = chunk_tesselate_queue.pop_front() {
                        let dim = client_world.get_player_dimension_mut().unwrap();
                        if dim.get_chunk_at_vec(pos.1).is_none() {
                            dim.generate_chunk(pos.1);
                        }
                        let mut tessellator = shared_tessellator.write().unwrap();
                        let registry = registry.read().unwrap();
                        let blocks = registry.get_block_register();
                        client_world.tesselate_chunk(pos.1, &mut tessellator, &client.gpu.device, &blocks);
                    }
                }

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
