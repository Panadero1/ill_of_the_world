use std::iter;

use wgpu::{Backends, InstanceDescriptor};
use winit::{dpi::PhysicalSize, window::Window};

use super::{
    camera::CameraController,
    instance::Instance,
    m_3d::M3DManager,
    texture,
    ui::{model::Positioner, UIManager},
};

pub struct Graphics {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    depth_texture: texture::Texture,
    pub m3d_mgr: M3DManager,
    pub ui_mgr: UIManager,
}

impl Graphics {
    // Tutorial says creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance_desc = InstanceDescriptor {
            // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
            backends: Backends::all(),
            // this is a slower compiler, but we don't need external libraries to use it
            // so it's just easier this way
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        };
        let instance = wgpu::Instance::new(instance_desc);
        let surface = unsafe {
            instance
                .create_surface(window)
                .expect("unable to make surface")
        };

        // Todo: are all adapters supported by this filtering? Does it do a good job here?
        // https://docs.rs/wgpu/latest/wgpu/struct.Adapter.html
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Todo: we disable features here. If performance is bad, what features can we enable?
        // adapter.features() or device.features()
        // https://docs.rs/wgpu/latest/wgpu/struct.Features.html

        // Todo: limits are defaults by tutorial. If an issue, see link below
        // https://docs.rs/wgpu/latest/wgpu/struct.Limits.html
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surf_caps = surface.get_capabilities(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surf_caps.formats[0],
            width: size.width,
            height: size.height,
            // Fifo means VSync
            // If I need to change this, I can do surface.get_capabilities
            // Always guaranteed to work on all platforms are:
            // - Fifo
            // - AutoVsync
            // - AutoNoVsync
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let m3d_mgr = M3DManager::new(&device, &config, &queue);

        // m3d_mgr.add_instanced("cube", "cube.obj", &device, &queue, instances);

        let ui_mgr = UIManager::new(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            m3d_mgr,
            ui_mgr,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.m3d_mgr.resize_projection(new_size);
            self.ui_mgr.update_positions(&self.device, new_size);
        }
    }

    pub fn update_cam(&mut self, dt: instant::Duration) {
        // todo: fix camera to my liking
        self.m3d_mgr.update_cam(dt, &mut self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // This is the background color
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.m3d_mgr.render(&mut render_pass);

            self.ui_mgr.render(&mut render_pass);
            // There is custom drop code for type RenderPass that uses it
            // Need to drop before next borrow of encoder which is why this is in a block
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn add_model_ui(&mut self, name: &str, texture_file_name: &str, positioner: Positioner) {
        // todo: shared ui texture. Just load that and take tex coords
        self.ui_mgr.add(
            name,
            texture_file_name,
            &self.device,
            &self.queue,
            self.size,
            positioner,
        );
    }

    pub fn add_model_3d(&mut self, name: &str, obj_file_name: &str) {
        self.m3d_mgr
            .add(name, obj_file_name, &self.device, &self.queue);
    }

    pub fn add_model_3d_instanced(
        &mut self,
        name: &str,
        obj_file_name: &str,
        instances: Vec<Instance>,
    ) {
        // todo: let user modify these instances in some way
        self.m3d_mgr
            .add_instanced(name, obj_file_name, &self.device, &self.queue, instances);
    }

    pub fn position_light(&mut self, pos: [f32; 3]) {
        self.m3d_mgr.update_light(pos, &mut self.queue);
    }

    pub fn camera_control(&mut self) -> &mut CameraController {
        self.m3d_mgr.camera_control()
    }
}
