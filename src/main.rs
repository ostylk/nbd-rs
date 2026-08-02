use anyhow::Error;
use futures::StreamExt;
use genetlink::new_connection;
use netlink_packet_core::{NLM_F_ACK, NLM_F_DUMP, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload};
use netlink_packet_generic::{GenlMessage, ctrl::{GenlCtrl, GenlCtrlCmd}};
use netlink_packet_nbd::{NbdCmd, NbdPayload, attributes::{NbdAttrs, NbdSocket, NbdSocketAttrs::SocketFd}};

#[tokio::main]
async fn main() -> Result<(), Error> {
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

    Ok(())
}
