use std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs, SocketAddr},
    ptr,
    sync::{
        mpsc::{self, Sender, Receiver, TryRecvError},
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
            let mut mgr = ClientManager::new(addr, uc, recv).unwrap();
            mgr.run();
        });

        Ok(ClientManagerHandle { send, jh, updates })
    }

    pub fn stop(self) {
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
    stopper: Receiver<()>,
    recv_c: Arc<Mutex<Receiver<()>>>,
    send: Sender<()>,
    jhs: Vec<JoinHandle<()>>,
}

impl ClientManager {
    fn new(
        addr: SocketAddr,
        updates: Arc<Mutex<Vec<WorldUpdate>>>,
        stopper: Receiver<()>,
    ) -> io::Result<ClientManager> {
        let listener = TcpListener::bind(addr)?;
        let jhs = Vec::new();
        let (send, recv) = mpsc::channel();

        let recv_c = Arc::new(Mutex::new(recv));

        listener.set_nonblocking(true).expect("cannot set listener nonblocking");

        Ok(ClientManager {
            updates,
            listener,
            stopper,
            recv_c,
            send,
            jhs,
        })
    }

    fn run(mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let recv_c = self.recv_c.clone();

                    let jh = thread::spawn(move || {
                        let mut client = ClientConnection::new(stream);

                        loop {
                            match recv_c.lock().unwrap().try_recv() {
                                Ok(_) => break,
                                Err(TryRecvError::Empty) => {
                                    // do work
                                    client.read_from_stream();
                                }
                                Err(TryRecvError::Disconnected) => {
                                    // resolve disconnection
                                    todo!()
                                }
                            }
                        }
                    });

                    self.jhs.push(jh);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // nothing to accept; can check for stop call
                    if self.stopper.try_recv().is_ok() {
                        self.stop();
                        break;
                    }
                },
                Err(e) => panic!("Cannot accept client: {:?}", e),
            }
        }
    }

    fn stop(self) {
        for _ in 0..self.jhs.len() {
            self.send.send(()).expect("couldn't send message to shut down client connectino");
        }

        for jh in self.jhs {
            jh.join().expect("couldn't stop thread");
        }
    }
}
