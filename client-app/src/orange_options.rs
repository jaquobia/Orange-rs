use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct OrangeOptions {
    server_ip: String,
}

impl OrangeOptions {
    pub fn new() -> Self {
        Self { server_ip: String::new() }
    }
    pub fn server_ip(&self) -> &str {
        &self.server_ip
    }
}
