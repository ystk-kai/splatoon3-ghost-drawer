use super::{Board, HardwareError, SystemdService, UsbGadget};
use async_trait::async_trait;
use crate::domain::setup::repositories::SetupError;

#[async_trait]
pub trait BoardRepository {
    async fn detect_board(&self) -> Result<Board, HardwareError>;
    async fn check_kernel_modules(&self, board: &mut Board) -> Result<(), HardwareError>;
    async fn load_kernel_module(&self, module_name: &str) -> Result<(), HardwareError>;
    async fn configure_boot_settings(&self, board: &Board) -> Result<(), HardwareError>;
}

#[async_trait]
pub trait UsbGadgetRepository {
    async fn create_gadget(&self, gadget: &UsbGadget) -> Result<(), HardwareError>;
    async fn configure_gadget(&self, gadget: &UsbGadget) -> Result<(), HardwareError>;
    async fn activate_gadget(&self, gadget: &mut UsbGadget) -> Result<(), HardwareError>;
    async fn deactivate_gadget(&self, gadget: &mut UsbGadget) -> Result<(), HardwareError>;
    async fn get_gadget_state(&self, gadget_id: &str) -> Result<UsbGadget, HardwareError>;
    async fn remove_gadget(&self, gadget_id: &str) -> Result<(), HardwareError>;
}

#[async_trait]
pub trait SystemdServiceRepository {
    async fn create_service(&self, service: &SystemdService) -> Result<(), HardwareError>;
    async fn enable_service(&self, service: &mut SystemdService) -> Result<(), HardwareError>;
    async fn start_service(&self, service: &mut SystemdService) -> Result<(), HardwareError>;
    async fn stop_service(&self, service: &mut SystemdService) -> Result<(), HardwareError>;
    async fn get_service_state(&self, service_name: &str) -> Result<SystemdService, HardwareError>;
    async fn reload_daemon(&self) -> Result<(), HardwareError>;
}

pub trait UsbGadgetManager: Send + Sync {
    fn configure_as_pro_controller(&self) -> Result<(), SetupError>;
    fn is_gadget_configured(&self) -> Result<bool, SetupError>;
}