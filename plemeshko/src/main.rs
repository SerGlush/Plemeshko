#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
#![feature(iterator_try_collect)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::mut_mutex_lock)] // false positives
#![allow(dead_code)]

use std::{sync::Mutex, time::Instant};

use env::{SharedEnv, SimEnv};
use sim::{Sim, SimSnapshot};

use crate::env::AppEnv;

#[macro_use]
mod util;
mod app;
mod env;
mod framework;
mod sim;

fn load_sim(env: &SimEnv) -> anyhow::Result<Sim> {
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

macro_rules! error_catch_print_exit {
    ($e:expr, $msg:literal) => {
        match $e {
            Ok(value) => value,
            Err(e) => {
                println!($msg, e);
                std::process::exit(1);
            }
        }
    };
}

fn main() {
    // todo: consider RwLock / partial locking; multithreaded sim
    let senv = error_catch_print_exit!(SharedEnv::new(), "Shared env init failed: {}");
    let sim = error_catch_print_exit!(load_sim(&senv), "Error reading Sim snapshot: {}");
    // sim and env are never dropped
    static_assertions::assert_not_impl_all!(Sim: Drop);
    static_assertions::assert_not_impl_all!(app::App: Drop);
    let (sim, senv) = Box::leak(Box::new((Mutex::new(sim), senv)));
    let aenv = error_catch_print_exit!(AppEnv::new(senv), "App env init failed: {}");
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
                    let step_result = sim.step(senv);
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

        framework::run(sim, aenv);
    });
}
