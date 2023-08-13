use pollster::FutureExt;
use winit::dpi::PhysicalSize;

use self::{
    model::{DrawUI, Model, Positioner},
    vertex::UIVertex,
};

use super::{
    model::vertex::Vertex, pipeline::create_render_pipeline, resources::load_model_ui, texture,
};

pub mod buttons;
pub mod component;
pub mod model;
pub mod vertex;

pub struct UIManager {
    models: Vec<Model>,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    // bind_groups: HashMap<String, BindGroup>,
}

impl UIManager {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> UIManager {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the corresponding entry above
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("UI Pipeline Layout"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("UI Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("ui.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[UIVertex::desc()],
                shader,
                "UI",
            )
        };
        UIManager {
            models: Vec::new(),
            pipeline,
            layout,
            // bind_groups: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        name: &str,
        texture_file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas_size: PhysicalSize<u32>,
        positioner: Positioner,
    ) {
        self.models.push(
            load_model_ui(
                name,
                texture_file_name,
                device,
                queue,
                &self.layout,
                canvas_size,
                positioner,
            )
            .block_on()
            .unwrap(),
        );
    }

    pub fn update_positions(&mut self, device: &wgpu::Device, canvas_size: PhysicalSize<u32>) {
        for m in &mut self.models {
            m.update_position(device, canvas_size)
        }
    }

    pub fn render<'a: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) {
        render_pass.set_pipeline(&self.pipeline);
        for m in &self.models {
            render_pass.draw_model_ui(m);
        }
    }
}
