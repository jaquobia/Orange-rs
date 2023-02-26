use std::{collections::VecDeque, sync::{Arc, RwLock, mpsc, atomic::AtomicBool, Mutex}, borrow::BorrowMut, ops::{DerefMut, Deref} };
use env_logger::Builder;
use log::{info, LevelFilter, warn};
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
        Position, EntityPos, BlockPos
    }, workers::WorkerThread}, 
    server::{
        MinecraftServer,
        server_player::ServerPlayer
    }, packets::prot14::Packet,
};
use orange_networking::{network_interface::NetworkThread, packet::PacketEnumHolder};
use ultraviolet::DVec3;
use winit::event::{DeviceEvent, VirtualKeyCode, WindowEvent, Event};
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

#[derive(Debug)]
enum ServerConnectError {
    InvalidAddress,
    Kick(String),
}

fn join_server(username: String, protocol_id: i32, address: String, port: u32) -> Result<(NetworkThread<Packet>, u64, BlockPos, EntityPos, f64, f32, f32, bool), ServerConnectError> {
    let network_thread = match NetworkThread::connect_to_server(address, port) {
        Ok(nt) => { nt },
        Err(_) => { return Err(ServerConnectError::InvalidAddress); }
    };

    network_thread.send_packet(Packet::Handshake { handshake_data: username.clone() });
    let mut ttime = 0;
    let mut spawn_position = BlockPos::new(0, 0, 0);
    let mut player_position = EntityPos::new(0.0, 0.0, 0.0);
    let mut player_stance = 0.0;
    let mut player_pitch = 0.0;
    let mut player_yaw = 0.0;
    let mut player_on_ground = false;

    let mut do_login = true;

    while do_login {
    for packet in network_thread.get_packets() {
        match packet {
            Packet::KeepAlive => { network_thread.send_packet(Packet::KeepAlive {}); },
            Packet::Handshake { handshake_data } => {
                let login_packet = Packet::Login{ protocol: protocol_id, username: username.clone(), seed: 0, dimension: 0 };
                network_thread.send_packet(login_packet); 
                warn!("Handshake Packet Recieved! {handshake_data}, sending login request as {username}."); 
            },
            Packet::Login { protocol, username, seed, dimension } => { 
                warn!("Login successful!");
            },
            Packet::TimeUpdate { time } => { ttime = time; },
            Packet::SpawnPosition { x, y, z } => { spawn_position = BlockPos::new(x, y, z); },
            Packet::UpdateHealth { health } => { warn!("Update Health"); },
            Packet::PlayerPositionAndLook { x, y_c_stance_s, stance_c_y_s, z, yaw, pitch, on_ground } => { 
                player_position = EntityPos::new(x as f32, y_c_stance_s as f32, z as f32);
                player_stance = stance_c_y_s;
                player_on_ground = on_ground;
                player_yaw = yaw;
                player_pitch = pitch;
                network_thread.send_packet(Packet::PlayerPositionAndLook { x, y_c_stance_s: stance_c_y_s, stance_c_y_s: y_c_stance_s, z, yaw, pitch, on_ground });

                do_login = false;
            },
            Packet::NamedEntitySpawn { entity, name, x, y, z, rotation, pitch, held_item } => { warn!("{name} spawned"); },
            Packet::SpawnMob { entity, entity_type, x, y, z, yaw, pitch, meta } => { warn!("Spawn Mob"); },
            Packet::EntityVelocity { entity, vel_x, vel_y, vel_z } => { warn!("Entity Velocity"); },
            Packet::Entity { entity } => { warn!("Spawn {entity}"); },
            Packet::EntityMoveRelative { entity, dx, dy, dz } => { warn!("Entity Move Rel"); },
            Packet::EntityLook { entity, yaw, pitch } => { warn!("Entitiy Look"); },
            Packet::EntityLookMoveRelative { entity, dx, dy, dz, yaw, pitch } => { warn!("Entity Move Look"); },
            Packet::EntityStatus { entity, status } => { warn!("Entity Status"); },
            Packet::EntityMeta { entity, meta } => { warn!("Entity Meta"); },
            Packet::PreChunk { x, z, mode } => { warn!("PreChunk {x},{z}"); },
            Packet::MapChunk { x, y, z, size_x, size_y, size_z, compressed_data } => { warn!("Chunk Update: {x},{y},{z}"); },
            Packet::BedWeatherState { state_reason } => {  },
            Packet::SetContainerSlot { window_id, slot, item_data } => { warn!("Set Slot Item"); },
            Packet::SetWindowItems { window_id, window_data } => { warn!("Set Window Item"); },
            Packet::ItemData { item_type, item_id, item_data } => { warn!("Item Data"); },
            Packet::DisconnectKick { reason } => { return Err(ServerConnectError::Kick(reason)); }
            _ => { warn!("Unexpected packet during login: {packet:?}"); }
        }
    }
    }
    warn!("Logged in, leaving the login sequence.");
    Ok((network_thread, ttime, spawn_position, player_position, player_stance, player_pitch, player_yaw, player_on_ground))
}

