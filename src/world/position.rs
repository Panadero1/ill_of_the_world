use super::{DOUBLE, SINGLE};

pub type Position = usize;

/// from chunk,column,block coordinates
/// 
/// # Arguments
/// * `chunk` - the chunk number (0-255)
/// * `column` - the column number (0-255)
/// * `block` - the block number (0-255)
#[inline]
pub fn from_ccb(chunk: u8, column: u8, block: u8) -> Position {
    chunk as usize * DOUBLE + column as usize * SINGLE + block as usize
}

/// from x,y,z coordinates
///
/// # Arguments
/// * `x` - x position (east-west)
/// * `y` - y position (height)
/// * `z` - z position (north-south)
#[inline]
pub fn from_xyz(x: i16, y: u8, z: i16) -> Position {
    let x = x.rem_euclid(256) as u8;
    let z = z.rem_euclid(256) as u8;

    let chunk = ((x / 16) | (z & 0b11110000)) as usize;
    let column = ((x % 16) | (16 * (z % 16))) as usize;
    let block = y as usize;

    chunk * DOUBLE + column * SINGLE + block
}

pub fn chunk(pos: Position) -> u8 {
    (pos / DOUBLE) as u8
}

pub fn column(pos: Position) -> u8 {
    ((pos / SINGLE) % SINGLE) as u8
}

pub fn block(pos: Position) -> u8 {
    (pos % SINGLE) as u8
}
