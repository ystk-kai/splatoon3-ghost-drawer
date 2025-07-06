use crate::interfaces::web::server::create_server;

pub struct RunApplicationUseCase {
    // Dependencies for the application would be injected here
}

impl RunApplicationUseCase {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, host: String, port: u16) -> anyhow::Result<()> {
        // Delegate to the web server module
        create_server(host, port).await
    }
}