use crate::world::World;

struct State {
    world: World,
}

impl State {
    pub fn new() -> State {
        State {
            world: World::empty(),
        }
    }

    
}
