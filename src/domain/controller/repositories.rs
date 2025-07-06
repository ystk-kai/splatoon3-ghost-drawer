use super::{
    ControllerError, ControllerMapping, ControllerSession, HidReport,
    ProController,
};
use async_trait::async_trait;

#[async_trait]
pub trait ControllerRepository {
    async fn create_controller(&self, controller: &ProController) -> Result<(), ControllerError>;
    async fn get_controller(&self, id: &str) -> Result<ProController, ControllerError>;
    async fn update_controller(&self, controller: &ProController) -> Result<(), ControllerError>;
    async fn delete_controller(&self, id: &str) -> Result<(), ControllerError>;
    async fn list_controllers(&self) -> Result<Vec<ProController>, ControllerError>;
    async fn connect_controller(&self, controller: &mut ProController) -> Result<(), ControllerError>;
    async fn disconnect_controller(&self, controller: &mut ProController) -> Result<(), ControllerError>;
}

#[async_trait]
pub trait HidDeviceRepository {
    async fn write_report(&self, device_path: &str, report: &HidReport) -> Result<(), ControllerError>;
    async fn read_report(&self, device_path: &str) -> Result<HidReport, ControllerError>;
    async fn open_device(&self, device_path: &str) -> Result<(), ControllerError>;
    async fn close_device(&self, device_path: &str) -> Result<(), ControllerError>;
    async fn list_devices(&self) -> Result<Vec<String>, ControllerError>;
}

#[async_trait]
pub trait ControllerSessionRepository {
    async fn create_session(&self, session: &ControllerSession) -> Result<(), ControllerError>;
    async fn get_session(&self, id: &str) -> Result<ControllerSession, ControllerError>;
    async fn update_session(&self, session: &ControllerSession) -> Result<(), ControllerError>;
    async fn delete_session(&self, id: &str) -> Result<(), ControllerError>;
    async fn list_sessions(&self) -> Result<Vec<ControllerSession>, ControllerError>;
    async fn get_active_sessions(&self) -> Result<Vec<ControllerSession>, ControllerError>;
}

#[async_trait]
pub trait ControllerMappingRepository {
    async fn create_mapping(&self, mapping: &ControllerMapping) -> Result<(), ControllerError>;
    async fn get_mapping(&self, id: &str) -> Result<ControllerMapping, ControllerError>;
    async fn update_mapping(&self, mapping: &ControllerMapping) -> Result<(), ControllerError>;
    async fn delete_mapping(&self, id: &str) -> Result<(), ControllerError>;
    async fn list_mappings(&self) -> Result<Vec<ControllerMapping>, ControllerError>;
    async fn get_mappings_by_artwork(&self, artwork_id: &str) -> Result<Vec<ControllerMapping>, ControllerError>;
}