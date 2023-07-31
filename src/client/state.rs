use crate::{world::World, graphics};

pub struct State {
    world: World,
}

impl State {
    pub fn new() -> State {
        State {
            world: World::empty(),
        }
    }

    pub fn run(&mut self) {
        pollster::block_on(graphics::run());
        // todo: add api for graphics and pages
    }
}
