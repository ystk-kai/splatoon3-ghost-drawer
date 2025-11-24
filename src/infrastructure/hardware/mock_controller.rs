use crate::domain::controller::{ControllerCommand, ControllerEmulator};
use crate::domain::hardware::errors::HardwareError;
use std::thread;
use std::time::Duration;
use tracing::{debug, info};

pub struct MockController;

impl Default for MockController {
    fn default() -> Self {
        Self::new()
    }
}

impl MockController {
    pub fn new() -> Self {
        Self
    }
}

impl ControllerEmulator for MockController {
    fn initialize(&self) -> Result<(), HardwareError> {
        info!("Initializing Mock Controller...");
        Ok(())
    }

    fn is_connected(&self) -> Result<bool, HardwareError> {
        Ok(true)
    }

    fn execute_command(&self, command: &ControllerCommand) -> Result<(), HardwareError> {
        debug!("Mock executing command: {}", command.name);
        for action in &command.sequence {
            // Simulate action duration
            thread::sleep(Duration::from_millis(action.duration_ms as u64));
        }
        Ok(())
    }

    fn shutdown(&self) -> Result<(), HardwareError> {
        info!("Shutting down Mock Controller");
        Ok(())
    }
}
