use bytemuck::{Pod, Zeroable};
use log::warn;
use orange_networking::{orange_networking_derive::PacketEnumHolder, packet::{PacketEnumHolder, PacketParseable, PacketParseError}, ByteArray};

#[repr(u8)]
#[derive(Debug, Clone, PacketEnumHolder)]
pub enum Packet {
    KeepAlive = 0,
    Login { protocol: i32, username: String, seed: i64, dimension: i8 } = 0x01,
    Handshake { handshake_data: String } = 0x02,
    Chat { chat_data: String } = 0x03,
    TimeUpdate { time: u64 } = 0x04,
    EntityChangeEquipment { entity_id: i32, equipment_slot: u16, item_id: u16, item_damage: u16, } = 0x05,
    SpawnPosition { x: i32, y: i32, z: i32,} = 0x06,
    InteractWithEntity { user: i32, entity: i32, is_left_click: bool, } = 0x07,
    UpdateHealth { health: i16 } = 0x08,
    Respawn { world: i8 } = 0x09,
    PlayerOnGround { on_ground: bool } = 0x0A,
    PlayerPosition { x: f64, y: f64, stance: f64, z: f64, on_ground: bool, } = 0x0B,
    PlayerLook { yaw: f32, pitch: f32, on_ground: bool, } = 0x0C,
    PlayerPositionAndLook { x: f64, y_c_stance_s: f64, stance_c_y_s: f64, z: f64, yaw: f32, pitch: f32, on_ground: bool } = 0x0D,
    PlayerDigging { status: i8, x: i32, y: i8, z: i32, face: i8 } = 0x0E,
    // Variable data, amount and damage are optional
    PlayerUse { x: i32, y: i8, z: i32, direction: i8, item_data: ItemPacketData } = 0x0F,
    PlayerChangeSlot { slot: i16 } = 0x10,
    PlayerUseBed { entity: i32, in_bed: i8, x: i32, y: i8, z: i32 } = 0x11,
    Animation { entity: i32, animat: i8 } = 0x12,
    EntityAction { entity: i32, action: i8 } = 0x13,
    NamedEntitySpawn { entity: i32, name: String, x: i32, y: i32, z: i32, rotation: i8, pitch: i8, held_item: i16 } = 0x14,
    PickupSpawn { entity: i32, item: i16, count: i8, damage_meta: i16, x: i32, y: i32, z: i32, rotation: i8, pitch: i8, roll: i8 } = 0x15,
    CollectItem { item_entity: i32, collector_entity: i32 } = 0x16,
    // Variable data, unk_x/y/z are all optional
    CreateNonMobEntity { entity: i32, entity_type: i8, x: i32, y: i32, z: i32, unknown: NonMobUnknownData,  } = 0x17,
    // Variable data, meta: find a way to read this, its size is variable
    SpawnMob { entity: i32, entity_type: i8, x: i32, y: i32, z: i32, yaw: i8, pitch: i8, meta: EntityMeta } = 0x18,
    // String max length 13
    EntityPaintings { entity: i32, title: String, x: i32, y: i32, z: i32, direction: i8 } = 0x19,
    UpdatePosition { strafe: f32, forward: f32, pitch: f32, yaw: f32, unk: bool, is_jumping: bool } = 0x1B,
    EntityVelocity { entity: i32, vel_x: i16, vel_y: i16, vel_z: i16 } = 0x1C,
    DestroyEntity { entity: i32 } = 0x1D,
    Entity { entity: i32 } = 0x1E,
    EntityMoveRelative { entity: i32, dx: i8, dy: i8, dz: i8 } = 0x1F,
    EntityLook { entity: i32, yaw: i8, pitch: i8 } = 0x20,
    EntityLookMoveRelative { entity: i32, dx: i8, dy: i8, dz: i8, yaw: i8, pitch: i8 } = 0x21,
    EntityTeleport { entity: i32, x: i32, y: i32, z: i32, yaw: i8, pitch: i8 } = 0x22,
    EntityStatus { entity: i32, status: i8 } = 0x26,
    AttachEntity { entity: i32, vehicle_entity: i32 } = 0x27,
    // Variable data, meta: find a way to read this, its size is variable
    EntityMeta { entity: i32, meta: EntityMeta } = 0x28,
    PreChunk { x: i32, z: i32, mode: bool } = 0x32,
    // Variable data, compressed data is supposed to be a byte array of unknown length
    MapChunk { x: i32, y: i16, z: i32, size_x: i8, size_y: i8, size_z: i8, compressed_data: ByteArray } = 0x33,
    // Variable data, coords_array, type_array, and metadata_array should all be arrays of length
    // num_blocks - all stored in coords_type_metadata_array
    MultiBlockChange { chunk_x: i32, chunk_z: i32, coords_type_metadata_array: MultiBlockChangeData } = 0x34,
    BlockChange { x: i32, y: i8, z: i32, block_type: i8, metadata: i8 } = 0x35,
    BlockAction { x: i32, y: i16, z: i32, instrument_or_state: i8, pitch_or_direction: i8 } = 0x36,
    // Variable data, blocks is an array of set of positions in (byte byte byte) as (x y z) offset
    Explosion { x: f64, y: f64, z: f64, radius: f32, explosion_data: ExplosionData } = 0x3C,
    SoundEffect { effect_id: i32, x: i32, y: i8, z: i32, data: i32 } = 0x3D,
    BedWeatherState { state_reason: i8 } = 0x46,
    ThunderBolt { entity: i32, unk_flag: bool, x: i32, y: i32, z: i32 } = 0x47,
    OpenContainerWindow { window_id: i8, inventory_type: i8, title: String, slot_count: i8 } = 0x64,
    CloseContainerWindow { window_id: i8 } = 0x65,
    ClickContainerWindow { window_id: i8, slot: i16, right_click: bool, action: i16, shift: bool, item_id: i16, item_count: i8, item_uses: i16 } = 0x66,
    // Variable data, item_uses and item_count are optional based on item_id >= 0
    SetContainerSlot { window_id: i8, slot: i16, item_data: ItemPacketData } = 0x67,
    // Variable data, payload is supposed to be an specialized array of item data
    SetWindowItems { window_id: i8, window_data: WindowItemsData } = 0x68,
    UpdateProgressBar { window_id: i8, progress_bar: i16, value: i16 } = 0x69,
    Transaction { window_id: i8, action_id: i16, accepted: bool } = 0x6A,
    UpdateSign { x: i32, y: i16, z: i32, line_1: String, line_2: String, line_3: String, line_4: String } = 0x82,
    // Variable data, ascii text is an array of bytes
    ItemData { item_type: i16, item_id: i16, item_data: ItemAsciiData } = 0x83,
    IncrementStatistic { statistic_id: i32, amount: i8 } = 0xC8,
    DisconnectKick { reason: String } = 0xFF,
}

