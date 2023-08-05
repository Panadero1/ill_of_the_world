use wgpu::{
    BufferAddress,
    VertexFormat::{Float32x2, Float32x3},
};

use crate::graphics::model::vertex::Vertex;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for UIVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UIVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: Float32x2,
                },
            ],
        }
    }
}
