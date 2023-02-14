use super::ServerType;

pub struct NetworkInterface {
    pub server_type: ServerType,
}

impl NetworkInterface {

    pub fn new(server_type: ServerType) -> Self {
        Self {
            server_type,
        }
    }



}
