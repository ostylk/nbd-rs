use crate::nbd::client::Client;
use anyhow::{Context, Result};
use tokio::net::TcpStream;

pub mod client;
pub mod definitions;
pub mod payload;

pub async fn connect_tcp(host: &str, port: u16) -> Result<Client> {
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(&addr)
        .await
        .with_context(|| format!("Could not connect to {addr}"))?;
    let client = Client::negotiate(stream)
        .await
        .context("nbd server handshake/negotiation failed. Is it up to date?")?;
    Ok(client)
}
