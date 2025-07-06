use crate::domain::controller::{ControllerError, ControllerRepository, ControllerSession, ControllerSessionRepository, ProController};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// インメモリのコントローラーリポジトリ実装
pub struct InMemoryControllerRepository {
    controllers: Arc<RwLock<HashMap<String, ProController>>>,
}

impl Default for InMemoryControllerRepository {
    fn default() -> Self {
        Self {
            controllers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl InMemoryControllerRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ControllerRepository for InMemoryControllerRepository {
    async fn create_controller(&self, controller: &ProController) -> Result<(), ControllerError> {
        let mut controllers = self.controllers.write().await;
        if controllers.contains_key(&controller.id) {
            return Err(ControllerError::AlreadyConnected);
        }
        controllers.insert(controller.id.clone(), controller.clone());
        Ok(())
    }

    async fn get_controller(&self, id: &str) -> Result<ProController, ControllerError> {
        let controllers = self.controllers.read().await;
        controllers
            .get(id)
            .cloned()
            .ok_or_else(|| ControllerError::ControllerNotFound(id.to_string()))
    }

    async fn update_controller(&self, controller: &ProController) -> Result<(), ControllerError> {
        let mut controllers = self.controllers.write().await;
        if !controllers.contains_key(&controller.id) {
            return Err(ControllerError::ControllerNotFound(controller.id.clone()));
        }
        controllers.insert(controller.id.clone(), controller.clone());
        Ok(())
    }

    async fn delete_controller(&self, id: &str) -> Result<(), ControllerError> {
        let mut controllers = self.controllers.write().await;
        controllers
            .remove(id)
            .ok_or_else(|| ControllerError::ControllerNotFound(id.to_string()))?;
        Ok(())
    }

    async fn list_controllers(&self) -> Result<Vec<ProController>, ControllerError> {
        let controllers = self.controllers.read().await;
        Ok(controllers.values().cloned().collect())
    }

    async fn connect_controller(&self, controller: &mut ProController) -> Result<(), ControllerError> {
        if controller.is_connected {
            return Err(ControllerError::AlreadyConnected);
        }
        
        // デフォルトのデバイスパスを設定
        if controller.device_path.is_none() {
            controller.connect("/dev/hidg0");
        }
        
        self.update_controller(controller).await
    }

    async fn disconnect_controller(&self, controller: &mut ProController) -> Result<(), ControllerError> {
        if !controller.is_connected {
            return Err(ControllerError::NotConnected);
        }
        
        controller.disconnect();
        self.update_controller(controller).await
    }
}

/// インメモリのセッションリポジトリ実装
pub struct InMemorySessionRepository {
    sessions: Arc<RwLock<HashMap<String, ControllerSession>>>,
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ControllerSessionRepository for InMemorySessionRepository {
    async fn create_session(&self, session: &ControllerSession) -> Result<(), ControllerError> {
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session.id) {
            return Err(ControllerError::SessionAlreadyActive);
        }
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn get_session(&self, id: &str) -> Result<ControllerSession, ControllerError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(id)
            .cloned()
            .ok_or_else(|| ControllerError::SessionNotFound(id.to_string()))
    }

    async fn update_session(&self, session: &ControllerSession) -> Result<(), ControllerError> {
        let mut sessions = self.sessions.write().await;
        if !sessions.contains_key(&session.id) {
            return Err(ControllerError::SessionNotFound(session.id.clone()));
        }
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn delete_session(&self, id: &str) -> Result<(), ControllerError> {
        let mut sessions = self.sessions.write().await;
        sessions
            .remove(id)
            .ok_or_else(|| ControllerError::SessionNotFound(id.to_string()))?;
        Ok(())
    }

    async fn list_sessions(&self) -> Result<Vec<ControllerSession>, ControllerError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    async fn get_active_sessions(&self) -> Result<Vec<ControllerSession>, ControllerError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.is_active)
            .cloned()
            .collect())
    }
}