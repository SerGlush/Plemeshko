use std::sync::{Arc, Mutex};

use crate::sim::Sim;

mod error;
mod gfx;
mod gui;
mod win;

pub fn run(sim: Arc<Mutex<Sim>>) {
    win::run(
        || {
            let sim = sim.lock().unwrap();
            match gui::Gui::init(&sim) {
                Ok(gui) => std::ops::ControlFlow::Continue(gui),
                Err(_) => todo!(),
            }
        },
        |gui| std::ops::ControlFlow::Continue(()),
    );
}
