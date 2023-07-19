pub mod graphics;
pub mod client;
pub mod server;
pub mod world;

fn main() {
    pollster::block_on(graphics::run());
}
