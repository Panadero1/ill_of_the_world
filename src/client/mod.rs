//! the client serves as the interface with the user
//!
//! it handles all graphics calls
//! the server module still interfaces through the client module

use std::time::Instant;

use crate::{graphics, server::Server};

use self::state::State;

mod draw;
mod state;

pub fn run() {
    let mut server = Server::new(1);
    let mut last = Instant::now();
    server.update();
    println!("time with 1 thread: {:?}", (Instant::now() - last).as_micros());
    
    let mut server = Server::new(2);
    last = Instant::now();
    server.update();
    println!("time with 2 threads: {:?}", (Instant::now() - last).as_micros());

    // pollster::block_on(graphics::run(Box::new(State::new())));
}
