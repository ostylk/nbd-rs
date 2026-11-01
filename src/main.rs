use anyhow::{Context, Result};
use clap::Parser;
use futures::StreamExt;
use genetlink::{message::RawGenlMessage, new_connection};
use log::{error, info};
use netlink_packet_core::{NetlinkPayload, ParseableParametrized};
use netlink_packet_generic::GenlMessage;
use netlink_packet_nbd::{
    NbdCmd, NbdPayload,
    attributes::{NbdAttrs, NbdSocket, NbdSocketAttrs::SocketFd},
};
use netlink_sys::AsyncSocket;

use crate::{
    cli::{Cli, Commands, DeviceCommands, RemoteCommands, ServerArgs},
    nbd::connect_tcp,
    netlink::Client,
};

mod cli;
mod nbd;
mod netlink;

async fn run(cli: Cli) -> Result<()> {
    info!("Starting up...");

    match cli.cmd {
        Commands::Remote { server, cmd } => run_remote(server, cmd).await,
        Commands::Device { cmd } => run_device(cmd).await,
    }
}

async fn run_device(cmd: DeviceCommands) -> Result<()> {
    let mut client = Client::connect()?;

    match cmd {
        DeviceCommands::List => {
            let devices = client
                .list_devices()
                .await
                .context("Cannot retrieve device list from kernel.")?;
            for device in devices {
                let state = if device.connected {
                    "connected"
                } else {
                    "disconnected"
                };
                println!("/dev/nbd{}\t{state}", device.index);
            }
        }
        DeviceCommands::Disconnect { index } => {
            client
                .disconnect_device(index)
                .await
                .context("Cannot disconnect nbd device.")?;
        }
    }

    Ok(())
}

async fn run_remote(server: ServerArgs, cmd: RemoteCommands) -> Result<()> {
    let (fd, size, flags) = match cmd {
        RemoteCommands::List => {
            let mut client = connect_tcp(&server.host, server.port).await?;
            let exports = client
                .list_exports()
                .await
                .context("Cannot retrieve exports list from nbd server.")?;
            info!("Servers exposes the following exports:");
            for export in exports {
                info!("- {export}");
            }
            return Ok(());
        }
        RemoteCommands::Info { name } => {
            let mut client = connect_tcp(&server.host, server.port).await?;
            let export_name = name.unwrap_or("".to_string());
            let info = client
                .info_export(&export_name)
                .await
                .context(format!("Cannot query info about export {export_name}."))?;

            println!("{info:?}");
            return Ok(());
        }
        RemoteCommands::Go { name } => {
            let mut client = connect_tcp(&server.host, server.port).await?;
            let export_name = name.unwrap_or("".to_string());
            let (size, flags) = client.go(&export_name).await.context(format!(
                "Cannot switch into transmission phase with export {export_name}."
            ))?;
            let fd = client.into_fd()?;
            (fd, size, flags)
        }
    };

    let mut client = Client::connect()?;

    let genlmsg = GenlMessage::from_payload(NbdPayload {
        cmd: NbdCmd::Connect,
        nlas: vec![
            NbdAttrs::Index(0),
            NbdAttrs::SizeBytes(size),
            NbdAttrs::BlockSizeBytes(4096),
            NbdAttrs::DeadConnTimeout(10),
            NbdAttrs::ServerFlags(flags.into_bits().into()),
            NbdAttrs::Sockets(vec![NbdSocket(vec![SocketFd(fd as u32)])]),
        ],
    });

    {
        let resps = client.request(genlmsg).await?;
        let mut resps = std::pin::pin!(resps);
        while let Some(resp) = resps.next().await {
            println!("{:?}", resp?);
        }
    }

    // Socket to listen for shit
    let (mut conn2, _, mut recv) = new_connection()?;
    let mcast_grp = client.resolve_nbd_mcast().await?;
    conn2.socket_mut().socket_mut().bind_auto()?;
    conn2.socket_mut().socket_mut().add_membership(mcast_grp)?;
    tokio::spawn(conn2);

    println!("Waiting for unsolicited messages!");
    loop {
        let (unsol, addr) = recv.recv().await?;
        println!("FROM {addr} UNSOLICITED {unsol:?}");

        if let NetlinkPayload::InnerMessage(RawGenlMessage {
            header, payload, ..
        }) = unsol.payload
        {
            let _pkt = NbdPayload::parse_with_param(payload.as_slice(), header);
        }
    }
}

#[tokio::main]
async fn main() {
    // Init logging
    env_logger::init();
    // Parse cli arguments
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        error!("{e:?}");
        std::process::exit(1);
    }
}
