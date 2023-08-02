mod test_world;
mod orange_options;
mod cli_options;

use std::{collections::VecDeque, sync::{Arc, RwLock}, fs::File, io::{Write, Read}, net::{SocketAddr, Ipv4Addr}, str::FromStr, fmt::Display};
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, warn};
use orange_rs::{
    client::{
        camera::CameraControllerMovement,
        Client,
        gui::screen::MainMenu,
        minecraft_client::MinecraftClient,
        rendering::{
            ElapsedTime,
            tessellator::TerrainTessellator
        }
    },
    entities::{EntityCamera, EntityController, EntityMotion, EntityTransform},
    minecraft::mc_resource_handler,
    packets::prot14::Packet,
    util::{
        pos::{
            ChunkPos,
            EntityPos,
            Position
        },
        workers::WorkerThread
    },

};

use orange_networking::network_interface::NetworkThread;
use rine::RineApplication;
use ultraviolet::{DVec3, IVec3, Vec3};
use winit::event::{DeviceEvent, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;
use orange_rs::minecraft::mc_resource_handler::{CAMERA_BIND_GROUP_NAME, LIGHTMAP_TEXTURE_NAME, TERRAIN_OPAQUE_PIPELINE, TERRAIN_TRANSPARENT_PIPELINE};
use orange_rs::minecraft::registry::Registry;
use orange_rs::util::frustrum::Frustrum;
use orange_rs::util::pos::NewChunkPosition;
use orange_rs::world::ChunkStorageTrait;
use crate::{test_world::TestWorld, orange_options::OrangeOptions};

#[derive(Debug)]
enum ServerConnectError {
    InvalidAddress,
    UnexpectedPacket(String),
    Kick(String),
}

impl std::error::Error for ServerConnectError {}

impl Display for ServerConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
                    let username = if username.len() > 16 {
                        username[0..16].to_string()
                    } else {
                        username.clone()
                    };
                    warn!("Username is length {}", username.len());
                    let login_packet = Packet::Login{ protocol: protocol_id, username: username.clone(), seed: 0, dimension: 0 };
                    network_thread.send_packet(login_packet); 
                    warn!("Handshake Packet Received! {handshake_data}, sending login request as {username}.");
                },
                Packet::Login { protocol, seed, dimension, .. } => {
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

const CHUNK_HEIGHT: usize = 8;

enum GameState {
    MainMenu,
    JoiningServer {
        test_world: Option<Arc<RwLock<TestWorld>>>,
        server_thread: Option<WorkerThread>,
    },
    InGame {
        test_world: Arc<RwLock<TestWorld>>,
        server_thread: WorkerThread,
    },
}

impl GameState {
    pub fn to_main_menu(&mut self) {
        match self {
            Self::MainMenu => { return; },
            Self::InGame { test_world, server_thread } => { *self = Self::MainMenu },
            Self::JoiningServer { test_world, server_thread } => { *self = Self::MainMenu }
        }
    }

    pub fn to_joining_server(&mut self) {
        match self {
            Self::MainMenu => { *self = Self::JoiningServer { test_world: None, server_thread: None } },
            Self::InGame { test_world, server_thread } => { panic!("Cant return to joining server from ingame!"); },
            Self::JoiningServer { test_world, server_thread } => { return; }
        }
    }

    pub fn to_ingame(&mut self) {
        match self {
            Self::MainMenu => { panic!("Can't go inagme from main menu! Needs to go through a joining server phase first!") },
            Self::InGame { test_world, server_thread } => { return; },
            Self::JoiningServer { test_world, server_thread } => { *self = Self::InGame { test_world: test_world.take().unwrap(), server_thread: server_thread.take().unwrap() }; }
        }
    }
}

struct OrangeClient {

    username: String,

    game_state: GameState,

    render_time: ElapsedTime,
    client: Client,
    winit_input_helper: WinitInputHelper,
    minecraft: MinecraftClient,
    registry: Arc<RwLock<Registry>>,
    tessellator: Arc<RwLock<TerrainTessellator>>,
    tessellate_queue: VecDeque<IVec3>,


    server_ip: String,
    server_port: String,
    debug: bool,
}

impl OrangeClient {
    pub fn join_server_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.game_state {
            GameState::MainMenu => {
                self.game_state.to_joining_server();

                let ip_v4 = Ipv4Addr::from_str(&self.server_ip)?;
                let ip_port = self.server_port.parse::<u16>()?;
                let ip = SocketAddr::new(std::net::IpAddr::V4(ip_v4), ip_port);
                let mut test_world = TestWorld::new(CHUNK_HEIGHT, &self.registry.read().unwrap());
                let mut network_thread = join_server(self.username.clone(), 14, ip.ip().to_string(), ip.port() as u32, &mut test_world)?;
                let test_world_o = Arc::new(RwLock::new(test_world));

                let mut server_thread_o = WorkerThread::new();
                let mut tick_time = instant::Instant::now();
                let one_twentieth = instant::Duration::from_secs_f64(1.0 / 20.0);
                let test_world_copy = test_world_o.clone();
                server_thread_o.spawn(move |running| {
                    let tick_time_now = instant::Instant::now(); 

                    if !running {
                        network_thread.stop();
                        return;
                    }

                    if (tick_time_now - tick_time) < one_twentieth {
                        return;
                    }

                    if let Ok(mut test_world) = test_world_copy.write() {
                        test_world.tick(&network_thread);
                    }
                    tick_time = tick_time_now;
                });
                if let GameState::JoiningServer { test_world, server_thread } = &mut self.game_state {
                    test_world.insert(test_world_o);
                    server_thread.insert(server_thread_o);
                }
                self.game_state.to_ingame();
                Ok(())
            },
            GameState::InGame { test_world, server_thread } => {
                return Ok(());
            },
            GameState::JoiningServer { server_thread, test_world } => {
                return Ok(());
            }
        }
    }
    
    pub fn join_server_world(&mut self) {
        match &self.game_state {
            GameState::MainMenu => {
                return;
            },
            GameState::InGame { test_world, server_thread } => {
                return;
            },
            GameState::JoiningServer { test_world, server_thread } => {
                self.game_state.to_ingame();
            }
        }
    }

    pub fn join_server_cancel(&mut self) {
        match &self.game_state {
            GameState::MainMenu => {
                return;
            },
            GameState::InGame { test_world, server_thread } => {
                return;
            },
            GameState::JoiningServer { test_world, server_thread } => {
                self.game_state.to_main_menu();
            }
        }
    }

    pub fn leave_server(&mut self) {
        match &self.game_state {
            GameState::MainMenu => {
                return;        
            },
            GameState::InGame { test_world, server_thread } => {
                self.game_state.to_main_menu();
            },
            GameState::JoiningServer { test_world, server_thread } => {
                return;
            }
        }
    }

    pub fn update(&mut self) {
        // match self.game_state {
        //     GameState::MainMenu => {
        //         return;
        //     },
        //     GameState::InGame => {
        //         return;
        //     },
        //     GameState::JoiningServer => {
        //         return;
        //     }
        // }
    }
}

