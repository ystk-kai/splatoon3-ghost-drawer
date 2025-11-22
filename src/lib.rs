// Application Layer
pub mod application {
    pub mod use_cases {
        pub mod cleanup_gadget;
        pub mod cleanup_system;
        pub mod configure_usb_gadget;
        pub mod diagnose_connection;
        pub mod fix_connection;
        pub mod fix_permissions_use_case;
        pub mod paint_artwork;
        pub mod run_application;
        pub mod setup_system;
        pub mod setup_usb_gadget;
        pub mod show_system_info;
        pub mod test_controller;

        // Re-exports
        pub use cleanup_gadget::*;
        pub use cleanup_system::*;
        pub use configure_usb_gadget::*;
        pub use diagnose_connection::*;
        pub use fix_connection::*;
        pub use fix_permissions_use_case::*;
        pub use paint_artwork::*;
        pub use run_application::*;
        pub use setup_system::*;
        pub use setup_usb_gadget::*;
        pub use show_system_info::*;
        pub use test_controller::*;
    }
}

// Debug utilities
pub mod debug;

// Domain Layer
pub mod domain {
    pub mod artwork {
        pub mod entities;
        pub mod repositories;
        pub mod services;
        pub mod value_objects;
    }

    pub mod controller {
        pub mod emulator;
        pub mod entities;
        pub mod errors;
        pub mod repositories;
        pub mod value_objects;

        // Re-exports
        pub use emulator::*;
        pub use entities::*;
        pub use errors::*;
        pub use repositories::*;
        pub use value_objects::*;
    }

    pub mod events;

    pub mod hardware {
        pub mod entities;
        pub mod errors;
        pub mod repositories;
        pub mod value_objects;

        // Re-exports
        pub use entities::*;
        pub use errors::*;
        pub use repositories::*;
        pub use value_objects::*;
    }

    pub mod painting {
        pub mod services;
        pub mod value_objects;

        // Re-exports
        pub use services::*;
        pub use value_objects::*;
    }

    pub mod setup {
        pub mod entities;
        pub mod repositories;

        // Re-exports
        pub use entities::*;
        pub use repositories::*;
    }

    pub mod shared {
        pub mod events;
        pub mod value_objects;

        // Re-exports
        pub use events::*;
        pub use value_objects::*;
    }
}

// Infrastructure Layer
pub mod infrastructure {
    pub mod hardware {
        pub mod board_detector;
        pub mod controller_repository;
        pub mod linux_hid_controller;
        pub mod linux_hid_device;
        pub mod linux_usb_gadget;
        pub mod linux_usb_gadget_manager;
        pub mod mock_controller;
        pub mod systemd_service;
    }

    pub mod setup {
        mod linux_board_detector;
        mod linux_boot_configurator;
        mod linux_systemd_manager;

        // Re-exports
        pub use linux_board_detector::*;
        pub use linux_boot_configurator::*;
        pub use linux_systemd_manager::*;
    }
}

// Interface Layer
pub mod interfaces {
    pub mod web {
        mod artwork_handlers;
        pub mod embedded_assets;
        mod error_response;
        mod handlers;
        pub mod log_streamer;
        mod models;
        pub mod server;

        // Internal re-exports
        pub(crate) use artwork_handlers::*;
        pub(crate) use handlers::*;
    }
}

// CLI
pub mod cli;

// 公開API
pub use domain::*;

// エラー型の定義
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// アプリケーション全体の設定
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            debug: true,
        }
    }
}
