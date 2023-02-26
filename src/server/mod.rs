use log::{info, warn};

use crate::{world::dimension::Dimension, identifier::Identifier};

use self::{player_handler::PlayerConnectionHandler, server_player::ServerPlayer};

pub mod player_handler;
pub mod server_player;

pub enum ServerType {
    // Standalone server without a client
    Dedicated,
    // Stand-in for a dedicated server on a client
    Remote,
    // Server that interacts directly with a client
    Integrated,
}

pub struct MinecraftServer {
    dimensions: Vec<Dimension>,
    players: PlayerConnectionHandler,
    server_type: ServerType,
}

impl MinecraftServer {
    pub fn new(server_type: ServerType) -> Self {
        let dimensions = vec![];
        let players = PlayerConnectionHandler::new();
        
        Self {
            dimensions,
            players,
            server_type,
        }
    }

    pub fn connect_player(&mut self, player: ServerPlayer) {
        warn!("Player connected: {}", player.username());
        self.players.add_player(player);
    }

    pub fn tick(&mut self) {
        for dimension in &mut self.dimensions {
            dimension.tick();
        }
    }

    pub fn run_on_dimension<F: FnOnce(&Dimension)>(&self, identifier: &Identifier, f: F) -> bool {
        for dimension in &self.dimensions {
            if dimension.identifier == *identifier {
                f(dimension);
                return true;
            }
        }
        return false;
    }

    pub fn run_on_dimension_mut<F: FnOnce(&mut Dimension)>(&mut self, identifier: &Identifier, f: F) -> bool {
        for dimension in &mut self.dimensions {
            if dimension.identifier == *identifier {
                f(dimension);
                return true;
            }
        }
        return false;
    }

    pub fn dimensions(&self) -> &Vec<Dimension> {
        &self.dimensions
    }

    pub fn dimensions_mut(&mut self) -> &mut Vec<Dimension> {
        &mut self.dimensions
    }
}