impl RineApplication for OrangeClient {
    fn create(window_client: &rine::RineWindowClient) -> Self {

        let cli = cli_options::OrangeCliArgs::parse();

        let home_path =  cli.orange_directory.unwrap_or_else(||dirs::data_dir().unwrap().join(".orange"));
        log::warn!("Orange(config) Path: {}", home_path.display());

        if !home_path.exists() {
            match std::fs::create_dir_all(home_path.to_path_buf()) {
                Ok(_) => {  },
                Err(e) => { log::error!("Could not create orange folder! {e}"); }
            }
        }

        let orange_options_path = home_path.join("options.toml");
        let orange_assets_path = home_path.join("assets");

        // Get or default the options
        let orange_options: OrangeOptions = orange_options_path.exists()
            .then(|| { File::open(orange_options_path.to_path_buf()) })
            .map(|f| { let mut s = String::new(); if let Ok(mut f) = f { f.read_to_string(&mut s).unwrap(); } s })
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_else(|| OrangeOptions::new());

        // Write the options out to file for verification and first time creation
        std::fs::write(orange_options_path.to_path_buf(), toml::to_string(&orange_options).unwrap_or(String::new()).as_bytes()).unwrap();

        let ip_params: Vec<&str> = orange_options.server_ip().split(":").collect();
        let param_ip = ip_params.get(0).map_or_else(|| "localhost", |&v| v).to_string();
        let param_port = ip_params.get(1).and_then(|v|v.parse().ok()).unwrap_or(25565);

        let render_time = ElapsedTime::new();
        let mut client = Client::new(window_client.device(), window_client.config(), window_client.window().inner_size());
        let winit_input_helper = WinitInputHelper::new();
        let minecraft = MinecraftClient::new(CHUNK_HEIGHT);
        minecraft.set_screen::<MainMenu>();
        let registry = Arc::new(RwLock::new(Registry::load_from(orange_rs::game_version::GameVersion::B173)));
        {
            let device = window_client.device();
            let queue = window_client.queue();
            let config = window_client.config();
            mc_resource_handler::create_resources(&mut client, device, queue, config);
            mc_resource_handler::load_binary_resources(&mut client, device, queue);
        }

        // The tessellator to be used to mesh the chunks, intended for multithreaded usage (TODO)
        let shared_tessellator = Arc::new(RwLock::new(TerrainTessellator::new()));

        let tessellate_queue = VecDeque::<IVec3>::new();

        let username = cli.username.or_else(|| Some(orange_options.offline_username().to_string()) ).unwrap_or_else(||String::from("Dev"));
        Self {
            username,
            game_state: GameState::MainMenu,
            render_time,
            client,
            winit_input_helper,
            minecraft,
            registry,
            tessellator: shared_tessellator,
            tessellate_queue,
            server_ip: param_ip,
            server_port: param_port.to_string(),
            debug: false
        }
    }

