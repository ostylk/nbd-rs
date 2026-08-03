use anyhow::Error;
use futures::StreamExt;
use genetlink::new_connection;
use netlink_packet_core::{NLM_F_ACK, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload};
use netlink_packet_generic::GenlMessage;
use netlink_packet_nbd::{NbdCmd, NbdPayload, attributes::{NbdAttrs, NbdSocket, NbdSocketAttrs::SocketFd}};
use tokio::net::TcpStream;

use crate::nbd::negotiate;

mod nbd;

async fn connect_nbd() -> Result<(), Error> {
    let stream = TcpStream::connect("127.0.0.1:8002").await?;
    let mut client = negotiate(stream).await?;

    println!("Requesting exports");
    let exports = client.list_exports().await?;
    for export in exports {
        println!("Found export: {}", export);
    }
    let (size, flags) = client.export_name("test1").await?;
    println!("Bound to device with size {size} and flags {flags:?}");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    connect_nbd().await?;

    let (conn, mut handle, mut recv) = new_connection()?;
    tokio::spawn(conn);

    let genlmsg = GenlMessage::from_payload(NbdPayload {
        cmd: NbdCmd::Connect,
        nlas: vec![
            NbdAttrs::Index(0),
            NbdAttrs::SizeBytes(1024 * 1024 * 1024),
            NbdAttrs::BlockSizeBytes(4096),
            NbdAttrs::Sockets(vec![
                NbdSocket(vec![
                    SocketFd(5),
                ]),
            ]),
        ],
    });

    let mut nlhdr = NetlinkHeader::default();
    nlhdr.flags = NLM_F_REQUEST | NLM_F_ACK;
    let nlmsg = NetlinkMessage::new(nlhdr, genlmsg.into());

    let mut resps = handle.request(nlmsg).await?;
    while let Some(resp) = resps.next().await {
        let resp = resp?;
        match resp.payload {
            NetlinkPayload::InnerMessage(genlmsg) => {
                println!("{:?}", genlmsg);
            }
            NetlinkPayload::Error(err) => {
                eprintln!("Received a netlink error message: {err:?}");
                panic!("{}", err);
            }
            _ => {}
        }
    }

    loop {
        let (unsol, addr) = recv.recv().await?;
        println!("FROM {addr} UNSOLICITED {unsol:?}");
    }
}
