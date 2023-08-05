use block::Block;

pub mod block;
mod generation;

pub struct World {
    chunks: Vec<Chunk>,
}

impl World {
    /// constructs world of empty air tiles
    pub fn empty() -> World {
        let res = World { chunks: vec![Chunk::empty(); 256] };
        res
    }

    /// get an immutable reference to the block at the given position
    pub fn get_block(&self, pos: Position) -> &Block {
        &self.chunks[pos.chunk as usize] // Chunk
            .columns[pos.column as usize] // Column
            .blocks[pos.height as usize] // Block
    }

    /// get a mutable reference to the block at the given position
    pub fn get_block_mut(&mut self, pos: Position) -> &mut Block {
        &mut self.chunks[pos.chunk as usize] // Chunk
            .columns[pos.column as usize] // Column
            .blocks[pos.height as usize] // Block
    }

    /// update the world given this specific update
    pub fn process_update(&mut self, update: Update) {
        self.get_block_mut(update.pos).kind = update.new_kind;
    }

    /// simulates one "tick" at the given chunk
    pub fn simulate(&mut self, chunk: u8) {
        todo!()
    }

    /// generates a new world by the
    pub fn generate() -> World {
        generation::generate()
    }
}

#[derive(Clone, Copy)]
pub struct Chunk {
    columns: [Column; 256],
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            columns: [Column::empty(); 256],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Column {
    blocks: [Block; 256],
}

impl Column {
    pub fn empty() -> Column {
        Column {
            blocks: [Block::new(0, 0); 256],
        }
    }
}

pub struct Update {
    pos: Position,
    new_kind: u8,
}

pub struct Position {
    chunk: u8,
    column: u8,
    height: u8,
}

impl Position {
    pub fn new(chunk: u8, column: u8, height: u8) -> Position {
        Position {
            chunk,
            column,
            height,
        }
    }

    /// from x,y,z coordinates
    ///
    /// # Arguments
    /// * `x` - x position (east-west)
    /// * `y` - y position (north-south)
    /// * `z` - z position (height)
    pub fn from_xyz(x: u8, y: u8, z: u8) -> Position {
        Position {
            chunk: (x / 16) + ((y / 16) * 16),
            column: (x % 16) + ((y % 16) * 16),
            height: z,
        }
    }

    /// to x,y,z coordinates
    pub fn to_xyz(&self) -> (u8, u8, u8) {
        (
            ((self.chunk % 16) * 16) + self.column % 16,
            ((self.chunk / 16) * 16) + self.column / 16,
            self.height,
        )
    }
}
