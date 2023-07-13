use super::server_player::ServerPlayer;

/**
 *  Represents the player connection handler
 *
 */
pub struct PlayerConnectionHandler {
    players: Vec<ServerPlayer>,
}

impl PlayerConnectionHandler {
    pub fn new() -> Self {
        let players = vec![];
        Self {
            players,
        }
    }

    pub fn add_player(&mut self, player: ServerPlayer) {
        self.players.push(player);
    }
}
