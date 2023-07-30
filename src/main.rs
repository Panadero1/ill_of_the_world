pub mod client;
pub mod graphics;
pub mod server;
pub mod util;
pub mod world;

fn main() {
    pollster::block_on(graphics::run());
}
