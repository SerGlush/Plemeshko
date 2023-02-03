use winit::event::Event;

use crate::{app::App, sim::Sim, state::AppState};

use self::{
    graphics::{Graphics, RenderError},
    gui::Gui,
};

mod graphics;
mod gui;
mod window;

pub fn run(mut app_st: AppState) -> ! {
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
                        app_st.shared.sim.lock().unwrap().as_mut().map(Sim::exit);
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
            match app.update(&mut app_st) {
                Ok(_) => (),
                Err(e) => {
                    *control_flow = winit::event_loop::ControlFlow::ExitWithCode(1);
                    println!("App update error: {e}");
                    return;
                }
            }
            window.request_redraw(); // ?
        }
        Event::RedrawRequested(window_id) => {
            if window_id == window.id() {
                if let Err(e) = gui.run(&window, |egui_ctx| app.ui(&mut app_st, egui_ctx)) {
                    *control_flow = winit::event_loop::ControlFlow::ExitWithCode(1);
                    println!("App update error: {e:#}");
                    return;
                }
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
