#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
#![feature(iterator_try_collect)]
#![feature(associated_type_bounds)]
#![feature(fs_try_exists)]
#![feature(new_uninit)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::mut_mutex_lock)] // false positives
#![allow(dead_code)]

use std::time::Instant;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use sim::Sim;
use state::initialize_state;

#[macro_use]
mod util;
#[macro_use]
mod state;
mod app;
mod framework;
mod sim;

fn main() -> Result<()> {
    initialize_log().context("Initializing log")?;
    let (shared_st, app_st) = initialize_state().context("Initializing state")?;
    std::thread::scope(|thread_scope| {
        thread_scope.spawn(|| {
            let mut tick_delay = Sim::TICK_DELAY;
            loop {
                let instant = Instant::now();
                {
                    let mut sim = shared_st.sim.lock().unwrap();
                    let Some(sim) = sim.as_mut() else {
                        println!("Sim is None");
                        break;
                    };
                    if sim.exited() {
                        break;
                    }
                    let step_result = sim.step(shared_st);
                    match step_result {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Sim error: {e:#}");
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

fn initialize_log() -> Result<()> {
    const ENV_LOG_TARGET: &str = "PLEMESHKO_LOG";
    let Ok(target) = std::env::var(ENV_LOG_TARGET) else {
        return Ok(());
    };

    let time_format =
        time::format_description::parse("[hour]:[minute]:[second].[subsecond digits:3]")?;

    if &target == "stdout" || &target == "stderr" {
        let fern_colors = fern::colors::ColoredLevelConfig::new()
            .info(fern::colors::Color::Cyan)
            .warn(fern::colors::Color::Yellow)
            .error(fern::colors::Color::Red);
        let fern_dispatch = fern::Dispatch::new().level(log::LevelFilter::Info).format(
            move |out, message, record| {
                let time = time::OffsetDateTime::from(std::time::SystemTime::now());
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    time.format(&time_format).unwrap(),
                    if record.target().starts_with(env!("CARGO_PKG_NAME")) {
                        record.target().green()
                    } else {
                        record.target().normal()
                    },
                    fern_colors.color(record.level()),
                    message
                ))
            },
        );
        match target.as_str() {
            "stdout" => fern_dispatch.chain(std::io::stdout()),
            "stderr" => fern_dispatch.chain(std::io::stderr()),
            _ => panic!(),
        }
        .apply()?;
        return Ok(());
    }
    if target.len() < 2 || target.starts_with('/') {
        bail!("Can't parse env: {ENV_LOG_TARGET}");
    }
    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .format(move |out, message, record| {
            let time = time::OffsetDateTime::from(std::time::SystemTime::now());
            out.finish(format_args!(
                "{}[{}][{}] {}",
                time.format(&time_format).unwrap(),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(target.chars().skip(1).collect::<String>())?)
        .apply()?;
    Ok(())
}
