use block::Block;

pub mod block;

pub struct World {
    chunks: Box<[Chunk; 256]>,
}

impl World {
    pub fn empty() -> World {
        // so ugly
        World {
            chunks: Box::new(
                [Chunk {
                    columns: [Column {
                        blocks: [Block::default(); 256],
                    }; 256],
                }; 256],
            ),
        }
    }

    pub fn generate() -> World {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct Chunk {
    columns: [Column; 256],
}

#[derive(Clone, Copy)]
pub struct Column {
    blocks: [Block; 256],
}
