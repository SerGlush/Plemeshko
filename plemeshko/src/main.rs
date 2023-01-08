#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
#![deny(elided_lifetimes_in_paths)]
#![allow(dead_code)]

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use server::Sim;

mod client;
mod cor;
mod server;

fn main() {
    // todo: consider RwLock / partial locking; multithreaded sim
    let sim = match Sim::new() {
        Ok(sim) => Box::leak::<'static>(Box::new(Mutex::new(sim))),
        Err(e) => {
            println!("Sim initialization error: {e}");
            std::process::exit(1);
        }
    };
    std::thread::scope(|thread_scope| {
        thread_scope.spawn(|| {
            let mut tick_delay = Duration::ZERO;
            loop {
                let instant = Instant::now();
                {
                    let mut sim = sim.lock().unwrap();
                    if sim.exited() {
                        break;
                    }
                    let step_result = sim.step();
                    match step_result {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Sim error: {e}");
                            break;
                        }
                    }
                }
                tick_delay += Sim::TICK_DELAY;
                // note: `instant.elapsed()` before and after "sleep" aren't equal
                if tick_delay - instant.elapsed() - Sim::TICK_THRESHOLD > Duration::ZERO {
                    std::thread::sleep(tick_delay);
                }
                tick_delay -= instant.elapsed();
            }
        });

        client::run(sim);
    });
}
