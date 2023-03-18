mod test_world;

use std::{collections::VecDeque, sync::{Arc, RwLock, mpsc, atomic::AtomicBool, Mutex}, borrow::BorrowMut, ops::{DerefMut, Deref} };
use env_logger::Builder;
use log::{LevelFilter, warn};
use orange_rs::{
    registry::Registry,
    client::{
        minecraft_client::MinecraftClient, 
        rendering::{
            ElapsedTime, 
            State, 
            tessellator::TerrainTessellator
        }, 
        camera::CameraControllerMovement, 
        Client, 
        gui::screen::{Screen, MainMenu}
    }, 
    util::{
        pos::{
            ChunkPos,
            Position,
            EntityPos,
            BlockPos
        },
        workers::WorkerThread
    },
    packets::prot14::Packet,
    entities::{EntityTransform, EntityMotion, EntityController, EntityCamera},
    minecraft::mc_resource_handler,

};
use orange_networking::{network_interface::NetworkThread};
use ultraviolet::{DVec3, IVec3, Vec3};
use winit::event::{DeviceEvent, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;
use orange_rs::util::pos::NewChunkPosition;
use orange_rs::world::{ChunkStorageTrait};
use crate::test_world::TestWorld;

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
    UnexpectedPacket(String),
    Kick(String),
}

fn join_server(username: String, protocol_id: i32, address: String, port: u32, world: &mut TestWorld) -> Result<NetworkThread<Packet>, ServerConnectError> {
    let network_thread = match NetworkThread::connect_to_server(address, port) {
        Ok(nt) => { nt },
        Err(_) => { return Err(ServerConnectError::InvalidAddress); }
    };

    network_thread.send_packet(Packet::Handshake { handshake_data: username.clone() });
    let mut player_id: i32 = 0;
    world.player = Some(world.entities.push((EntityTransform { position: EntityPos::zero(), rotation: Vec3::zero() }, EntityMotion { velocity: Vec3::zero() }, EntityController { on_ground: true, stance: 1.6 }, EntityCamera { } )));
    
    let mut do_login = true;
    while do_login {
        for packet in network_thread.get_packets() {
            match packet {
                Packet::Handshake { handshake_data } => {
                    let login_packet = Packet::Login{ protocol: protocol_id, username: username.clone(), seed: 0, dimension: 0 };
                    network_thread.send_packet(login_packet); 
                    warn!("Handshake Packet Recieved! {handshake_data}, sending login request as {username}."); 
                },
                Packet::Login { protocol, username, seed, dimension } => {
                    player_id = protocol;
                    world.set_dimension_id(dimension);
                    world.set_seed(seed);
                    do_login = false;
                    break;
                },
                Packet::DisconnectKick { reason } => { return Err(ServerConnectError::Kick(reason)); }
                _ => { return Err(ServerConnectError::UnexpectedPacket(format!("{:?}", packet))); }
            }
        }
    }
    warn!("Logged in, leaving the login sequence.");
    Ok(network_thread)
}

