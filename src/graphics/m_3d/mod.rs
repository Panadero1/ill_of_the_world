//! for 3d graphics

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use pollster::FutureExt;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use self::{
    model::{DrawModel, InstanceModel, Model},
    vertex::ModelVertex,
};

use super::{
    camera::{Camera, CameraController, CameraUniform, Projection},
    instance::{Instance, InstanceRaw},
    light::{DrawUnlit, LightUniform},
    model::vertex::Vertex,
    pipeline::create_render_pipeline,
    resources::{self, load_model},
    texture,
};

pub mod model;
pub mod vertex;

pub struct M3DManager {
    models: Vec<InstanceModel>,
    m3d_pipeline: wgpu::RenderPipeline,
    texture_bgl: wgpu::BindGroupLayout,

    // camera
    camera_bg: wgpu::BindGroup,
    projection: Projection,
    camera_control: CameraController,
    camera: Camera,
    camera_unif: CameraUniform,
    camera_buf: wgpu::Buffer,

    //light
    // todo: figure this out
    light_pipeline: wgpu::RenderPipeline,
    light_bg: wgpu::BindGroup,
    light_unif: LightUniform,
    light_buf: wgpu::Buffer,
    light_model: Model,
}

impl M3DManager {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> M3DManager {
        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // visibility field can be bitflag with all ShaderStages types
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the corresponding entry above
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let m3d_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bgl,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader,
                "regular",
            )
        };

        let light_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc()],
                shader,
                "light",
            )
        };

        // Todo: make this a constructor...
        let light_unif = LightUniform {
            position: [0.0, 20.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_unif]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buf.as_entire_binding(),
            }],
            label: None,
        });

        let camera = Camera::new((0.0, 5.0, 10.0), 0.0, 10.0, 5.0);
        let projection =
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_control = CameraController::new(4.0, 0.4);

        let mut camera_unif = CameraUniform::new();
        camera_unif.update_view_proj(&camera, &projection);

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_unif]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let unlit_model = load_model("light", "pyramid1.obj", device, queue, &texture_bgl)
            .block_on()
            .unwrap();

        M3DManager {
            models: Vec::new(),
            m3d_pipeline,
            light_pipeline,
            camera_bg,
            projection,
            light_bg,
            camera_control,
            camera,
            camera_unif,
            camera_buf,
            light_unif,
            light_buf,
            texture_bgl,
            light_model: unlit_model,
        }
    }

    pub fn add(
        &mut self,
        name: &str,
        obj_file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.models.push(InstanceModel {
            model: resources::load_model(name, obj_file_name, device, queue, &self.texture_bgl)
                .block_on()
                .unwrap(),
            instance_buffer: basic_instance_buf(device),
            num_instances: 1,
        });
    }

    pub fn add_instanced(
        &mut self,
        name: &str,
        obj_file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: Vec<Instance>,
    ) {
        self.models.push(InstanceModel {
            model: resources::load_model(name, obj_file_name, device, queue, &self.texture_bgl)
                .block_on()
                .unwrap(),
            instance_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Instance Buffer", name)),
                contents: bytemuck::cast_slice(
                    &instances.iter().map(Instance::to_raw).collect::<Vec<_>>(),
                ),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            num_instances: instances.len() as u32,
        });
    }

    pub fn render<'a: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) {
        render_pass.set_pipeline(&self.light_pipeline);
        render_pass.draw_light_model(&self.light_model, &self.camera_bg, &self.light_bg);

        render_pass.set_pipeline(&self.m3d_pipeline);

        // render all models (all of them have an instance buffer to avoid
        // tricky bugs. Just in case)
        for m in &self.models {
            render_pass.set_vertex_buffer(1, m.instance_buffer.slice(..));
            render_pass.draw_model_instanced(
                &m.model,
                0..m.num_instances,
                &self.camera_bg,
                &self.light_bg,
            );
        }
    }

    pub fn resize_projection(&mut self, new_size: PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height);
    }

    pub fn camera_control(&mut self) -> &mut CameraController {
        &mut self.camera_control
    }

    pub fn update_cam(&mut self, dt: instant::Duration, queue: &mut wgpu::Queue) {
        self.camera_control.update_camera(&mut self.camera, dt);
        self.camera_unif
            .update_view_proj(&self.camera, &self.projection);
        queue.write_buffer(
            &self.camera_buf,
            0,
            bytemuck::cast_slice(&[self.camera_unif]),
        );
    }

    pub fn update_light(&mut self, pos: [f32; 3], queue: &mut wgpu::Queue) {
        self.light_unif.position = pos;
        queue.write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&[self.light_unif]));
    }
}

fn basic_instance_buf(device: &wgpu::Device) -> wgpu::Buffer {
    let basic_instance = Instance {
        position: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(0.0)),
    };

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Single Instance Buffer"),
        contents: bytemuck::cast_slice(&vec![basic_instance.to_raw()]),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
