use super::graphics::Graphics;
use egui::{ClippedPrimitive, Context, TexturesDelta};
use winit::{dpi::PhysicalSize, event::WindowEvent};

pub struct Gui {
    state: egui_winit::State,
    context: Context,
    renderer: egui_wgpu::Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
    screen_descriptor: egui_wgpu::renderer::ScreenDescriptor,
}

impl Gui {
    pub fn new(
        gfx: &Graphics,
        event_loop: &winit::event_loop::EventLoop<()>,
        pixels_per_point: f32,
        screen_size: PhysicalSize<u32>,
    ) -> Self {
        let context = egui::Context::default();
        let mut style = egui::Style::default();
        for (_, font) in style.text_styles.iter_mut() {
            font.size *= 1.5;
        }
        context.set_style(style);
        let renderer = egui_wgpu::Renderer::new(&gfx.device, gfx.surface_config.format, None, 1);
        let mut state = egui_winit::State::new(event_loop);
        state.set_pixels_per_point(pixels_per_point);
        let paint_jobs = Vec::new();
        let textures = TexturesDelta::default();
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [screen_size.width, screen_size.height],
            pixels_per_point,
        };
        Gui {
            state,
            context,
            renderer,
            paint_jobs,
            textures,
            screen_descriptor,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent<'_>) {
        let _ = self.state.on_event(&self.context, event);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.screen_descriptor.size_in_pixels = [new_size.width, new_size.height];
        }
    }

    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        self.screen_descriptor.pixels_per_point = pixels_per_point;
    }

    pub fn run<R>(&mut self, window: &winit::window::Window, f: impl FnOnce(&Context) -> R) -> R {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_frame(raw_input);
        let result = f(&self.context);
        let output = self.context.end_frame();

        self.textures.append(output.textures_delta);
        self.state
            .handle_platform_output(window, &self.context, output.platform_output);
        self.paint_jobs = self.context.tessellate(output.shapes);
        result
    }

    pub fn pre_render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );
    }

    pub fn post_render(&mut self) {
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }

    pub fn render<'a>(&'a mut self, rpass: &mut wgpu::RenderPass<'a>) {
        self.renderer
            .render(rpass, &self.paint_jobs, &self.screen_descriptor);
    }
}
