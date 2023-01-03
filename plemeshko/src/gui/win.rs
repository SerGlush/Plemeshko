use std::ops::ControlFlow;

use egui::TexturesDelta;
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{self, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

async fn init_wgpu() {
    let wgpu_instance = wgpu::Instance::new(wgpu::Backends::all());
    let wgpu_adapter = wgpu_instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Wgpu adapter");
    let (wgpu_device, wgpu_queue) = wgpu_adapter
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
}

pub(super) fn run<
    G,
    Init: FnOnce() -> ControlFlow<i32, G>,
    Upd: FnMut(&mut G) -> ControlFlow<i32>,
>(
    initialize: Init,
    update: Upd,
) {
    // Winit
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Hello Pixels + egui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let wgpui = init_wgpu();

    let mut gui = match initialize() {
        ControlFlow::Continue(gui) => gui,
        ControlFlow::Break(code) => std::process::exit(code),
    };

    let wgpui = futures::executor::block_on(wgpui);

    // Egui
    // let egui_ctx = egui::Context::default();
    // let mut egui_state = egui_winit::State::new(&event_loop);
    // // egui_state.set_max_texture_side(max_texture_size);
    // egui_state.set_pixels_per_point(window.scale_factor() as f32); // ?
    // let screen_descriptor = ScreenDescriptor {
    //     size_in_pixels: [WIDTH, HEIGHT],
    //     pixels_per_point: window.scale_factor() as f32,
    // };
    // let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
    // let textures = TexturesDelta::default();

    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = event_loop::ControlFlow::Exit;
                return;
            }

            // // Update the scale factor
            // if let Some(scale_factor) = input.scale_factor() {
            //     framework.scale_factor(scale_factor);
            // }

            // // Resize the window
            // if let Some(size) = input.window_resized() {
            //     framework.resize(size.width, size.height);
            // }

            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                // framework.handle_event(&event);
            }
            // Draw the current frame
            Event::RedrawRequested(_) => {
                // Prepare egui
                // framework.prepare(&window);

                // framework-render
            }
            _ => (),
        }
    });
}
