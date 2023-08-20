use crate::world::WorldUpdate;

pub struct ClientHandler {}

impl ClientHandler {
    pub fn new() -> ClientHandler {
        ClientHandler {}
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn get_updates(&self) -> Vec<WorldUpdate> {
        todo!()
    }

    pub fn send_updates(&mut self, updates: &Vec<WorldUpdate>) {
        todo!()
    }
}
