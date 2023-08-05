use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexFormat::Float32x3, VertexFormat::Float32x4};

use super::model;

pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl model::vertex::Vertex for InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next instance
            // when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    // Vertex shader uses locations 0-4
                    shader_location: 5,
                    format: Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s
                // We need to define four slots here then reassemble in the shader
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 9,
                    format: Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as BufferAddress,
                    shader_location: 10,
                    format: Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as BufferAddress,
                    shader_location: 11,
                    format: Float32x3,
                },
            ],
        }
    }
}
