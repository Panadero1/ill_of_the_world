//! the client serves as the interface with the user
//! 
//! it handles all graphics calls
//! the server module still interfaces through the client module

use self::state::State;

mod state;
mod draw;

pub fn run() {
    let mut state = State::new();
    state.run();
}