use wgpu::{CommandEncoder, RenderPass, SurfaceError, SurfaceTexture, TextureView};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct Graphics {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
}

pub struct Frame {
    surface_texture: SurfaceTexture,
    surface_view: TextureView,
    pub encoder: CommandEncoder,
}

pub enum RenderError {
    Skip,
    OutOfMemory,
}

impl Graphics {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        // SAFETY:
        // ?
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Wgpu adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: wgpu::Label::None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Wgpu device");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &surface_config);
        Graphics {
            device,
            queue,
            surface,
            surface_config,
        }
    }

    pub fn surface_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.surface_config.width,
            height: self.surface_config.height,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn handle_event(&mut self, _event: &WindowEvent<'_>) -> bool {
        false
    }

    pub fn new_frame(&mut self) -> Result<Frame, RenderError> {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(SurfaceError::Lost) => {
                self.resize(self.surface_size());
                return Err(RenderError::Skip);
            }
            Err(SurfaceError::OutOfMemory) => return Err(RenderError::OutOfMemory),
            Err(_) => return Err(RenderError::Skip),
        };
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        Ok(Frame {
            surface_texture,
            surface_view,
            encoder,
        })
    }

    pub fn begin<'a>(&mut self, frame: &'a mut Frame) -> RenderPass<'a> {
        frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            })
    }

    pub fn end(&mut self, frame: Frame) {
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        frame.surface_texture.present();
    }
}
