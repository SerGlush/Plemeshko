#![feature(hash_raw_entry)]
#![feature(map_try_insert)]

use std::{sync::{Mutex, Arc}, time::Duration};

use sim::Sim;

mod sim;
mod tui;

fn main() {
    let sim = Arc::new(Mutex::new(Sim::default()));
    let sim_thread_handle = {
        let sim = sim.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(100));
                if let Step::Halt = sim.lock().unwrap().step() {
                    break;
                }
            }
        })
    };

    tui::run(sim);
    sim_thread_handle.join().unwrap();
}
