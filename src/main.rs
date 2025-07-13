mod cli;

use crate::cli::{Cli, Commands};
use clap::Parser;
use std::sync::Arc;
use tracing::{error, info};

use splatoon3_ghost_drawer::application::use_cases::{
    CleanupGadgetUseCase, CleanupSystemUseCase, ConfigureUsbGadgetUseCase,
    DiagnoseConnectionUseCase, FixConnectionUseCase, FixPermissionsUseCase, RunApplicationUseCase,
    SetupSystemUseCase, ShowSystemInfoUseCase, TestControllerUseCase,
};
use splatoon3_ghost_drawer::debug::{DebugConfig, init_logging};
use splatoon3_ghost_drawer::infrastructure::hardware::linux_usb_gadget_manager::LinuxUsbGadgetManager;
use splatoon3_ghost_drawer::infrastructure::setup::{
    LinuxBoardDetector, LinuxBootConfigurator, LinuxSystemdManager,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let debug_config = DebugConfig {
        enable_file_logging: false,
        log_directory: "/tmp/splatoon3-ghost-drawer-logs".to_string(),
        ..DebugConfig::default()
    };
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
            let use_case =
                SetupSystemUseCase::new(board_detector, boot_configurator, systemd_manager);

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
        Commands::Cleanup { gadget_only } => {
            info!("Executing cleanup command (gadget_only: {})", gadget_only);

            if gadget_only {
                // USB Gadgetのみクリーンアップ
                let use_case = CleanupGadgetUseCase::new();
                match use_case.execute() {
                    Ok(_) => {
                        println!("✅ USB Gadget cleanup completed successfully!");
                    }
                    Err(e) => {
                        error!("Gadget cleanup failed: {}", e);
                        eprintln!("❌ Gadget cleanup failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // システム全体のクリーンアップ
                let use_case =
                    CleanupSystemUseCase::new(board_detector, boot_configurator, systemd_manager);

                match use_case.execute() {
                    Ok(_) => {
                        println!("✅ System cleanup completed successfully!");
                        println!("⚠️  Please reboot your device for the changes to take effect.");
                        println!("    Run: sudo reboot");
                    }
                    Err(e) => {
                        error!("Cleanup failed: {}", e);
                        eprintln!("❌ Cleanup failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Info { verbose } => {
            info!("Showing system information...");
            let use_case = ShowSystemInfoUseCase::new(board_detector, usb_gadget_manager);

            match use_case.execute(verbose) {
                Ok(_) => {
                    info!("System information displayed successfully");
                }
                Err(e) => {
                    error!("Failed to show system info: {}", e);
                    eprintln!("❌ Failed to show system info: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Test { duration, mode } => {
            info!("Starting controller test...");

            // Check if we have proper permissions
            if !nix::unistd::Uid::effective().is_root() {
                eprintln!("❌ Error: This command requires root privileges.");
                eprintln!("   Please run with sudo: sudo splatoon3-ghost-drawer test");
                std::process::exit(1);
            }

            // Create controller emulator
            use splatoon3_ghost_drawer::infrastructure::hardware::linux_hid_controller::LinuxHidController;
            let controller = Arc::new(LinuxHidController::new());
            let use_case = TestControllerUseCase::new(controller);

            match use_case.execute(duration, &mode).await {
                Ok(_) => {
                    println!("✅ Controller test completed successfully!");
                }
                Err(e) => {
                    error!("Controller test failed: {}", e);
                    eprintln!("❌ Controller test failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Diagnose => {
            info!("Running connection diagnostics...");

            // Check if we have proper permissions
            if !nix::unistd::Uid::effective().is_root() {
                eprintln!("❌ Error: This command requires root privileges.");
                eprintln!("   Please run with sudo: sudo splatoon3-ghost-drawer diagnose");
                std::process::exit(1);
            }

            let use_case = DiagnoseConnectionUseCase::new();
            match use_case.execute() {
                Ok(_) => {
                    info!("Diagnostics completed");
                }
                Err(e) => {
                    error!("Diagnostics failed: {}", e);
                    eprintln!("❌ Diagnostics failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::FixConnection => {
            info!("Fixing USB connection...");

            // Check if we have proper permissions
            if !nix::unistd::Uid::effective().is_root() {
                eprintln!("❌ Error: This command requires root privileges.");
                eprintln!("   Please run with sudo: sudo splatoon3-ghost-drawer fix-connection");
                std::process::exit(1);
            }

            let use_case = FixConnectionUseCase::new(usb_gadget_manager.clone());
            match use_case.execute() {
                Ok(_) => {
                    println!("✅ Connection fix completed!");
                }
                Err(e) => {
                    error!("Connection fix failed: {}", e);
                    eprintln!("❌ Connection fix failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::FixPermissions => {
            info!("Fixing HID device permissions...");

            // Check if we have proper permissions
            if !nix::unistd::Uid::effective().is_root() {
                eprintln!("❌ Error: This command requires root privileges.");
                eprintln!("   Please run with sudo: sudo splatoon3-ghost-drawer fix-permissions");
                std::process::exit(1);
            }

            let use_case = FixPermissionsUseCase::new(usb_gadget_manager.clone());
            match use_case.execute() {
                Ok(_) => {
                    println!("✅ Permissions fix completed!");
                }
                Err(e) => {
                    error!("Permissions fix failed: {}", e);
                    eprintln!("❌ Permissions fix failed: {}", e);
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
