use block::Block;

pub mod block;
mod generation;

pub struct World {
    /// A collection of blocks all together.
    /// These are stored in chunk-column-block order
    /// - there are 256 chunks stored sequentially
    /// - within each chunk are 256 sequentially stored columns
    /// - within each column are 256 sequentially stored blocks
    blocks: Vec<Block>,
}

const SINGLE: usize = 256;
const DOUBLE: usize = 256 * 256;
const TRIPLE: usize = 256 * 256 * 256;

impl World {
    /// constructs world of empty air tiles
    pub fn empty() -> World {
        let res = World {
            blocks: vec![Block::default(); TRIPLE],
        };
        res
    }

    /// get an immutable reference to the block at the given position
    pub fn get_block(&self, pos: Position) -> &Block {
        &self.blocks[pos.index]
    }

    /// get a mutable reference to the block at the given position
    pub fn get_block_mut(&mut self, pos: Position) -> &mut Block {
        &mut self.blocks[pos.index]
    }

    /// update the world given this specific update
    pub fn process_update(&mut self, update: Update) {
        self.get_block_mut(update.pos).kind = update.new_kind;
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

pub struct Update {
    pos: Position,
    new_kind: u8,
}

pub struct Position {
    index: usize,
}

impl Position {
    pub fn new(index: usize) -> Position {
        Position { index }
    }

    /// from x,y,z coordinates
    ///
    /// # Arguments
    /// * `x` - x position (east-west)
    /// * `y` - y position (height)
    /// * `z` - z position (north-south)
    pub fn from_xyz(x: i16, y: u8, z: i16) -> Position {
        let x = x.rem_euclid(256) as u8;
        let z = z.rem_euclid(256) as u8;

        let chunk = ((x / 16) | (z & 0b11110000)) as usize;
        let column = ((x % 16) | (16 * (z % 16))) as usize;
        let block = y as usize;

        Position {
            index: chunk * DOUBLE + column * SINGLE + block,
        }
    }

    pub fn chunk(&self) -> u8 {
        (self.index / DOUBLE) as u8
    }

    pub fn column(&self) -> u8 {
        ((self.index / SINGLE) % SINGLE) as u8
    }

    pub fn block(&self) -> u8 {
        (self.index % SINGLE) as u8
    }
}
