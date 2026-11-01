use clap::{Args, Parser, Subcommand};

use crate::nbd::definitions::PORT;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Talks to an nbd server
    Remote {
        #[command(flatten)]
        server: ServerArgs,
        #[command(subcommand)]
        cmd: RemoteCommands,
    },
    /// Manages local nbd block devices
    Device {
        #[command(subcommand)]
        cmd: DeviceCommands,
    },
}

#[derive(Args)]
pub struct ServerArgs {
    #[arg(short = 'H', long, global = true, default_value_t = "127.0.0.1".into())]
    pub host: String,
    #[arg(short, long, global = true, default_value_t = PORT, value_parser = clap::value_parser!(u16).range(1..))]
    pub port: u16,
}

#[derive(Subcommand)]
pub enum RemoteCommands {
    /// Lists all exported shares by the nbd server (if possible)
    List,
    /// Queries more specific information about an export
    Info { name: Option<String> },
    /// Binds and monitors the nbd connection
    Go { name: Option<String> },
}

#[derive(Subcommand)]
pub enum DeviceCommands {
    /// Lists all /dev/nbdX currently bound
    List,
    /// Disconnects an nbd device
    Disconnect { index: u32 },
}
