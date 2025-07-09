use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "splatoon3-ghost-drawer",
    author = "Splatoon3 Ghost Drawer Team",
    version,
    about = "A drawing robot that creates art on Splatoon 3",
    long_about = "A drawing robot that creates art on Splatoon 3 by emulating a Nintendo Switch Pro Controller"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Perform first-time system setup (requires root privileges)
    Setup {
        /// Force setup even if already configured
        #[arg(short, long)]
        force: bool,
    },
    /// Run the main application and web server
    Run {
        /// Port to bind the web server to
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Host to bind the web server to
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Remove all configurations created by setup (requires root privileges)
    Cleanup,
    /// Show system and connection information
    #[command(name = "info")]
    Info {
        /// Show verbose output with detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test controller connection and functionality
    #[command(name = "test")]
    Test {
        /// Test duration in seconds (0 for manual control)
        #[arg(short, long, default_value = "10")]
        duration: u16,
        /// Test mode: basic, buttons, sticks, or interactive
        #[arg(short, long, default_value = "basic")]
        mode: String,
    },
    /// Diagnose connection issues with detailed information
    #[command(name = "diagnose")]
    Diagnose,
    /// Fix USB connection issues (mainly for Orange Pi Zero 2W)
    #[command(name = "fix-connection")]
    FixConnection,
    /// [Internal] Configure USB gadget via configfs (called by systemd)
    #[command(name = "_internal_configure_gadget", hide = true)]
    InternalConfigureGadget,
}
