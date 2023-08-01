use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct OrangeOptions {
    server_ip: String,
    offline_username: String,
}

impl OrangeOptions {
    pub fn new() -> Self {
        Self { server_ip: String::new(), offline_username: "".into() }
    }
    pub fn server_ip(&self) -> &str {
        &self.server_ip
    }
    pub fn offline_username(&self) -> &str {
        &self.offline_username
    }
}
