use wgpu::{
    BufferAddress,
    VertexFormat::{Float32x2, Float32x3},
};

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
