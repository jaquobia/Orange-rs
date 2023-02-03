use std::{collections::VecDeque, sync::{Arc, RwLock, mpsc}};

use orange_rs::{
    registry::Registry, 
    identifier::Identifier, 
    MCThread, 
    level::dimension::{
        DimensionChunkDescriptor, 
        Dimension
    }, 
    client::{
        client_world::ClientWorld, 
        rendering::{
            ElapsedTime, 
            State, 
            tessellator::TerrainTessellator
        }, 
        camera::CameraControllerMovement, 
        Client, 
        mc_resource_handler
    }, util::pos::{ChunkPos, Position}
};
use ultraviolet::{DVec3, IVec2};
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
        let world_render_state = State::new(
            &client.gpu.device,
            &client.gpu.config,
            &client.projection,
            &client.camera,
            &client.layouts["mc_terrain_tex_layout"],
        );
        client.state.replace(world_render_state);
    }
    let registry = Arc::new(RwLock::new(Registry::load_from(orange_rs::game_version::GameVersion::B173)));

    let shared_tessellator = Arc::new(RwLock::new(TerrainTessellator::new()));

    let mut generate_queue = VecDeque::<DimensionChunkDescriptor>::new();
    let mut tessellate_queue = VecDeque::<DimensionChunkDescriptor>::new();

    // let (tx_gen, rx_gen) = mpsc::channel();
    // let (tx_main_to_tes, rx_main_to_tes) = mpsc::channel();
    // let (tx_tes_to_main, rx_tes_to_main) = mpsc::channel();

    let client_world = Arc::new(RwLock::new(ClientWorld::new(8)));

    // Identifier, id, chunk height, chunk offset
    let chunk_height = 8;
    let level = orange_rs::level::dimension::Dimension::new(
        Identifier::from("overworld"),
        0,
        chunk_height,
        0,
        registry.read().unwrap().get_block_register(),
        );

    client_world.write().unwrap().add_dimension(level);

    use std::thread;
    let mut server_thread_handle = Some(thread::spawn(move || {
        loop {

        }
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
                // Rejoin thread to main thread
                server_thread_handle.take().unwrap().join().expect("Couldn't properly rejoin server to main thread");
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
    
                let player_pos = client.camera.position;

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

                    let sky_color = DVec3::new(0.1, 0.2, 0.3);


                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: sky_color.x,
                                    g: sky_color.y,
                                    b: sky_color.z,
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

                    // Not sure how this would happen, but a possibility exists
                    if client_world.get_player_dimension().is_none() {
                        panic!("A world with no dimension?!");
                    }

                    let render_distance_as_vec = ChunkPos::new(render_distance as i32, render_distance as i32);
                    let player_chunk_pos: ChunkPos = player_pos.to_chunk_pos();
                    let min_extent = player_chunk_pos - render_distance_as_vec;
                    let max_extent = player_chunk_pos + render_distance_as_vec;

                    let state = client.state.as_ref().unwrap();

                    render_pass.set_pipeline(&state.render_pipeline);
                    render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, client.get_texture("terrain.png").bind_group(), &[]);

                    // AABB in frustrum culling?
                    // self.draw_chunks_in_range(&mut render_pass, world, min_extent, max_extent);
                    client_world.draw_chunks(min_extent.clone(), max_extent.clone(), &mut render_pass, &mut tessellate_queue);

                    std::mem::drop(render_pass);
                }

                {
                    let mut client_world = client_world.write().unwrap();
                    client_world.process_chunks();

                    println!("Chunks To Tessellate/Generate: {}", tessellate_queue.len());
                    if let Some(pos) = tessellate_queue.pop_front() {
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
