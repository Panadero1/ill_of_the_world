//! the client serves as the interface with the user
//!
//! it handles all graphics calls
//! the server module still interfaces through the client module

use crate::graphics;

use self::state::State;

pub mod camera;
mod draw;
mod state;

pub fn run() {
    pollster::block_on(graphics::run(Box::new(State::new())));
}
