mod config;
mod process;

use std::{thread, time::Duration};
use std::error::Error;
use std::env;

use config::Config;
use process::ProcessManager;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe { env::set_var("RUST_LOG", "info"); }
    env_logger::init();

    log::info!("Starting process supervisor");

    let config = Config::from_file("config.toml")?;
    
    let mut manager = ProcessManager::new();

    for service_config in &config.services {
        manager.start_service(service_config)?;
    }

    loop {
        manager.check_processes()?;
        thread::sleep(Duration::from_secs(1));
    }
}