    fn draw(&mut self, window_client: &rine::RineWindowClient, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let device = window_client.device();
        let client = &self.client;

        match &self.game_state {
            GameState::MainMenu => {},
            GameState::JoiningServer { test_world, server_thread } => {},
            GameState::InGame { test_world, server_thread } => {
                let player_pos = match test_world.read().unwrap().get_player_transform() {
                    Some(transform) => {
                        transform.position 
                    },
                    _ => { EntityPos::zero() }
                };

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

                    render_pass.set_pipeline(client.get_pipeline(TERRAIN_OPAQUE_PIPELINE).unwrap());
                    render_pass.set_bind_group(0, client.get_bind_group(CAMERA_BIND_GROUP_NAME).unwrap(), &[]);
                    render_pass.set_bind_group(1, client.get_texture("terrain.png").bind_group(), &[]);
                    render_pass.set_bind_group(2, client.get_texture(LIGHTMAP_TEXTURE_NAME).bind_group(), &[]);

                    let directions = client.camera.vectors();
                    let aspect = client.projection.aspect;
                    let fovy = client.projection.fovy;
                    let znear = client.projection.znear;
                    let zfar = client.projection.zfar;
                    let camera_position = client.camera.position();
                    let frustrum = Frustrum::new(camera_position, directions.0, directions.1, directions.2, aspect, fovy, znear, zfar);

                    // AABB in frustrum culling?
                    let mut render_list: Vec<IVec3> = vec![];
                    let vec16 = Vec3::new(16.0, 16.0, 16.0);
                    for x in min_extent.x..=max_extent.x {
                        for z in min_extent.y..=max_extent.y {
                            for y in 0..CHUNK_HEIGHT as i32 {
                                let chunk_pos_min = Vec3::new((x << 4) as f32, (y << 4) as f32, (z << 4) as f32);
                                let chunk_pos_max = chunk_pos_min + vec16;

                                if !frustrum.aabb_intersects(chunk_pos_min, chunk_pos_max) { continue; }
                                render_list.push(IVec3::new(x, y, z));
                            }
                        }
                    }
                    let camera_pos_i = camera_position.to_block_pos();
                    let vec8 = IVec3::new(8, 8, 8);
                    // Sort by center of chunks; if sorting by min point, chunks to the x+/y+/z+ are likely to be drawn before the chunk of the player
                    render_list.sort_unstable_by(|a, b| {
                        let dist_a = *a + (vec8 * 16) - camera_pos_i;
                        let dist_b = *b + (vec8 * 16) - camera_pos_i;
                        dist_a.mag().cmp(&dist_b.mag())
                    });
                    for chunk_pos in &render_list {
                        if let Ok(mesh) = self.minecraft.client_chunk_storage.get_chunk(*chunk_pos) {
                            mesh.draw(&mut render_pass);
                        }
                    }

                    render_pass.set_pipeline(client.get_pipeline(TERRAIN_TRANSPARENT_PIPELINE).unwrap());
                    render_list.reverse();
                    for chunk_pos in &render_list {
                        if let Ok(mesh) = self.minecraft.client_chunk_storage.get_chunk(*chunk_pos) {
                            mesh.draw_transparent(&mut render_pass);
                        }
                    }
                }
                {

                    let registry = self.registry.read().unwrap();
                    let blocks = registry.get_block_register();
                    let states = registry.get_blockstate_register();
                    let textures = registry.get_texture_register();
                    let models = registry.get_model_register();
                    // The maximum number of tessellations to be done every frame
                    let max_tessellations = 8;
                    // let max_tessellations = 256;
                    let mut num_tessellations = 0;
                    if let Ok(server_world) = test_world.read() {
                        let mut tessellator = self.tessellator.write().unwrap();
                        for x in min_extent.x..=max_extent.x {
                            for z in min_extent.y..=max_extent.y {
                                for y in 0..server_world.get_height() as i32 {
                                    let pos = IVec3::new(x, y, z);
                                    match server_world.chunk_storage.get_chunk(pos) {
                                        Ok(chunk) if chunk.is_dirty() => {
                                            num_tessellations += 1;
                                            let section_position = NewChunkPosition::new(x, y, z).to_entity_pos();

                                            // let nearby_chunks = server_world.chunk_storage.get_nearby_chunks(pos);
                                            tessellator.tessellate_chunk_section(chunk, section_position, pos, blocks, states, models, textures, &server_world.chunk_storage);
                                            let mesh = tessellator.build(device);
                                            self.minecraft.client_chunk_storage.set_chunk(mesh, pos).unwrap();
                                            self.tessellate_queue.push_back(pos);
                                        },
                                        _ => {}
                                    };
                                    if num_tessellations > max_tessellations { break; }
                                }
                            }
                        }
                    }
                    if self.tessellate_queue.len() > 0 {
                        if let Ok(mut server_world) = test_world.write() {
                            for pos in &self.tessellate_queue {
                                if let Ok(chunk) = server_world.chunk_storage.get_chunk_mut(*pos) {
                                    chunk.set_dirty(false);
                                }
                            }
                        }
                    }
                    self.tessellate_queue.clear();
                }
            }
        }


    }

    fn resize(&mut self, size: (u32, u32), window_client: &rine::RineWindowClient) {
        self.client.resize(size, window_client.device(), window_client.config());
    }

    fn handle_event<T>(&mut self, event: &winit::event::Event<T>, control_flow: &mut winit::event_loop::ControlFlow, window_client: &mut rine::RineWindowClient) {
        if let GameState::InGame { test_world, server_thread } = &self.game_state {

            let event_helper = &mut self.winit_input_helper;
            let client = &mut self.client;

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
                use VirtualKeyCode as Key;

                if event_helper.key_held(Key::W) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Forward, true);
                }
                if event_helper.key_held(Key::S) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Backward, true);
                }
                if event_helper.key_held(Key::A) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Left, true);
                }
                if event_helper.key_held(Key::D) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Right, true);
                }
                if event_helper.key_pressed(Key::G) {
                    warn!("Player Position: {:?}", client.camera.position());
                }
                if event_helper.key_pressed(Key::H) {
                    if let Ok(test_world) = test_world.read() {
                        if let Some(transform) = test_world.get_player_transform() {
                            client.camera.set_position(transform.position);
                        }
                    }
                }
                if event_helper.key_pressed(Key::F3) {
                    self.debug = !self.debug;
                }
                if event_helper.key_held(Key::Space) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Up, true);
                }
                if event_helper.key_held(Key::LShift) {
                    client
                        .camera_controller
                        .process_keyboard(CameraControllerMovement::Down, true);
                }
                if event_helper.key_pressed(Key::V) {
                    window_client.toggle_vsync();
                }
                if event_helper.key_pressed(Key::Escape) {
                    client.toggle_cursor_visible(window_client.window());
                }
                if let Some(size) = event_helper.window_resized() {
                    client.resize(size.into(), window_client.device(), window_client.config());
                }

                self.render_time.tick();

                client.update(self.render_time.elapsed_time() as f32, window_client.queue());
            }
        }
    }

    #[cfg(feature = "egui-int")]
    fn egui_draw(&mut self, ctx: &rine::egui::Context) {
        use rine::egui as egui;
        match &self.game_state {
            GameState::InGame { test_world, server_thread } => {
                if !self.debug {
                    return;
                }
                let world = test_world.read();
                if !world.is_ok() { return; }
                let world = world.unwrap();
                let player_pos_real = world.get_player_transform().unwrap().position;
                let player_pos_camera = self.client.camera.position();
                let player_pos_int = player_pos_camera.to_block_pos();
                let player_pos_chunk = (player_pos_int.x >> 4, player_pos_int.y >> 4, player_pos_int.z >> 4).into();
                let player_pos_chunk_inner = player_pos_int.to_inner_chunk_pos().0;
                let player_block = world.chunk_storage.get_chunk(player_pos_chunk).map(|c| c.get_block_at_vec(player_pos_chunk_inner)).unwrap_or(0);
                std::mem::drop(world); // stop borrowing
                let mut leaving_server = false;
                egui::Window::new("Orange Window").auto_sized().show(ctx, |ui| {
                    ui.label(format!("Entity Position: {:.2?}", player_pos_real));
                    ui.label(format!("Camera Position: {:.2?}", player_pos_camera));
                    ui.label(format!("Camera Block Position: {:?}", player_pos_int));
                    ui.label(format!("Camera Chunk: ({:?}, {:?})", player_pos_chunk, player_pos_chunk_inner));
                    ui.label(format!("Block on player: {:?}", player_block));
                    if ui.button("Quit").clicked() {
                        leaving_server = true;
                        return;
                    }
                });
                if leaving_server {
                    self.leave_server();
                }
            },
            GameState::JoiningServer { test_world, server_thread } => {
                egui::Window::new("Orange Window").auto_sized().show(ctx, |ui| {
                    if ui.button("Cancel").clicked() {
                        self.join_server_cancel();
                    }
                });
            },
            GameState::MainMenu => {
                egui::Window::new("Orange Window").auto_sized().show(ctx, |ui| {
                    egui::text_edit::TextEdit::singleline(&mut self.username).char_limit(16).show(ui).response.changed();
                    ui.label("Server Ip:");
                    ui.text_edit_singleline(&mut self.server_ip);
                    egui::text_edit::TextEdit::singleline(&mut self.server_port).char_limit(5).show(ui).response.changed();
                    if ui.button("Join Server").clicked() {
                        self.join_server_connect();
                    } 
                });
            },
        }
    }

}

/**
 * Thigs to work on:  
 * - Uv correction (rotation/uv lock),
 * - Model caching (integration with blockstates), 
 * - Finish models for block, 
 * - Tints for leaves and grass (implement the entire biome system...),
 * - Fix bugs in lighting and ao (artifacts at ll=0/1, ao having sharp corners)
 * - Fix ao with model rotation
 * - Fix rotation of elements/faces on x axis
 * - Implement gamma curve (brightness)
 * - Implement GUIs
 * - Implement Huds
 * - Implement Inventories and Interactable GUI elements
 * - Implement Items
 * - Implement Blockstates
 * - Implement Blockstate model varients by rotations and multipart
 * - Implement ResourceLoader
 * - Generate a default resourcepack
 * - Implement player physics
 * - Implement Entities (models)
 * - Implement Entity movement & physics
 * - Maps
 * - Sky Light (chunk light maps)
 * - Light updates
 *
**/

fn main() {

    // env_logger::init();
    Builder::new().filter_level(LevelFilter::Warn).init();

    rine::start_rine_application::<OrangeClient>();
}
