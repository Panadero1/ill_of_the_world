use std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs, SocketAddr},
    ptr,
    sync::{
        mpsc::{self, Sender, Receiver},
        Arc, Mutex,
    },
    task::{RawWaker, Waker},
    thread::{self, JoinHandle},
};

use pollster::FutureExt;

use crate::world::WorldUpdate;

use super::connection::ClientConnection;

const BLOCK_UPDATE_SIZE: usize = 16;

pub struct ClientManagerHandle {
    send: Sender<()>,
    jh: JoinHandle<()>,
    updates: Arc<Mutex<Vec<WorldUpdate>>>,
}

impl ClientManagerHandle {
    pub fn start(addr: SocketAddr) -> io::Result<ClientManagerHandle> {
        let (send, recv) = mpsc::channel();

        let updates = Arc::new(Mutex::new(Vec::new()));

        let uc = updates.clone();

        let jh = thread::spawn(move || {
            let mut mgr = ClientManager::new(addr, uc).unwrap();
            mgr.run();
        });

        Ok(ClientManagerHandle { send, jh, updates })
    }

    pub fn stop(mut self) {
        self.send
            .send(())
            .expect("couldn't send message to client connection thread");

        self.jh
            .join()
            .expect("couldn't stop client connection thread");
    }

    pub fn get_updates(&mut self) -> Vec<WorldUpdate> {
        self.updates.lock().unwrap().drain(..).collect()
    }

    pub fn send_updates(&mut self, updates: &Vec<WorldUpdate>) {
        todo!()
    }
}

struct ClientManager {
    updates: Arc<Mutex<Vec<WorldUpdate>>>,
    listener: TcpListener,
    recv_c: Arc<Receiver<()>>,
    send: Sender<()>,
    clients: Vec<ClientConnection>,
}

impl ClientManager {
    fn new(
        addr: SocketAddr,
        updates: Arc<Mutex<Vec<WorldUpdate>>>,
    ) -> io::Result<ClientManager> {
        let listener = TcpListener::bind(addr)?;
        let clients = Vec::new();
        let (send, recv) = mpsc::channel();

        let recv_c = Arc::new(recv);


        Ok(ClientManager {
            updates,
            listener,
            recv_c,
            send,
            clients,
        })
    }

    fn run(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => (),
                Err(e) => eprintln!("Error connecting client: {:?}", e),
            }
        }
    }
}
