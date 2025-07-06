mod cli;

use crate::cli::{Cli, Commands};
use clap::Parser;
use std::sync::Arc;
use tracing::{error, info};

use splatoon3_ghost_drawer::application::use_cases::{
    ConfigureUsbGadgetUseCase, RunApplicationUseCase, SetupSystemUseCase,
};
use splatoon3_ghost_drawer::debug::{init_logging, DebugConfig};
use splatoon3_ghost_drawer::infrastructure::hardware::linux_usb_gadget_manager::LinuxUsbGadgetManager;
use splatoon3_ghost_drawer::infrastructure::setup::{
    LinuxBoardDetector, LinuxBootConfigurator, LinuxSystemdManager,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let debug_config = DebugConfig::default();
    if let Err(e) = init_logging(&debug_config) {
        eprintln!("Failed to initialize logging: {}", e);
    }

    let cli = Cli::parse();

    // Dependency injection
    let board_detector = Arc::new(LinuxBoardDetector::new());
    let boot_configurator = Arc::new(LinuxBootConfigurator::new());
    let systemd_manager = Arc::new(LinuxSystemdManager::new());
    let usb_gadget_manager = Arc::new(LinuxUsbGadgetManager::new());

    match cli.command {
        Commands::Setup { force } => {
            info!("Executing setup command...");
            let use_case = SetupSystemUseCase::new(
                board_detector,
                boot_configurator,
                systemd_manager,
            );
            
            match use_case.execute(force) {
                Ok(_) => {
                    println!("✅ System setup completed successfully!");
                    println!("⚠️  Please reboot your device for the changes to take effect.");
                    println!("    Run: sudo reboot");
                }
                Err(e) => {
                    error!("Setup failed: {}", e);
                    eprintln!("❌ Setup failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Run { port, host } => {
            info!("Starting application...");
            let use_case = RunApplicationUseCase::new();
            
            match use_case.execute(host, port).await {
                Ok(_) => {
                    info!("Application terminated normally");
                }
                Err(e) => {
                    error!("Application failed: {}", e);
                    eprintln!("❌ Application failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::InternalConfigureGadget => {
            info!("Configuring USB gadget...");
            let use_case = ConfigureUsbGadgetUseCase::new(usb_gadget_manager);
            
            match use_case.execute() {
                Ok(_) => {
                    info!("USB gadget configured successfully");
                }
                Err(e) => {
                    error!("USB gadget configuration failed: {}", e);
                    eprintln!("❌ USB gadget configuration failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}