#[derive(Debug, Clone)]
pub struct ItemAsciiData { map_bytes: Vec<u8> }
impl PacketParseable for ItemAsciiData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        [
        (self.map_bytes.len() as u8).to_packet_bytes(),
        self.map_bytes.to_vec(),
        ].concat()
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        warn!("ItemAsciiData");
        let mut consumed = 0usize;
        let vec_size = match u8::from_packet_bytes(bytes) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        } as usize;
        let bytes = &bytes[consumed..];
        if bytes.len() < vec_size {
            return Err(PacketParseError::NotEnoughData);
        }
        let data = &bytes[0..vec_size];
        consumed += vec_size;
        Ok((Self { map_bytes: data.to_vec() }, consumed ))
    }
}

#[derive(Debug, Clone)]
pub struct WindowItemsData {
    payload: Vec<Option<ItemPacketData>>,
}

impl PacketParseable for WindowItemsData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        let d1 = (self.payload.len() as i16).to_packet_bytes();
        let d2 = self.payload.iter().flat_map(|item| {
            match item {
                Some(data) => { [ 
                    data.id.to_packet_bytes(),
                    data.amount.to_packet_bytes(),
                    data.damage.to_packet_bytes(),
                ].concat() },
                None => { (-1i16).to_packet_bytes() },
            }
        }).collect();

        [ d1, d2 ].concat()
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        warn!("WindowItemData");
        let mut consumed = 0usize;
        let inv_size = match i16::from_packet_bytes(bytes) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        } as usize;
       
        let mut items = vec![];
        for _ in 0..inv_size {
            let item = {
                let id = match i16::from_packet_bytes(&bytes[consumed..]) {
                    Ok((value, size)) => { consumed += size; value },
                    Err(e) => { return Err(e); },
                };

                if id == -1 {
                    None
                } else {
                    let amount = match i8::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, size)) => { consumed += size; value },
                        Err(e) => { return Err(e); },
                    };
                    let damage = match i16::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, size)) => { consumed += size; value },
                        Err(e) => { return Err(e); },
                    };
                    Some(ItemPacketData { id, amount, damage })
                }
            };
            items.push(item);
        }
        warn!("There were {} items", items.len());
        Ok((Self { payload: items }, consumed))
    }
}

