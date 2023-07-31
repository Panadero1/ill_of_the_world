use winit::dpi::PhysicalSize;

use self::{
    component::Component,
    model::{Positioner, UIModel, UIVertex, DrawUI},
};

use super::{model::Vertex, resources::load_model_ui, state::create_render_pipeline, texture};

pub mod buttons;
pub mod component;
pub mod model;

pub struct UIManager {
    pub models: Vec<UIModel>,
    pub pipeline: wgpu::RenderPipeline,
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

    pub async fn add(
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
            .await
            .unwrap(),
        );
    }

    pub fn update_positions(&mut self, device: &wgpu::Device, canvas_size: PhysicalSize<u32>) {
        for m in &mut self.models {
            m.update_position(device, canvas_size)
        }
    }
}
