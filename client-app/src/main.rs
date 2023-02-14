use std::{collections::VecDeque, sync::{Arc, RwLock, mpsc, atomic::AtomicBool, Mutex}, borrow::BorrowMut, ops::{DerefMut, Deref} };
use env_logger::Builder;
use log::{info, LevelFilter};
use orange_rs::{
    registry::Registry, 
    identifier::Identifier, 
    world::dimension::{
        DimensionChunkDescriptor, 
        Dimension
    }, 
    client::{
        minecraft_client::MinecraftClient, 
        rendering::{
            ElapsedTime, 
            State, 
            tessellator::TerrainTessellator
        }, 
        camera::CameraControllerMovement, 
        Client, 
        mc_resource_handler, gui::screen::{Screen, MainMenu}
    }, 
    util::{pos::{
        ChunkPos, 
        Position
    }, workers::WorkerThread}, 
    server::{MinecraftServer, server_player::ServerPlayer},
};
use ultraviolet::DVec3;
use winit::event::{DeviceEvent, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

fn prepare_client(client: &mut Client) {
// Create the texture layout and load the textures from the binary
    mc_resource_handler::mc_terrain_tex_layout(client);
    mc_resource_handler::load_binary_resources(client);

    // Layouts is created after state, but state should be part of client
    // A weird ordering issue that should eventually be fixed, but not important atm
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
}

fn main() {

    // env_logger::init();
    Builder::new().filter_level(LevelFilter::Info).init();

    // Eventually parse these for username and stuff
    let _args: Vec<String> = std::env::args().collect();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(orange_rs::get_app_icon("icon.png"))
        .build(&event_loop)
        .unwrap();

    // The windowing client, manages the window and wgpu pipeline structs
    let mut client: Client = Client::new(window);

    // Evaluates how much time occured between render passes
    let mut render_time = ElapsedTime::new();

    // Helps reduce winit's immediate event system into a polling type
    let mut event_helper = WinitInputHelper::new();

    prepare_client(&mut client);

    // A registry of all the resources to be used by the client or server
    let registry = Arc::new(RwLock::new(Registry::load_from(orange_rs::game_version::GameVersion::B173)));

    // The tessellator to be used to mesh the chunks, intended for multithreaded useage (TODO)
    let shared_tessellator = Arc::new(RwLock::new(TerrainTessellator::new()));

    // Queues for tessellation and generation, might become mostly irrelevenat later
    let mut generate_queue = VecDeque::<DimensionChunkDescriptor>::new();
    let mut tessellate_queue = VecDeque::<DimensionChunkDescriptor>::new();
 
    // The height of the world in chunk sections, to be used to provide compatibility with anvil
    // and mcregion world types; but may also provide ability to configure a custom world height
    let chunk_height = 8;
    
    // The internal server, which as part of the client, will either be of type Integrated or
    // Remote
    let server_world: Arc<RwLock<Option<MinecraftServer>>> = Arc::new(RwLock::new(None));

    let mut minecraft = MinecraftClient::new(chunk_height);
    // minecraft.set_screen(Some(Box::new(MainMenu::new())));
    minecraft.set_screen::<MainMenu>();
    
    {
        // Identifier, id, chunk height, chunk offset
        let level = Dimension::new(
            Identifier::from("overworld"),
            0,
            chunk_height,
            0,
            registry.read().unwrap().get_block_register(),
            );

        match server_world.write() {
            Ok(mut server_world) => {
                *server_world = Some(MinecraftServer::new(orange_rs::server::ServerType::Integrated));
                match &mut *server_world {
                    Some(server_world) => {
                        server_world.dimensions_mut().push(level);
                        server_world.connect_player(ServerPlayer::new())
                    },
                    None => { },
                };
            },
            Err(_) => {  },
        }; 
    }



    let mut tick_time = instant::Instant::now();
    let one_twentieth = instant::Duration::from_secs_f64(1.0 / 20.0);
    let server_world_copy = server_world.clone();
    let mut server_thread = WorkerThread::new();
    server_thread.spawn(move || {
        let tick_time_now = instant::Instant::now(); 

        if (tick_time_now - tick_time) >= one_twentieth {
            // info!("Time now: {:?}", tick_time_now);
            match server_world_copy.write() {
                Ok(mut guard) => { 
                    match guard.as_mut() {
                        Some(server_world) => { 
                            server_world.tick();
                        },
                        _ => { },
                    }
                },
                Err(_) => { },
            };
            // tick_time += one_twentieth;
            tick_time = tick_time_now;
        }
    });
    
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
                server_thread.stop();
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

                let render_distance = 10;
                let render_distance_as_vec = ChunkPos::new(render_distance as i32, render_distance as i32);
                let player_chunk_pos: ChunkPos = player_pos.to_chunk_pos();
                let min_extent = player_chunk_pos - render_distance_as_vec;
                let max_extent = player_chunk_pos + render_distance_as_vec;
                {

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

                    let state = client.state.as_ref().unwrap();

                    render_pass.set_pipeline(&state.render_pipeline);
                    render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, client.get_texture("terrain.png").bind_group(), &[]);

                    // AABB in frustrum culling?
                    minecraft.draw_chunks(min_extent.clone(), max_extent.clone(), &mut render_pass);

                }

                {
                    minecraft.process_chunks(min_extent.clone(), max_extent.clone(), &mut tessellate_queue);
                    if tessellate_queue.len() != 0 {
                        info!("Chunks To Tessellate/Generate: {}", tessellate_queue.len());
                    }

                    let max_tesselations = 5.min(tessellate_queue.len());
                    for _ in 0..max_tesselations {
                        if let Some(pos) = tessellate_queue.pop_front() { 
                            let mut tessellator = shared_tessellator.write().unwrap();
                            let registry = registry.read().unwrap();
                            let blocks = registry.get_block_register();
                            match server_world.read() {
                                Ok(server_world) => {
                                    match &*server_world {
                                        Some(server_world) => {
                                            match minecraft.tesselate_chunk(pos.1, &mut tessellator, &client.gpu.device, &blocks, server_world.dimensions().get(0).unwrap()) {
                                                Ok(()) => {  },
                                                Err(()) => { tessellate_queue.push_back(pos); },
                                            };
                                        },
                                        None => { },
                                    };
                                },
                                Err(_) => {  },
                            };

                            
                        }
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
