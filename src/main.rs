/*  File: main.rs
    Author: Saiful Islam
    Date: 16th December, 2025
    Description: This file is the entry point for the process supervisor application.
*/

mod config;
mod process;

use std::{thread, time::Duration};
use std::error::Error;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use config::Config;
use process::ProcessManager;

fn main() -> Result<(), Box<dyn Error>> {
    // Set the RUST_LOG environment variable for logging.
    unsafe { env::set_var("RUST_LOG", "info"); }
    // Initialize the logger.
    env_logger::init();

    log::info!("Starting process supervisor");

    // Create an AtomicBool to control the main loop, allowing for graceful shutdown.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up a Ctrl+C handler to gracefully shut down the supervisor.
    ctrlc::set_handler(move || {
        println!();
        log::info!("SIGINT received. Shutting down.");
        r.store(false, Ordering::SeqCst); // set running = false
    })?;

    // Load configuration from the "config.toml" file.
    let config = Config::from_file("config.toml")?;
    // Create a new ProcessManager instance.
    let mut manager = ProcessManager::new();

    // Start all services defined in the configuration.
    for service_config in &config.services {
        manager.start_service(service_config)?;
    }

    // Main loop: continuously check and manage processes until the 'running' flag is false.
    while running.load(Ordering::SeqCst) {
        // Check the status of all managed processes and restart them if necessary.
        manager.check_processes()?;
        // Pause for a short duration to avoid busy-waiting.
        thread::sleep(Duration::from_secs(1));
    }

    log::info!("Shutting down all services.");
    manager.stop_all();

    Ok(())
}