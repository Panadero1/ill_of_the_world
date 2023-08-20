use std::{
    sync::mpsc::{self, channel, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

use crate::world::{position, World, WorldUpdate};

use self::networking::ClientHandler;

mod networking;

pub struct ServerHandle {
    send: Sender<()>,
    jh: Option<JoinHandle<()>>,
}

impl ServerHandle {
    pub fn start() -> ServerHandle {
        let (send, recv) = channel();

        let server = Server::new();

        let jh = thread::spawn(move || server.run(recv));

        ServerHandle { send, jh: Some(jh) }
    }

    pub fn stop(&mut self) {
        if let Some(jh) = self.jh.take() {
            self.send.send(()).expect("could not tell server to stop");
            jh.join().expect("could not stop server thread");
        }
    }
}

struct Server {
    /// for all block data that the client sees
    blocks: World,
    /// for all internally tracked data. Doesn't need to be sent
    /// out so it's stored separately for cache efficiency
    states: World,
    client_handler: ClientHandler,
}

impl Server {
    fn new() -> Server {
        Server {
            blocks: World::generate(),
            states: World::empty(),
            client_handler: ClientHandler::new(),
        }
    }

    fn process_update(&mut self, update: WorldUpdate, updates_to_send: &mut Vec<WorldUpdate>) {
        todo!()
    }

    fn update_world(&mut self, updates_to_send: &mut Vec<WorldUpdate>) {
        todo!()
    }

    fn run(mut self, recv: Receiver<()>) {
        let mut updates_to_send = Vec::new();
        // loop until told to stop
        while let Err(TryRecvError::Empty) = recv.try_recv() {
            updates_to_send.clear();
            // 1. take in client requests from separate client handlers
            // accept new connections
            self.client_handler.update();
            let updates = self.client_handler.get_updates();

            // 2. update world based on requests
            for update in updates {
                self.process_update(update, &mut updates_to_send);
            }

            // 3. perform one world tick (may need to be separated into sections to speed up)
            self.update_world(&mut updates_to_send);
            // 4. send updates to clients
            self.client_handler.send_updates(&updates_to_send);
        }
    }
}
