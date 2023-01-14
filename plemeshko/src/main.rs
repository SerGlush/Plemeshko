#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
#![feature(iterator_try_collect)]
#![deny(elided_lifetimes_in_paths)]
#![allow(dead_code)]

use std::{sync::Mutex, time::Instant};

use env::Env;
use sim::{Sim, SimSnapshot};

#[macro_use]
mod util;
mod app;
mod env;
mod framework;
mod sim;

fn load_sim(env: &Env) -> anyhow::Result<Sim> {
    let mut cli_args_iter = std::env::args();
    cli_args_iter.next(); // exe
    Ok(match cli_args_iter.next() {
        Some(snapshot_path) => {
            let file = std::fs::File::open(snapshot_path)?;
            let reader = std::io::BufReader::new(file);
            let snapshot = serde_json::from_reader::<_, SimSnapshot>(reader)?;
            Sim::restore(env, snapshot)?
        }
        None => Sim::new(),
    })
}

fn main() {
    // todo: consider RwLock / partial locking; multithreaded sim
    let env = match Env::new() {
        Ok(env) => env,
        Err(e) => {
            println!("Sim initialization error: {e}");
            std::process::exit(1);
        }
    };
    let sim = match load_sim(&env) {
        Ok(sim) => sim,
        Err(e) => {
            println!("Error reading Sim snapshot: {e}");
            std::process::exit(1);
        }
    };
    let sim = Box::leak(Box::new(Mutex::new(sim)));
    std::thread::scope(|thread_scope| {
        thread_scope.spawn(|| {
            let mut tick_delay = Sim::TICK_DELAY;
            loop {
                let instant = Instant::now();
                {
                    let mut sim = sim.lock().unwrap();
                    if sim.exited() {
                        break;
                    }
                    let step_result = sim.step(&env);
                    match step_result {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Sim error: {e}");
                            // todo: signal to the ui when step fails
                            break;
                        }
                    }
                }
                tick_delay -= Sim::TICK_DELAY;
                // note: `instant.elapsed()` before and after "sleep" aren't equal
                if tick_delay + instant.elapsed() < Sim::TICK_DELAY - Sim::TICK_THRESHOLD {
                    std::thread::sleep(Sim::TICK_DELAY - tick_delay - instant.elapsed());
                }
                tick_delay += instant.elapsed();
                // todo: measure when doing heavy computations (don't forget about println overhead?)
                // on zero load shows 70μs+-20μs most of the time
                // println!("Tick delay overhead: {:10}μs", (tick_delay.as_nanos() - Sim::TICK_DELAY.as_nanos()) / 1000);
            }
        });

        framework::run(sim);
    });
}
