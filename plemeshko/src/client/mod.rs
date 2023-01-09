use std::{ops::DerefMut, sync::Mutex};

use winit::event::Event;

use crate::server::Sim;

use self::{
    app::App,
    graphics::{Graphics, RenderError},
    gui::Gui,
};

mod app;
mod error;
mod graphics;
mod gui;
mod window;

pub fn run(sim: &'static Mutex<Sim>) -> ! {
    let (event_loop, window) = window::initialize();
    let mut graphics = futures::executor::block_on(Graphics::new(&window));
    let mut gui = Gui::new(
        &graphics,
        &event_loop,
        window.scale_factor() as f32,
        window.inner_size(),
    );
    let mut app = App::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if window_id == window.id() {
                gui.handle_event(event);
                match event {
                    &winit::event::WindowEvent::Resized(new_inner_size) => {
                        graphics.resize(new_inner_size);
                        gui.resize(new_inner_size);
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        sim.lock().unwrap().exit();
                    }
                    winit::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        scale_factor,
                    } => {
                        let new_inner_size = **new_inner_size;
                        graphics.resize(new_inner_size);
                        gui.resize(new_inner_size); // ?
                        gui.set_pixels_per_point(*scale_factor as f32);
                    }
                    _ => (),
                }
            }
        }
        Event::MainEventsCleared => {
            app.update(sim.lock().unwrap().deref_mut());
            window.request_redraw(); // ?
        }
        Event::RedrawRequested(window_id) => {
            if window_id == window.id() {
                gui.run(&window, |ctx| app.gui(ctx, sim.lock().unwrap().deref_mut()));
                match graphics.new_frame() {
                    Ok(mut frame) => {
                        {
                            gui.pre_render(&graphics.device, &graphics.queue, &mut frame.encoder);
                            let mut rpass = graphics.begin(&mut frame);
                            gui.render(&mut rpass);
                        }
                        gui.post_render();
                        graphics.end(frame);
                    }
                    Err(RenderError::Skip) => (),
                    Err(RenderError::OutOfMemory) => {
                        *control_flow = winit::event_loop::ControlFlow::ExitWithCode(1);
                    }
                }
            }
        }
        _ => (),
    });
}
