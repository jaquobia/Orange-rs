use std::io::Read;
use legion::EntityStore;
use orange_networking::network_interface::NetworkThread;
use orange_rs::minecraft::prot14::generate_block_to_state_map;
use orange_rs::minecraft::registry::Registry;
use orange_rs::util::nibble;
use ultraviolet::{IVec2, IVec3, Vec3};
use orange_rs::entities::{EntityController, EntityTransform};
use orange_rs::packets::prot14::{MultiBlockChangeData, Packet};
use orange_rs::util::pos::{BlockPos, EntityPos, NewChunkPosition};
use orange_rs::world::chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE, TBlockData};
use orange_rs::world::{ChunkStorage, ChunkStoragePlanar, ChunkStorageTrait};
use rustc_hash::FxHashMap as HashMap;

pub struct TestWorld {
    height: usize,
    time: u64,
    spawn_position: BlockPos,
    dimension_id: i8,
    seed: i64,
    has_weather: bool,
    pub chunk_storage: ChunkStorage<Chunk>,
    pub entities: legion::World,

    pub player: Option<legion::Entity>,

    block_to_state_map: HashMap<u16, usize>,
}

impl TestWorld {
    pub fn new(height: usize, registry: &Registry) -> Self {
        let entity_world = legion::World::default();
        let block_to_state_map = generate_block_to_state_map(registry);

        Self {
            height,
            time: 0,
            spawn_position: BlockPos::new(0, 0, 0),
            dimension_id: 0,
            seed: 0,
            has_weather: false,
            chunk_storage: ChunkStorage::Planar(ChunkStoragePlanar::new(height)),
            entities: entity_world,
            player: None,
            block_to_state_map,
        }
    }

    pub fn set_time(&mut self, time: u64) {
        self.time = time;
    }

    pub fn set_spawn_point(&mut self, spawn_position: BlockPos) {
        self.spawn_position = spawn_position;
    }

    pub fn set_dimension_id(&mut self, id: i8) {
        self.dimension_id = id;
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = seed;
    }

    pub fn set_weather(&mut self, has_weather: bool) {
        self.has_weather = has_weather;
    }

    pub fn get_time(&self) -> u64 {
        self.time
    }

    pub fn get_spawn_point(&self) -> BlockPos {
        self.spawn_position.clone()
    }

    pub fn get_dimension_id(&self) -> i8 {
        self.dimension_id
    }

    pub fn get_seed(&self) -> i64 {
        self.seed
    }

    pub fn get_weather(&self) -> bool {
        self.has_weather
    }

    pub fn tick(&mut self, network_thread: &NetworkThread<Packet>) {
        let (stance, on_ground) = if let Some(controller) = self.get_player_controller() {
            (controller.stance, controller.on_ground)
        } else { (-1.6, false) };

        let player_entity = self.player.clone().unwrap();
        if let Some(mut entry) = self.entities.entry(player_entity) {
            if let Ok(transform) = entry.get_component_mut::<EntityTransform>()  {
                if !on_ground {
                    transform.position += (0.0, -0.04, 0.0).into();
                }
            }
        }
        if let Some(transform) = self.get_player_transform() {
            let (x, y, z) = transform.position.into();
            let (yaw, pitch) = (transform.rotation.x, transform.rotation.y);
            network_thread.send_packet(Packet::PlayerPositionAndLook { x: x as f64, y_c_stance_s: y as f64 - stance, stance_c_y_s: y as f64, z: z as f64, yaw: yaw as f32, pitch: pitch as f32, on_ground });
        }
        network_thread.send_packet(Packet::KeepAlive);
        for packet in network_thread.get_packets() {
            match packet {
                Packet::KeepAlive => { network_thread.send_packet(Packet::KeepAlive {}); },
                Packet::Handshake { handshake_data } => { log::warn!("Unexpectedly received a handshake packet! This is not supposed to happen after login!"); },
                Packet::Login { protocol, username, seed, dimension } => { log::warn!("Unexpectedly received a login packet! This is not supposed to happen after login!"); },
                Packet::Chat { chat_data } => { log::warn!("[Chat]{chat_data}"); },
                Packet::TimeUpdate { time } => { self.set_time(time); },
                Packet::EntityChangeEquipment { entity_id, equipment_slot, item_id, item_damage } => {
                    // warn!("Entity Change Equipment");
                },
                Packet::SpawnPosition { x, y, z } => { self.set_spawn_point(BlockPos::new(x, y, z)); },
                Packet::InteractWithEntity { user, entity, is_left_click } => {
                    // warn!("Interact with entity");
                },
                Packet::UpdateHealth { health } => { if health == 0 { network_thread.send_packet(Packet::Respawn { world: self.get_dimension_id() }); } },
                Packet::Respawn { world } => { self.set_dimension_id(world); }, // leave the respawn
                Packet::PlayerOnGround { on_ground } => { self.set_player_on_ground(on_ground); },
                Packet::PlayerPosition { x, y, stance, z, on_ground } => {
                    // warn!("Player Position packet");
                },
                Packet::PlayerLook { yaw, pitch, on_ground } => {
                    // warn!("Player Look packet");
                },
                Packet::PlayerPositionAndLook { x, y_c_stance_s, stance_c_y_s, z, yaw, pitch, on_ground } => {
                    // warn!("Received Stance: {stance_c_y_s}, received y: {y_c_stance_s}");
                    self.set_player_position(EntityPos::new(x as f32, y_c_stance_s as f32, z as f32));
                    self.set_player_look(Vec3::new(yaw, pitch, 0.0));
                    self.set_player_on_ground(on_ground);
                    self.set_player_stance(y_c_stance_s - stance_c_y_s);
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
                    self.handle_map_chunk(x, y as i32, z, size_x, size_y, size_z, compressed_data);
                },
                Packet::MultiBlockChange { chunk_x, chunk_z, coords_type_metadata_array } => {
                    // warn!("Multi Block Change");
                    self.set_blocks(chunk_x, chunk_z, coords_type_metadata_array);
                },
                Packet::BlockChange { x, y, z, block_type, metadata } => {
                    // warn!("Block Change");
                    self.set_block(x, y as i32, z, block_type as u8, metadata as u8);
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
        }
    }

    fn ecs_set_entity_component<T: Sync + Send + 'static, F>(entity: legion::Entity, world: &mut legion::World, mut f: F) where F: FnMut(&mut T) {
        if let Some(mut entry) = world.entry(entity) {
            if let Ok(component) = entry.get_component_mut::<T>() {
                f(component);
            }
        }
    }

    fn ecs_get_entity_component<T: Sync + Send + 'static + Clone>(entity: legion::Entity, world: &legion::World) -> Option<T> {
        match world.entry_ref(entity) {
            Ok(entry) => {
                match entry.get_component::<T>() {
                    Ok(component) => { let ret = (*component).clone(); Some(ret) },
                    _ => { None },
                }
            },
            _ => None,
        }
    }

