//! the client serves as the interface with the user
//!
//! it handles all graphics calls
//! the server module still interfaces through the client module

use std::time::Instant;

use crate::{graphics, server::ServerHandle};

use self::state::State;

mod draw;
mod state;

pub fn run() {
    pollster::block_on(graphics::run(Box::new(State::new())));
}
