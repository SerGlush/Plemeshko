#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
#![feature(iterator_try_collect)]
#![feature(associated_type_bounds)]
#![feature(fs_try_exists)]
#![feature(new_uninit)]
#![feature(map_many_mut)]
#![feature(try_find)]
#![feature(min_specialization)]
#![feature(trait_alias)]
#![deny(elided_lifetimes_in_paths)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::mut_mutex_lock)] // false positives
#![allow(dead_code)]

use std::time::Instant;

use anyhow::{Context, Result};
use sim::Sim;
use state::initialize_state;

#[macro_use]
mod log;
#[macro_use]
mod util;
#[macro_use]
mod state;
mod app;
mod framework;
mod sim;

fn main() -> Result<()> {
    crate::log::initialize_log().context("Initializing log")?;
    let (shared_st, app_st) = initialize_state().context("Initializing state")?;
    std::thread::scope(|thread_scope| {
        thread_scope.spawn(|| {
            let mut tick_delay = Sim::TICK_DELAY;
            loop {
                let instant = Instant::now();
                {
                    let mut sim = shared_st.sim.lock().unwrap();
                    let Some(sim) = sim.as_mut() else {
                        log::error!("Can't continue simulation: is `None`");
                        break;
                    };
                    if sim.exited() {
                        break;
                    }
                    let step_result = sim.step(shared_st);
                    match step_result {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Simulation error: {e:#}");
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

        framework::run(app_st);
    })
}