    pub fn set_player_position(&mut self, position: EntityPos) {
        // Self::ecs_set_entity_component::<EntityTransform, _>(self.player.unwrap(), &mut self.entities, |transform: &mut EntityTransform| { transform.position = position; });
        let mut entry = self.entities.entry_mut(self.player.unwrap()).unwrap();
        match entry.get_component_mut::<EntityTransform>() {
            Ok(transform) => {
                transform.position = position;
            },
            _ => {},
        }
    }

    pub fn set_player_look(&mut self, look: Vec3) {
        Self::ecs_set_entity_component::<EntityTransform, _>(self.player.unwrap(), &mut self.entities, |transform| { transform.rotation = look; });
    }

    pub fn set_player_on_ground(&mut self, on_ground: bool) {
        Self::ecs_set_entity_component::<EntityController, _>(self.player.unwrap(), &mut self.entities, |controller| { controller.on_ground = on_ground; });
    }

    pub fn set_player_stance(&mut self, stance: f64) {
        let mut entry = self.entities.entry_mut(self.player.unwrap()).unwrap();
        match entry.get_component_mut::<EntityController>() {
            Ok(controller) => {
                controller.stance = stance;
            },
            _ => {},
        }

    }

    pub fn get_player_transform(&self) -> Option<EntityTransform>{
        // Self::ecs_get_entity_component(self.player.unwrap(), &self.entities)
        match self.entities.entry_ref(self.player.unwrap()) {
            Ok(entry) => {
                match entry.get_component::<EntityTransform>() {
                    Ok(transform) => {
                        let ret = transform.clone();
                        Some(ret)
                    },
                    _ => { None }
                }
            },
            _ => { None }
        }
    }