fn main() {

    // env_logger::init();
    Builder::new().filter_level(LevelFilter::Warn).init();

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

    // The tessellator to be used to mesh the chunks, intended for multithreaded usage (TODO)
    let shared_tessellator = Arc::new(RwLock::new(TerrainTessellator::new()));

    let mut tessellate_queue = VecDeque::<IVec3>::new();
 
    // The height of the world in chunk sections, to be used to provide compatibility with anvil
    // and mcregion world types; but may also provide ability to configure a custom world height
    let chunk_height: usize = 8;

    let mut minecraft = MinecraftClient::new(chunk_height as u32);
    minecraft.set_screen::<MainMenu>();
    
    let username = String::from("TT3");
    let address = "127.0.0.0".to_string();
    let port = 25565;
    let mut test_world = TestWorld::new(chunk_height);
    let mut network_thread = match join_server(username, 14, address, port, &mut test_world) {
        Ok(network_thread) => {
            network_thread
        },
        Err(e) => {
            warn!("Failed to connect to server: {:?}", e);
            return;
        },
    };
    let test_world = Arc::new(RwLock::new(test_world));
    let mut tick_time = instant::Instant::now();
    let one_twentieth = instant::Duration::from_secs_f64(1.0 / 20.0);
    let test_world_copy = test_world.clone();
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

        if let Ok(mut test_world) = test_world_copy.write() {
            test_world.tick();
            let (stance, on_ground) = if let Some(controller) = test_world.get_player_controller() {
                (controller.stance, controller.on_ground)
            } else { (-1.6, false) };

            let player_entity = test_world.player.clone().unwrap();
            if let Some(mut entry) = test_world.entities.entry(player_entity) {
               if let Ok(transform) = entry.get_component_mut::<EntityTransform>()  {
                   if !on_ground {
                       transform.position += (0.0, -0.04, 0.0).into();
                   }
               }
            }
            if let Some(transform) = test_world.get_player_transform() {
                let (x, y, z) = transform.position.into();
                let (yaw, pitch) = (transform.rotation.x, transform.rotation.y);
                network_thread.send_packet(Packet::PlayerPositionAndLook { x: x as f64, y_c_stance_s: y as f64 - stance, stance_c_y_s: y as f64, z: z as f64, yaw: yaw as f32, pitch: pitch as f32, on_ground });
            }
            network_thread.send_packet(Packet::KeepAlive);
            for packet in network_thread.get_packets() {
                match packet {
                    Packet::KeepAlive => { network_thread.send_packet(Packet::KeepAlive {}); },
                    Packet::Handshake { handshake_data } => { warn!("Unexpectedly received a handshake packet! This is not supposed to happen after login!"); },
                    Packet::Login { protocol, username, seed, dimension } => { warn!("Unexpectedly received a login packet! This is not supposed to happen after login!"); },
                    Packet::Chat { chat_data } => { warn!("[Chat]{chat_data}"); },
                    Packet::TimeUpdate { time } => { test_world.set_time(time); },
                    Packet::EntityChangeEquipment { entity_id, equipment_slot, item_id, item_damage } => {
                        // warn!("Entity Change Equipment");
                    },
                    Packet::SpawnPosition { x, y, z } => { test_world.set_spawn_point(BlockPos::new(x, y, z)); },
                    Packet::InteractWithEntity { user, entity, is_left_click } => {
                        // warn!("Interact with entity");
                    },
                    Packet::UpdateHealth { health } => { if health == 0 { network_thread.send_packet(Packet::Respawn { world: test_world.get_dimension_id() }); } },
                    Packet::Respawn { world } => { test_world.set_dimension_id(world); }, // leave the respawn
                    Packet::PlayerOnGround { on_ground } => { test_world.set_player_on_ground(on_ground); },
                    Packet::PlayerPosition { x, y, stance, z, on_ground } => {
                        // warn!("Player Position packet");
                    },
                    Packet::PlayerLook { yaw, pitch, on_ground } => {
                        // warn!("Player Look packet");
                    },
                    Packet::PlayerPositionAndLook { x, y_c_stance_s, stance_c_y_s, z, yaw, pitch, on_ground } => {
                        // warn!("Received Stance: {stance_c_y_s}, received y: {y_c_stance_s}");
                        test_world.set_player_position(EntityPos::new(x as f32, y_c_stance_s as f32, z as f32));
                        test_world.set_player_look(Vec3::new(yaw, pitch, 0.0));
                        test_world.set_player_on_ground(on_ground);
                        test_world.set_player_stance(y_c_stance_s - stance_c_y_s);
                        network_thread.send_packet(Packet::PlayerPositionAndLook { x, y_c_stance_s: stance_c_y_s, stance_c_y_s: y_c_stance_s, z, yaw, pitch, on_ground });
                    },
                    Packet::PlayerDigging { status, x, y, z, face } => {
                        // warn!("Player Digging: {status}");
                    },
                    Packet::PlayerUse { x, y, z, direction, item_data } => {
                        // warn!("Player Use");
                    },
                    Packet::PlayerChangeSlot { slot } => {
                        // warn!("Player Change Slot");
                    },
                    Packet::PlayerUseBed { entity, in_bed, x, y, z } => {
                        // warn!("Player Use Bed");
                    },
                    Packet::Animation { entity, animat } => {
                        // warn!("Animation");
                    },
                    Packet::EntityAction { entity, action } => {
                        // warn!("Entity Action");
                    },
                    Packet::NamedEntitySpawn { entity, name, x, y, z, rotation, pitch, held_item } => {
                        // warn!("{name} spawned");
                    },
                    Packet::PickupSpawn { entity, item, count, damage_meta, x, y, z, rotation, pitch, roll } => {
                        // warn!("Pickup Spawned");
                    },
                    Packet::CollectItem { item_entity, collector_entity } => {
                        // warn!("Collect Item");
                    },
                    Packet::CreateNonMobEntity { entity, entity_type, x, y, z, unknown } => {
                        // warn!("Create NonMob Entity");
                    },
                    Packet::SpawnMob { entity, entity_type, x, y, z, yaw, pitch, meta } => {
                        // warn!("Spawn Mob");
                    },
                    Packet::EntityPaintings { entity, title, x, y, z, direction } => {
                        // warn!("Entity Painting {title}");
                    },
                    Packet::UpdatePosition { strafe, forward, pitch, yaw, unk, is_jumping } => {
                        // warn!("UpdatePosition");
                    },
                    Packet::EntityVelocity { entity, vel_x, vel_y, vel_z } => {
                        // warn!("Entity Velocity");
                    },
                    Packet::DestroyEntity { entity } => {
                        // warn!("Destroy Entity");
                    },
                    Packet::Entity { entity } => {
                        // warn!("Spawn {entity}");
                    },
                    Packet::EntityMoveRelative { entity, dx, dy, dz } => {
                        // warn!("Entity Move Rel");
                    },
                    Packet::EntityLook { entity, yaw, pitch } => {
                        // warn!("Entitiy Look");
                    },
                    Packet::EntityLookMoveRelative { entity, dx, dy, dz, yaw, pitch } => {
                        // warn!("Entity Move Look");
                    },
                    Packet::EntityTeleport { entity, x, y, z, yaw, pitch } => {
                        // warn!("Entity Teleport");
                    },
                    Packet::EntityStatus { entity, status } => {
                        // warn!("Entity Status");
                    },
                    Packet::AttachEntity { entity, vehicle_entity } => {
                        // warn!("Attach Entity");
                    },
                    Packet::EntityMeta { entity, meta } => {
                        // warn!("Entity Meta");
                    },
                    Packet::PreChunk { x, z, mode } => {
                    },
                    Packet::MapChunk { x, y, z, size_x, size_y, size_z, compressed_data } => {
                        test_world.handle_map_chunk(x, y as i32, z, size_x, size_y, size_z, compressed_data);
                    },
                    Packet::MultiBlockChange { chunk_x, chunk_z, coords_type_metadata_array } => {
                        // warn!("Multi Block Change");
                    },
                    Packet::BlockChange { x, y, z, block_type, metadata } => {
                        // warn!("Block Change");
                    },
                    Packet::BlockAction { x, y, z, instrument_or_state, pitch_or_direction } => {
                        // warn!("Block Action");
                    },
                    Packet::Explosion { x, y, z, radius, explosion_data } => {
                        // warn!("Explosion");
                    },
                    Packet::SoundEffect { effect_id, x, y, z, data } => {
                        // warn!("Sound effect");
                    },
                    Packet::BedWeatherState { state_reason } => {
                        // warn!("Weather State or Bed");
                    },
                    Packet::ThunderBolt { entity, unk_flag, x, y, z } => {
                        // warn!("Thunder Bolt");
                    },
                    Packet::OpenContainerWindow { window_id, inventory_type, title, slot_count } => {  },
                    Packet::CloseContainerWindow { window_id } => {  },
                    Packet::ClickContainerWindow { window_id, slot, right_click, action, shift, item_id, item_count, item_uses } => {  },
                    Packet::SetContainerSlot { window_id, slot, item_data } => {
                        // warn!("Set Slot Item");
                    },
                    Packet::SetWindowItems { window_id, window_data } => {
                        // warn!("Set Window Item");
                    },
                    Packet::UpdateProgressBar { window_id, progress_bar, value } => {  },
                    Packet::Transaction { window_id, action_id, accepted } => {  },
                    Packet::UpdateSign { x, y, z, line_1, line_2, line_3, line_4 } => {  },
                    Packet::ItemData { item_type, item_id, item_data } => {
                        // warn!("Item Data");
                    },
                    Packet::IncrementStatistic { statistic_id, amount } => {
                        // warn!("Updating Statistic");
                    },
                    Packet::DisconnectKick { reason } => {
                        // warn!("Disconnected: {reason}, stopping connection.");
                    }
                }
            } // Packet parsing
        // }
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
            if event_helper.close_requested() || event_helper.destroyed() {
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
            if event_helper.key_held(VirtualKeyCode::G) {
                // warn!("Chunk Count: {}", test_world.read().unwrap().chunks.chunks().len());
                warn!("Player Position: {:?}", client.camera.position);
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
                let player_pos = match test_world.read().unwrap().get_player_transform() {
                    Some(transform) => {
                        transform.position 
                    },
                    _ => { EntityPos::zero() }
                };
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
                    minecraft.process_chunks(min_extent.clone(), max_extent.clone());
                    
                    let registry = registry.read().unwrap();
                    let blocks = registry.get_block_register();
                    // The maximum number of tessellations to be done every frame
                    let max_tesselations = 2;
                    let mut num_tessellations = 0;
                    if let Ok(server_world) = test_world.read() {
                        let mut tessellator = shared_tessellator.write().unwrap();
                        for x in min_extent.x..=max_extent.x {
                            for z in min_extent.y..=max_extent.y {
                                for y in 0..server_world.get_height() {
                                    let pos = NewChunkPosition::new(x, y as i32, z);
                                    match server_world.chunk_storage.get_chunk(pos.vec) {
                                        Ok(chunk) if chunk.is_dirty() => {
                                            num_tessellations += 1;
                                            let chunk_block_pos = pos.to_block_pos();
                                            let section_position = chunk_block_pos.to_entity_pos();
                                            let section_index = pos.vec.y as usize;
                                            tessellator.tessellate_chunk_section(chunk, section_position, blocks);
                                            let mesh = tessellator.build(&client.gpu.device);
                                            minecraft.world_render.set_section_mesh(mesh, pos.to_chunk_pos(), section_index);
                                            tessellate_queue.push_back(pos.vec);
                                        },
                                        _ => {}
                                    };
                                    if num_tessellations > max_tesselations { break; }
                                }
                                if num_tessellations > max_tesselations { break; }
                            }
                            if num_tessellations > max_tesselations { break; }
                        }
                    }
                    if tessellate_queue.len() > 0 {
                        if let Ok(mut server_world) = test_world.write() {
                            for pos in &tessellate_queue {
                                if let Ok(chunk) = server_world.chunk_storage.get_chunk_mut(*pos) {
                                    chunk.set_dirty(false);
                                }
                            }
                        }
                    }
                    tessellate_queue.clear();
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