#[derive(Debug, Clone)]
pub struct ExplosionData {
    blocks: Vec<ExplosionBlockData>,
}

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct ExplosionBlockData { x: i8, y: i8, z: i8 }

impl PacketParseable for ExplosionData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        let d1 = (self.blocks.len() as i32).to_packet_bytes();
        let d2: Vec<u8> = bytemuck::cast_slice(&self.blocks).to_vec();
        [
            d1, d2,
        ].concat()
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        warn!("ExplosionData");
        let mut consumed = 0usize;
        let blocks_size = match i32::from_packet_bytes(bytes) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        } as usize;
        
        let bytes = &bytes[consumed..];
        let blocks: Vec<ExplosionBlockData> = bytemuck::cast_slice(&bytes[0..blocks_size*3]).to_vec();
        consumed += blocks_size * 3;
        Ok((Self { blocks }, consumed))
    }
}

#[derive(Debug, Clone)]
pub struct NonMobUnknownData {
    unknown: Option<InnerMobUnknownData>,  
}

#[derive(Debug, Clone)]
pub struct InnerMobUnknownData {
    flag_value: i32,
    ukn_x: i16,
    ukn_y: i16,
    ukn_z: i16,
}

impl PacketParseable for NonMobUnknownData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        match &self.unknown {
            Some(inner) => {
                [
                    true.to_packet_bytes(),
                    inner.ukn_x.to_packet_bytes(),
                    inner.ukn_y.to_packet_bytes(),
                    inner.ukn_z.to_packet_bytes(),
                ].concat()
            },
            None => {
                false.to_packet_bytes()
            }
        }
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        warn!("NonMobUnknownData");
        let mut consumed = 0usize;
        let flag = match i32::from_packet_bytes(bytes) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };

        if !(flag > 0) {
            return Ok((Self { unknown: None }, consumed));
        }

        let x = match i16::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        let y = match i16::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        let z = match i16::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        Ok((Self { unknown: Some( InnerMobUnknownData { flag_value: flag, ukn_x: x, ukn_y: y, ukn_z: z }) }, consumed))
    }
}

#[derive(Debug, Clone)]
pub struct ItemPacketData {
    id: i16, 
    amount: i8,
    damage: i16,
}

impl PacketParseable for ItemPacketData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        [
            self.id.to_packet_bytes(),
            self.amount.to_packet_bytes(),
            self.damage.to_packet_bytes(),
        ].concat()
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        warn!("ItemPacketData");
        let mut consumed = 0usize;
        let id = match i16::from_packet_bytes(bytes) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        if id == -1 {
            return Ok((Self { id: -1, amount: 0, damage: 0 }, consumed))
        }
        let amount = match i8::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        let damage = match i16::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        };
        Ok((Self { id, amount, damage }, consumed))
    }
}

#[derive(Debug, Clone)]
pub struct MultiBlockChangeData {
    pub coords: Vec<i16>,
    pub blocks: Vec<i8>,
    pub metadata: Vec<i8>,
}

