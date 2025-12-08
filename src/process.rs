use std::collections::HashMap;
use std::process::{self, Command};
use std::Error;

use crate::config::{RestartPolicy, ServiceConfig};

pub struct ProcessManager {
    processes: HashMap<String, ProcessInfo>,
}

#[derive(Debug)]
struct ProcessInfo {
    pub child: std::process::Child,
    restart_count: u32,
    last_exit_code: Option<i32>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
        }
    }

    pub fn start_service(&mut self, config: &ServiceConfig) -> Result<i32, Box<dyn Error>> {
        let mut command = Command::new(&config.command);
        command.args(&config.arg);
        command.envs(&config.env);
        let child = command.spawn()?;
        let pid = child.id() as i32;
        
        self.processes.insert(
            config.name.clone(),
            ProcessInfo {
                child: child,
                restart_count: 0,
                last_exit_code: None,
            },
        );

        log::info!("Started service {} with PID {}", config.name, pid);
        
        Ok(pid)
    }
}