    pub fn get_player_controller(&self) -> Option<EntityController> {
        // Self::ecs_get_entity_component(self.player.unwrap(), &self.entities)
        match self.entities.entry_ref(self.player.unwrap()) {
            Ok(entry) => {
                match entry.get_component::<EntityController>() {
                    Ok(controller) => { let ret = controller.clone(); Some(ret) },
                    _ => None,
                }
            },
            _ => { None }
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: u8, meta: u8) {
        let cpos = (x >> 4, y >> 4, z >> 4);
        match self.chunk_storage.get_chunk_mut(cpos.into()) {
            Ok(chunk) => {
                let block_data = block as u16 | ((meta as u16) << 8);
                let (ix, iy, iz) = (x & 15, y  & 15, z & 15);

                let block_data = match self.block_to_state_map.get(&block_data) {
                    Some(state) => *state,
                    _ => { log::error!("Failed to find id: {}|{}", block_data & 0b11111111, block_data >> 8); return; }
                };


                // warn!("Block Change 2: ({ix}, {iy}, {iz})|({:?}) <- {block}|{meta}", cpos);
                chunk.set_block_at_pos(ix as u32, iy as u32, iz as u32, block_data as TBlockData);
                chunk.set_dirty(true);
            },
            _ => {}
        }
    }

    pub fn set_blocks(&mut self, cx: i32, cz: i32, data: MultiBlockChangeData) {
        for (index, block) in data.blocks.into_iter().enumerate() {
            let coords = data.coords[index];
            let x = ((coords >> 12) & 0b0000000000001111) as u32;
            let z = ((coords >> 8) & 0b0000000000001111) as u32;
            let y = (coords & 0b0000000011111111) as i32;
            let meta = data.metadata[index];
            // println!("Setting ({x}, {y}, {z})|() <- {block} |{}", data.metadata[index]);
            let block_data = block as u16 | ((meta as u16) << 8);

            let block_data = match self.block_to_state_map.get(&block_data) {
                    Some(state) => *state,
                    _ => { log::error!("Failed to find id: {}|{}", block_data & 0b11111111, block_data >> 8); return; }
                };

            if let Ok(chunk) = self.chunk_storage.get_chunk_mut(IVec3::new(cx, y >> 4, cz)) {
                chunk.set_block_at_pos(x, (y & 15) as u32, z, block_data as TBlockData);
                chunk.set_dirty(true);
            }
        }
    }

    pub fn handle_map_chunk(&mut self, block_x: i32, block_y: i32, block_z: i32, size_x: i8, size_y: i8, size_z: i8, compressed_data: Vec<u8>) {
        let size_x = size_x as usize + 1;
        let size_y = size_y as usize + 1;
        let size_z = size_z as usize + 1;

        // The chunk's position id
        let chunk_x = block_x >> 4;
        let chunk_z = block_z >> 4;

        // The chunk's position as a block position
        let chunk_x_real = chunk_x << 4;
        let chunk_z_real = chunk_z << 4;

        // Starting position as inner chunk coords
        let chunk_x_start = (block_x - chunk_x_real) as u32;
        let chunk_y_start = block_y as u32; // Y should be an offset by nature of minecraft, in the range 0 -> 128
        let chunk_z_start = (block_z - chunk_z_real) as u32;

        let region_size = size_x * size_y * size_z;
        let mut inflater = flate2::read::ZlibDecoder::new(compressed_data.as_slice());
        let expected_size = (region_size * 5) >> 1;
        let meta_start = region_size;
        let block_light_start = (region_size * 3) >> 1;
        let sky_light_start = region_size * 2;
        let mut raw_data = vec![0; expected_size];
        let num_bytes = inflater.read(&mut raw_data).unwrap();

        let block_bytes = &raw_data[0..meta_start];
        let meta_bytes = &raw_data[meta_start..block_light_start];
        let block_light_bytes = &raw_data[block_light_start..sky_light_start];
        let sky_light_bytes = &raw_data[sky_light_start..];

        for y in 0..size_y {

            let actual_y = chunk_y_start + y as u32;
            let chunk_index = (actual_y / CHUNK_SECTION_AXIS_SIZE as u32) as i32;
            let local_y = actual_y % CHUNK_SECTION_AXIS_SIZE as u32;

            let chunk_pos = NewChunkPosition::new(chunk_x, chunk_index, chunk_z);
            let chunk = match self.chunk_storage.get_or_create_chunk(chunk_pos.vec, || { Chunk::create_empty() }) {
                Ok(chunk) => chunk,
                _ => continue,
            };

            for x in 0..size_x {
                for z in 0..size_z {

                    let block_index = y + (z * size_y) + (x * size_y * size_z);
                    let data: u16 = block_bytes[block_index].into();

                    let meta = nibble::nibble_get(meta_bytes, block_index);
                    let block_light = nibble::nibble_get(block_light_bytes, block_index);
                    let sky_light = nibble::nibble_get(sky_light_bytes, block_index);
                    let data = data | ((meta as u16) << 8);

                    let data = match self.block_to_state_map.get(&data) {
                        Some(state) => *state,
                        _ => { log::error!("Failed to find id: {}|{}", data & 0b11111111, data >> 8); continue; }
                    };

                    let x = chunk_x_start + x as u32;
                    let y = local_y as u32;
                    let z = chunk_z_start + z as u32;
                    chunk.set_block_at_pos(x, y, z, data as TBlockData);
                    chunk.set_blocklight_at_pos(x, y, z, block_light);
                    chunk.set_skylight_at_pos(x, y, z, sky_light);
                } // for z
            } // for x
            chunk.set_dirty(true);
        } // for y

        // Dirty Neighbors
        let updated_nearby_chunk_position = [IVec2::new(chunk_x + 1, chunk_z), IVec2::new(chunk_x - 1, chunk_z), IVec2::new(chunk_x, chunk_z + 1), IVec2::new(chunk_x, chunk_z - 1)];
        let a = chunk_y_start as i32 / CHUNK_SECTION_AXIS_SIZE as i32;
        let b = (chunk_y_start as i32 + size_y as i32) / CHUNK_SECTION_AXIS_SIZE as i32;
        for y in a .. b {
            for pos in updated_nearby_chunk_position {
                let pos = IVec3::new(pos.x, y, pos.y);
                let _ = self.chunk_storage.get_chunk_mut(pos).and_then(|chunk| {
                    chunk.set_dirty(true);
                    Ok(())
                });
            }
        }
    }
}
