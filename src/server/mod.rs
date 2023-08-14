use std::{sync::Arc, thread};

use instant::Duration;

use crate::{util::thread::ThreadPool, world::{World, position}};

/// indices for safe multithreaded chunk updating
///
/// the pattern created gives a tiling of the chunks where all
/// all indices in one subarray may be updated in an asynch fashion
///
/// the whole world would be updated in four passes, each of which are safe
///
/// thread safety is guaranteed since any block update chunk spillover
/// can only spill over to directly adjacent chunks.
///
/// the following set of indices have been verified
const CHUNK_THREAD_INDICES: [[u8; 64]; 4] = [
    [
        000, 002, 004, 006, 008, 010, 012, 014, 032, 034, 036, 038, 040, 042, 044, 046, 064, 066,
        068, 070, 072, 074, 076, 078, 096, 098, 100, 102, 104, 106, 108, 110, 128, 130, 132, 134,
        136, 138, 140, 142, 160, 162, 164, 166, 168, 170, 172, 174, 192, 194, 196, 198, 200, 202,
        204, 206, 224, 226, 228, 230, 232, 234, 236, 238,
    ],
    [
        001, 003, 005, 007, 009, 011, 013, 015, 033, 035, 037, 039, 041, 043, 045, 047, 065, 067,
        069, 071, 073, 075, 077, 079, 097, 099, 101, 103, 105, 107, 109, 111, 129, 131, 133, 135,
        137, 139, 141, 143, 161, 163, 165, 167, 169, 171, 173, 175, 193, 195, 197, 199, 201, 203,
        205, 207, 225, 227, 229, 231, 233, 235, 237, 239,
    ],
    [
        016, 018, 020, 022, 024, 026, 028, 030, 048, 050, 052, 054, 056, 058, 060, 062, 080, 082,
        084, 086, 088, 090, 092, 094, 112, 114, 116, 118, 120, 122, 124, 126, 144, 146, 148, 150,
        152, 154, 156, 158, 176, 178, 180, 182, 184, 186, 188, 190, 208, 210, 212, 214, 216, 218,
        220, 222, 240, 242, 244, 246, 248, 250, 252, 254,
    ],
    [
        017, 019, 021, 023, 025, 027, 029, 031, 049, 051, 053, 055, 057, 059, 061, 063, 081, 083,
        085, 087, 089, 091, 093, 095, 113, 115, 117, 119, 121, 123, 125, 127, 145, 147, 149, 151,
        153, 155, 157, 159, 177, 179, 181, 183, 185, 187, 189, 191, 209, 211, 213, 215, 217, 219,
        221, 223, 241, 243, 245, 247, 249, 251, 253, 255,
    ],
];

pub struct Server {
    world: Arc<World>,
    thread_pool: ThreadPool,
}

impl Server {
    pub fn new(threads: usize) -> Server {
        Server {
            world: Arc::new(World::generate()),
            thread_pool: ThreadPool::new(threads),
        }
    }

    pub fn update(&mut self) {
        // todo use actual thread pool crate
        for section in CHUNK_THREAD_INDICES {
            for i in section {
                let wc = self.world.clone();
                self.thread_pool.execute(move |_| {
                    let ptr = Arc::<World>::as_ptr(&wc) as *mut World;

                    unsafe {
                        for j in 0..255 {
                            for k in 0..255 {
                                (*ptr).get_block_mut(position::from_ccb(i, j, k)).kind = 2;
                                // thread::sleep(Duration::from_millis(1));
                            }
                        }
                    }
                    
                });
            }
        }
        // todo: get multithreaded updates working
    }
}
