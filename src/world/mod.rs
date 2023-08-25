use block::Block;

use self::{block::BlockUpdate, position::Position};

pub mod block;
mod generation;
pub mod position;
pub mod update;

const SINGLE: usize = 256;
const DOUBLE: usize = 256 * 256;
const TRIPLE: usize = 256 * 256 * 256;

pub struct World {
    /// A collection of blocks all together.
    /// These are stored in chunk-column-block order
    /// - there are 256 chunks stored sequentially
    /// - within each chunk are 256 sequentially stored columns
    /// - within each column are 256 sequentially stored blocks
    blocks: Vec<Block>,
}

impl World {
    /// constructs world of all zeros
    pub fn empty() -> World {
        let res = World {
            blocks: vec![Block::default(); TRIPLE],
        };
        res
    }

    /// get an immutable reference to the block at the given position
    pub fn get_block(&self, pos: Position) -> &Block {
        &self.blocks[pos]
    }

    /// get a mutable reference to the block at the given position
    pub fn get_block_mut(&mut self, pos: Position) -> &mut Block {
        &mut self.blocks[pos]
    }

    /// update the world given this specific update
    pub fn process_update(&mut self, update: BlockUpdate) {
        self.get_block_mut(position::from_ccb(update.chunk, update.column, update.block)).data = update.new_data;
    }

    /// simulates one "tick" at the given chunk
    pub fn simulate(&mut self, chunk: u8) {
        todo!()
    }

    /// generates a new world by the generation algorithm
    pub fn generate() -> World {
        generation::generate()
    }
}