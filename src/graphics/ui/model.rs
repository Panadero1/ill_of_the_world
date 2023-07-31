use crate::graphics::{model::Vertex, texture};
use wgpu::{
    util::DeviceExt, BindGroup, BufferAddress, VertexFormat::Float32x2, VertexFormat::Float32x3,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};

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

pub struct UIMaterial {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl UIMaterial {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            bind_group,
        }
    }
}

pub type Positioner = fn(PhysicalSize<u32>) -> Vec<UIVertex>;

pub struct UIModel {
    name: String,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_elements: u32,
    positioner: Positioner,
}

impl UIModel {
    const INDICES: &'static [u16] = &[0, 1, 2, 2, 3, 0];
    // top left CCW around to top right (U shape)
    pub fn new(
        name: &str,
        device: &wgpu::Device,
        diffuse_texture: texture::Texture,
        bind_group_layout: &wgpu::BindGroupLayout,
        positioner: Positioner,
        canvas_size: PhysicalSize<u32>,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some(&format!("{}_bind_group", name)),
        });

        let vertices = positioner(canvas_size);

        Self {
            bind_group,
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_vertex_buffer", name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui_index_buffer"),
                contents: bytemuck::cast_slice(UIModel::INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }),
            num_elements: 3*2,
            positioner,
            name: name.to_string(),
        }
    }

    pub fn update_position(&mut self, device: &wgpu::Device, canvas_size: PhysicalSize<u32>) {
        let position = (self.positioner)(canvas_size);
        // println!("{:?}", position);
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_vertex_buffer", self.name)),
            contents: bytemuck::cast_slice(&position),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }
}

pub trait DrawUI<'a> {
    // relevant one
    fn draw_model_ui(
        &mut self,
        model: &'a UIModel,
    );
}

impl<'a, 'b> DrawUI<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_model_ui(
        &mut self,
        model: &'a UIModel,
    ) {
        self.set_bind_group(0, &model.bind_group, &[]);
        self.set_vertex_buffer(0, model.vertex_buffer.slice(..));
        self.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..model.num_elements, 0, 0..1);

        // self.set_bind_group(0, &model.bind_group, &[]);
        // self.set_vertex_buffer(0, model.vertex_buffer.slice(..));
        // self.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        // self.draw_indexed(0..model.num_elements, 0, 0..1);
    }
}

/// Should not be called every frame; only every resize!
/// pos is position of top left corner
/// 
pub fn rect_vertices(
    canvas_size: PhysicalSize<u32>,
    size: PhysicalSize<u32>,
    pos: PhysicalPosition<i32>,
) -> Vec<UIVertex> {
    // Bottom left CCW to bottom right
    let canvas_size = PhysicalSize::new(canvas_size.width as f32, canvas_size.height as f32);
    let pos = PhysicalPosition::new(pos.x as f32, pos.y as f32);
    let size = PhysicalSize::new(size.width as f32, size.height as f32);
    vec![
        // Top left
        UIVertex {
            position: stretch_to_range(pos.x / canvas_size.width, pos.y / canvas_size.height),
            tex_coords: [0.0, 0.0],
        },
        // Bottom left
        UIVertex {
            position: stretch_to_range(
                pos.x / canvas_size.width,
                (pos.y + size.height) / canvas_size.height,
            ),
            tex_coords: [0.0, 1.0],
        },
        // Bottom right
        UIVertex {
            position: stretch_to_range(
                (pos.x + size.width) / canvas_size.width,
                (pos.y + size.height) / canvas_size.height,
            ),
            tex_coords: [1.0, 1.0],
        },
        // Top right
        UIVertex {
            position: stretch_to_range(
                (pos.x + size.width) / canvas_size.width,
                pos.y / canvas_size.height,
            ),
            tex_coords: [1.0, 0.0],
        },
    ]
}

fn stretch_to_range(x: f32, y: f32) -> [f32; 3] {
    [x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0]
}
