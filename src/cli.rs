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
    /// [Internal] Configure USB gadget via configfs (called by systemd)
    #[command(name = "_internal_configure_gadget", hide = true)]
    InternalConfigureGadget,
}