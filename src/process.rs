use std::collections::HashMap;
use std::process::{Command, Child};
use std::error::Error;

use crate::config::{RestartPolicy, ServiceConfig};

pub struct ProcessManager {
    processes: HashMap<String, ProcessInfo>,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub child: Child,
    pub config: ServiceConfig,
    restart_count: u32,
    last_exit_code: Option<i32>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
        }
    }

    pub fn spawn_child(config: &ServiceConfig) -> Result<Child, Box<dyn Error>> {
        let child: Child = Command::new(&config.command).args(&config.args).envs(&config.env).spawn()?;
        Ok(child)
    }

    pub fn start_service(&mut self, config: &ServiceConfig) -> Result<i32, Box<dyn Error>> {
        let child = Self::spawn_child(&config)?;
        let pid = child.id() as i32;
        
        self.processes.insert(
            config.name.clone(),
            ProcessInfo {
                child: child,
                config: config.clone(),
                restart_count: 0,
                last_exit_code: None,
            },
        );

        log::info!("Started service {} with PID {}", config.name, pid);
        
        Ok(pid)
    }

    pub fn should_restart(process_info: &ProcessInfo) -> bool {
        match process_info.config.restart_policy {
            RestartPolicy::Always => true,
            RestartPolicy::Never => false,
            RestartPolicy::OnFailure => {
                match process_info.last_exit_code {
                    Some(0) => false,
                    _ => true,
                }
            }
        }
    }

    pub fn check_processes(&mut self) -> Result<(), Box<dyn Error>> {
        for (name, info) in self.processes.iter_mut() {
            if info.last_exit_code.is_some() {
                continue;
            }
            match info.child.try_wait()? {            
                Some(status) => {   
                    info.last_exit_code = status.code();
                    log::warn!("Service {} exited with status {}", name, status);
                    if ProcessManager::should_restart(&info) && info.restart_count < info.config.max_restarts {
                        let new_child = ProcessManager::spawn_child(&info.config)?;
                        info.child = new_child;
                        info.last_exit_code = None;
                        info.restart_count += 1;
                    }
                    else {
                        continue;
                    }
                }
                None => {}
            }
        }
        Ok(())
    }

    // A method to stop all running services gracefully.
    pub fn stop_all(&mut self) {
        for (name, info) in self.processes.iter_mut() {
            log::info!("Killing service: {}", name);
            match info.child.kill() {
                Ok(_) => log::info!("{} was killed successfully.", name),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    log::debug!("{} has already exited.", name);
                }
                Err(e) => log::error!("Couldn't kill {}: {}", name, e),
            }

            // Wait for the child process to exit.
            let _ = info.child.wait();
        }
    }
}