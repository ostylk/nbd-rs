use clap::{Parser, Subcommand};

use crate::nbd::definitions::PORT;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'H', long, default_value_t = "127.0.0.1".into())]
    pub host: String,
    #[arg(short, long, default_value_t = PORT, value_parser = clap::value_parser!(u16).range(1..))]
    pub port: u16,

    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Lists all exported shares by the nbd server (if possible)
    List,
    /// Queries more specific information about an export
    Info,
    /// Binds and monitors the nbd connection
    Go,
}