impl PacketParseable for MultiBlockChangeData {
    fn to_packet_bytes(&self) -> Vec<u8> {
        let coords: &[u8] = bytemuck::cast_slice(&self.coords);
        let blocks: &[u8] = bytemuck::cast_slice(&self.blocks);
        let metadata: &[u8] = bytemuck::cast_slice(&self.metadata);
        [
            (self.coords.len() as i16).to_packet_bytes(),
            coords.to_vec(),
            blocks.to_vec(),
            metadata.to_vec(),
        ].concat()
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), PacketParseError> where Self: Sized {
        let mut consumed = 0usize;
        let blocks_size = match i16::from_packet_bytes(&bytes[consumed..]) {
            Ok((value, size)) => { consumed += size; value },
            Err(e) => { return Err(e); },
        } as usize; 

        let mut coords = vec![0i16; blocks_size];
        let mut blocks = vec![0i8; blocks_size];
        let mut metadata = vec![0i8; blocks_size];

        for i in 0..blocks_size {
            coords[i] = match i16::from_packet_bytes(&bytes[consumed..]) {
                Ok((value, size)) => { consumed += size; value },
                Err(e) => { return Err(e); },
            }
        }
        for i in 0..blocks_size {
            blocks[i] = match i8::from_packet_bytes(&bytes[consumed..]) {
                Ok((value, size)) => { consumed += size; value },
                Err(e) => { return Err(e); },
            }
        }
        for i in 0..blocks_size {
            metadata[i] = match i8::from_packet_bytes(&bytes[consumed..]) {
                Ok((value, size)) => { consumed += size; value },
                Err(e) => { return Err(e); },
            }
        }

        Ok((Self { coords, blocks, metadata }, consumed))
    }
}

#[derive(Debug, Clone)]
pub struct EntityMeta {
    data_list: Vec<EntityMetaType>,
}

#[derive(Debug, Clone)]
enum EntityMetaType {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    Str(String),
    Item(i16, i8, i16),
    Position(i32, i32, i32),
}

impl PacketParseable for EntityMeta {
    fn to_packet_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.data_list.iter().flat_map(|metatype| {
            let data = match metatype {
                EntityMetaType::Byte(b) => { 
                    [
                        (0u8 << 5u8).to_packet_bytes(),
                        b.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Short(s) => {
                    [
                        (1u8 << 5u8).to_packet_bytes(),
                        s.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Int(i) => {
                    [
                        (2u8 << 5u8).to_packet_bytes(),
                        i.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Float(f) => { 
                    [
                        (3u8 << 5u8).to_packet_bytes(),
                        f.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Str(s) => { 
                    [
                        (4u8 << 5u8).to_packet_bytes(),
                        s.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Item(i, s, d) => { 
                    [
                        (5u8 << 5u8).to_packet_bytes(),
                        i.to_packet_bytes(),
                        s.to_packet_bytes(),
                        d.to_packet_bytes(),
                    ].concat()
                },
                EntityMetaType::Position(x, y, z) => { 
                    [
                        (6u8 << 5u8).to_packet_bytes(),
                        x.to_packet_bytes(),
                        y.to_packet_bytes(),
                        z.to_packet_bytes(),
                    ].concat()
                },
            };
            data
        }).collect(); 
        bytes.extend_from_slice(&0x7F.to_packet_bytes());
        bytes
    }
    fn from_packet_bytes(bytes: &[u8]) -> Result<(Self, usize), orange_networking::packet::PacketParseError> where Self: Sized {
        let mut data_list = vec![];
        let mut consumed = 0usize;
       
        loop {

            let metaid: u8 = match u8::from_packet_bytes(&bytes[consumed..]) {
                Ok((value, used)) => { consumed += used; value },
                Err(e) => { return Err(PacketParseError::NotEnoughData); }
            };

            if metaid == 0x7F {
                break;
            }

            let meta = match metaid >> 5 {
                0 => { 
                    let b = match i8::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Byte(b)
                },
                1 => { 
                    let b = match i16::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Short(b)
                },
                2 => { 
                    let b = match i32::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Int(b)
                },
                3 => { 
                    let b = match f32::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Float(b)
                },
                4 => { 
                    let b = match String::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Str(b)
                },
                5 => { 
                    let b = match i16::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    let b1 = match i8::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    let b2 = match i16::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Item(b, b1, b2)
                },
                6 => {
                    let b = match i32::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    }; 
                    let b1 = match i32::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    }; 
                    let b2 = match i32::from_packet_bytes(&bytes[consumed..]) {
                        Ok((value, used)) => { consumed += used; value },
                        Err(e) => { return Err(PacketParseError::NotEnoughData); }
                    };
                    EntityMetaType::Position(b, b1, b2)
                },
                7 => { warn!("There should not be any meta byte id of 7!"); return Err(PacketParseError::NotAPacket); },
                _ => {
                    warn!("Unknown Meta Type {metaid}");
                    return Err(PacketParseError::NotAPacket);
                }
            };
            data_list.push(meta);

        }
        Ok((Self { data_list }, consumed))
    }
}
