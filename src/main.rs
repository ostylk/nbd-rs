use std::os::fd::RawFd;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use futures::StreamExt;
use genetlink::{message::RawGenlMessage, new_connection};
use log::{error, info};
use netlink_packet_core::{
    NLM_F_ACK, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload, ParseableParametrized,
};
use netlink_packet_generic::GenlMessage;
use netlink_packet_nbd::{
    NbdCmd, NbdPayload,
    attributes::{NbdAttrs, NbdSocket, NbdSocketAttrs::SocketFd},
    constants::GENL_MCAST_GROUP_NAME,
};
use netlink_sys::AsyncSocket;
use tokio::net::TcpStream;

use crate::{
    cli::Cli,
    nbd::{client::Client, connect_tcp, definitions::TransmissionFlags},
};

mod cli;
mod nbd;

async fn connect_nbd(host: &str, port: u16) -> Result<(RawFd, u64, TransmissionFlags)> {
    let stream = TcpStream::connect(format!("{host}:{port}")).await?;
    let client = Client::negotiate(stream).await?;

    let (size, flags, fd) = client.export_name("").await?;
    println!("Bound to device with size {size} and flags {flags:?}");

    Ok((fd, size, flags))
}

async fn run(cli: Cli) -> Result<()> {
    info!("Starting up...");

    match cli.cmd {
        cli::Commands::List => {
            let mut client = connect_tcp(&cli.host, cli.port).await?;
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
        cli::Commands::Info { name } => {
            let mut client = connect_tcp(&cli.host, cli.port).await?;
            let info = client
                .info_export(&name.unwrap_or("".to_string()))
                .await
                .context("Cannot query info about export  .")?;

            println!("{info:?}");
            return Ok(());
        }
        cli::Commands::Go => {}
    }

    let (conn, mut handle, _) = new_connection()?;
    tokio::spawn(conn);

    // Figure out multicast group id and setup second socket to listen for nbd events
    let family = handle.resolve_family::<NbdPayload>().await?;
    let mcast_grp = family
        .multicast_groups
        .get(GENL_MCAST_GROUP_NAME)
        .ok_or(anyhow!("No nbd multicast group found"))?;
    let (mut conn2, _, mut recv) = new_connection()?;
    conn2.socket_mut().socket_mut().bind_auto()?;
    conn2.socket_mut().socket_mut().add_membership(*mcast_grp)?;
    tokio::spawn(conn2);

    let (fd, size, flags) = connect_nbd(&cli.host, cli.port).await?;

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

    let mut nlhdr = NetlinkHeader::default();
    nlhdr.flags = NLM_F_REQUEST | NLM_F_ACK;
    let nlmsg = NetlinkMessage::new(nlhdr, genlmsg.into());

    let mut resps = handle.request(nlmsg).await?;
    if let Some(resp) = resps.next().await {
        let resp = resp?;
        match resp.payload {
            NetlinkPayload::InnerMessage(genlmsg) => {
                println!("{:?}", genlmsg);
            }
            NetlinkPayload::Error(err) => {
                eprintln!("Received a netlink error message: {err:?}");
            }
            _ => {}
        }
    }

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
