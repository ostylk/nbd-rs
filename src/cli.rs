use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'H', long, default_value_t = "127.0.0.1".into())]
    pub host: String,
    #[arg(short, long, default_value_t = 8002, value_parser = clap::value_parser!(u16).range(1..))]
    pub port: u16,

    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Lists all exported shares by the nbd server (if possible)
    List,
    /// Binds and monitors the nbd connection
    Go,
}