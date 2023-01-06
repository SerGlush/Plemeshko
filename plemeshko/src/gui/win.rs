use std::ops::ControlFlow;

use egui::TexturesDelta;
use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{self, EventLoop},
    window::WindowBuilder,
};

use super::gfx::Gfx;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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
    let window = {
        let size = LogicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Plemeshko :3")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let gfx = Gfx::new(&window);

    let mut gui = match initialize() {
        ControlFlow::Continue(gui) => gui,
        ControlFlow::Break(code) => std::process::exit(code),
    };

    let mut gfx = futures::executor::block_on(gfx);

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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if window_id == window.id() {
                match event {
                    winit::event::WindowEvent::Resized(new_inner_size) => {
                        gfx.resize(*new_inner_size)
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        gfx.resize(**new_inner_size)
                    }
                    _ => (),
                }
            }
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(window_id) => {
            if window_id == window.id() {
                match gfx.render() {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => gfx.reconfigure_surface(),
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = winit::event_loop::ControlFlow::ExitWithCode(1);
                    }
                    Err(_) => (),
                }
            }
        }
        _ => (),
    });
}
