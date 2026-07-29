use anyhow::Error;
use futures::StreamExt;
use genetlink::new_connection;
use netlink_packet_core::{NLM_F_ACK, NLM_F_DUMP, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage};
use netlink_packet_generic::{GenlMessage, ctrl::{GenlCtrl, GenlCtrlCmd}};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (conn, mut handle, mut recv) = new_connection()?;

    tokio::spawn(conn);

    let genlmsg = GenlMessage::from_payload(GenlCtrl {
        cmd: GenlCtrlCmd::GetFamily,
        nlas: vec![],
    });

    let mut nlhdr = NetlinkHeader::default();
    nlhdr.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_DUMP;
    let nlmsg = NetlinkMessage::new(nlhdr, genlmsg.into());

    let mut resps = handle.request(nlmsg).await?;
    while let Some(resp) = resps.next().await {
        let resp = resp?;
        println!("--- HEADER flags {} ---", resp.header.flags);
        println!("{:?}", resp.payload);
        println!("\n");
    }

    loop {
        let (unsol, addr) = recv.recv().await?;
        println!("FROM {addr} UNSOLICITED {unsol:?}");
    }

    Ok(())
}
