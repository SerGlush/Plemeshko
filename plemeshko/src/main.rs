#![feature(hash_raw_entry)]
#![feature(map_try_insert)]
#![feature(int_roundings)]
// #![feature(closure_lifetime_binder)]
#![deny(elided_lifetimes_in_paths)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use server::Sim;

mod client;
mod cor;
mod server;

fn main() {
    let sim = match Sim::init() {
        Ok(sim) => Arc::new(Mutex::new(sim)),
        Err(e) => {
            println!("Sim initialization error: {e}");
            std::process::exit(1);
        }
    };
    let quit = Arc::new(Mutex::new(false));
    let sim_thread_handle = {
        let sim = sim.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));
            sim.lock().unwrap().step();
            if *quit.lock().unwrap() {
                break;
            }
        })
    };

    client::run(sim);
    sim_thread_handle.join().unwrap();
}
