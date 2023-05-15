
pub struct ServerPlayer {
    pub is_local : bool,
    pub username : String,
    pub uuid: u64,
}

impl ServerPlayer {

    pub fn new() -> Self {
        Self {
            is_local: true,
            username: "TestPlayer001".to_string(),
            uuid: 1247,
        }
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn uuid(&self) -> u64 {
        self.uuid
    }

    pub fn is_local(&self) -> bool {
        return self.is_local;
    }

    pub fn is_remote(&self) -> bool {
        return !self.is_local;
    } 
}