fn main() {

    // env_logger::init();
    Builder::new().filter_level(LevelFilter::Off).init();

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

    let mut on_ground_real = true;
    let mut stance = 0.0;
    
    let username = String::from("TT");
    let address = "127.0.0.0".to_string();
    let port = 25565;
    let mut network_thread = match join_server(username, 14, address, port) {
        Ok((network_thread, time, spawn_position, player_position, player_stance, player_pitch, player_yaw, player_on_ground)) => {
            client.camera.position = player_position;
            client.camera.pitch = player_pitch;
            client.camera.yaw = player_yaw;
            on_ground_real = player_on_ground;
            stance = player_stance;
            network_thread
        },
        Err(e) => {
            warn!("Failed to connect to server: {:?}", e);
            return;
        },
    };
    let mut tick_time = instant::Instant::now();
    let one_twentieth = instant::Duration::from_secs_f64(1.0 / 20.0);
    let server_world_copy = server_world.clone();
    let mut server_thread = WorkerThread::new();
    server_thread.spawn(move |running| {
        let tick_time_now = instant::Instant::now(); 

        if !running {
            network_thread.stop();
            return;
        }

        if (tick_time_now - tick_time) < one_twentieth {
            return;
        }

        if let Ok(mut a) = server_world_copy.write() {
            if let Some(server_world) = a.as_mut() {
                server_world.tick();
                client.camera.position += (0.0, -0.04, 0.0).into();
                println!("{:?}", client.camera.position);
                let (x, y, z) = client.camera.position.into();
                let (yaw, pitch) = (client.camera.yaw, client.camera.pitch);
                network_thread.send_packet(Packet::PlayerPositionAndLook { x: x as f64, y_c_stance_s: y as f64, stance_c_y_s: (y as f64 + 1.6), z: z as f64, yaw: yaw as f32, pitch: pitch as f32, on_ground: false });
                network_thread.send_packet(Packet::KeepAlive);
                for packet in network_thread.get_packets() {
                    match packet {
                        Packet::KeepAlive => { warn!("Keep Alive"); },
                        Packet::Handshake { handshake_data } => { warn!("Unexpectedly received a handshake packet! This is not supposed to happen after login!"); },
                        Packet::Login { protocol, username, seed, dimension } => { warn!("Unexpectedly received a login packet! This is not supposed to happen after login!"); },
                        Packet::Chat { chat_data } => { warn!("[Chat]{chat_data}"); },
                        Packet::TimeUpdate { time } => { warn!("Time Update"); },
                        Packet::EntityChangeEquipment { entity_id, equipment_slot, item_id, item_damage } => { warn!("Entity Change Equipment"); },
                        Packet::SpawnPosition { x, y, z } => { warn!("Spawn Position"); },
                        Packet::InteractWithEntity { user, entity, is_left_click } => { warn!("Interact with entity"); },
                        Packet::UpdateHealth { health } => { warn!("Update Health"); },
                        Packet::Respawn { world } => { warn!("Respawn"); },
                        Packet::PlayerOnGround { on_ground } => { warn!("Player On Ground"); },
                        Packet::PlayerPosition { x, y, stance, z, on_ground } => { warn!("Recieved a position packet"); },
                        Packet::PlayerLook { yaw, pitch, on_ground } => { warn!("Player L"); },
                        Packet::PlayerPositionAndLook { x, y_c_stance_s, stance_c_y_s, z, yaw, pitch, on_ground } => { 
                            client.camera.position = (x as f32, y_c_stance_s as f32, z as f32).into();
                            client.camera.pitch = pitch;
                            client.camera.yaw = yaw;
                            on_ground_real = on_ground;
                            stance = stance_c_y_s;
                        },
                        Packet::PlayerDigging { status, x, y, z, face } => { warn!("Player Digging: {status}"); },
                        Packet::PlayerUse { x, y, z, direction, item_data } => { warn!("Player Use"); },
                        Packet::PlayerChangeSlot { slot } => { warn!("Player Change Slot"); },
                        Packet::PlayerUseBed { entity, in_bed, x, y, z } => { warn!("Player Use Bed"); },
                        Packet::Animation { entity, animat } => { warn!("Animation"); },
                        Packet::EntityAction { entity, action } => { warn!("Entity Action"); },
                        Packet::NamedEntitySpawn { entity, name, x, y, z, rotation, pitch, held_item } => { warn!("{name} spawned"); },
                        Packet::PickupSpawn { entity, item, count, damage_meta, x, y, z, rotation, pitch, roll } => { warn!("Pickup Spawned"); },
                        Packet::CollectItem { item_entity, collector_entity } => { warn!("Collect Item"); },
                        Packet::CreateNonMobEntity { entity, entity_type, x, y, z, unknown } => { warn!("Create NonMob Entity"); },
                        Packet::SpawnMob { entity, entity_type, x, y, z, yaw, pitch, meta } => { warn!("Spawn Mob"); },
                        Packet::EntityPaintings { entity, title, x, y, z, direction } => { warn!("Entity Painting {title}"); },
                        Packet::UpdatePosition { strafe, forward, pitch, yaw, unk, is_jumping } => { warn!("UpdatePosition"); },
                        Packet::EntityVelocity { entity, vel_x, vel_y, vel_z } => { warn!("Entity Velocity"); },
                        Packet::DestroyEntity { entity } => { warn!("Destroy Entity"); },
                        Packet::Entity { entity } => { warn!("Spawn {entity}"); },
                        Packet::EntityMoveRelative { entity, dx, dy, dz } => { warn!("Entity Move Rel"); },
                        Packet::EntityLook { entity, yaw, pitch } => { warn!("Entitiy Look"); },
                        Packet::EntityLookMoveRelative { entity, dx, dy, dz, yaw, pitch } => { warn!("Entity Move Look"); },
                        Packet::EntityTeleport { entity, x, y, z, yaw, pitch } => { warn!("Entity Teleport"); },
                        Packet::EntityStatus { entity, status } => { warn!("Entity Status"); },
                        Packet::AttachEntity { entity, vehicle_entity } => { warn!("Attch Entity"); },
                        Packet::EntityMeta { entity, meta } => { warn!("Entity Meta"); },
                        Packet::PreChunk { x, z, mode } => { warn!("PreChunk {x},{z}"); },
                        Packet::MapChunk { x, y, z, size_x, size_y, size_z, compressed_data } => { warn!("Chunk Update: {x},{y},{z}"); },
                        Packet::MultiBlockChange { chunk_x, chunk_z, num_blocks, coords_type_metadata_array } => { warn!("Multi Block Change"); },
                        Packet::BlockChange { x, y, z, block_type, metadata } => { warn!("Block Change"); },
                        Packet::BlockAction { x, y, z, instrument_or_state, pitch_or_direction } => { warn!("Block Action"); },
                        Packet::Explosion { x, y, z, radius, explosion_data } => { warn!("Explosion"); },
                        Packet::SoundEffect { effect_id, x, y, z, data } => { warn!("Sound effect"); },
                        Packet::BedWeatherState { state_reason } => { warn!("Weather State or Bed"); },
                        Packet::ThunderBolt { entity, unk_flag, x, y, z } => { warn!("Thunder Bolt"); },
                        Packet::OpenContainerWindow { window_id, inventory_type, title, slot_count } => {  },
                        Packet::CloseContainerWindow { window_id } => {  },
                        Packet::ClickContainerWindow { window_id, slot, right_click, action, shift, item_id, item_count, item_uses } => {  },
                        Packet::SetContainerSlot { window_id, slot, item_data } => { warn!("Set Slot Item"); },
                        Packet::SetWindowItems { window_id, window_data } => { warn!("Set Window Item"); },
                        Packet::UpdateProgressBar { window_id, progress_bar, value } => {  },
                        Packet::Transaction { window_id, action_id, accepted } => {  },
                        Packet::UpdateSign { x, y, z, line_1, line_2, line_3, line_4 } => {  },
                        Packet::ItemData { item_type, item_id, item_data } => { warn!("Item Data"); },
                        Packet::IncrementStatistic { statistic_id, amount } => { warn!("Updating Statistic"); },
                        Packet::DisconnectKick { reason } => { warn!("Disconnected: {reason}, stopping connection.");}
                    }
                } // Packet parsing
            }
        }
        // info!("Time now: {:?}", tick_time_now);            // tick_time += one_twentieth;
        tick_time = tick_time_now;
        
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

        if event_helper.update(&event)  {
            if event_helper.quit() {
                warn!("Stopping!");
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

            // if let WindowEvent::Resized(size) = event {
            //     client.resize((*size).into());
            // }



        // }
        // if let Event::MainEventsCleared = event {
 
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
                        // info!("Chunks To Tessellate/Generate: {}", tessellate_queue.len());
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
