use std::os::fd::{RawFd};

use anyhow::Error;
use futures::StreamExt;
use genetlink::{GenetlinkHandle, new_connection};
use netlink_packet_core::{NLM_F_ACK, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload};
use netlink_packet_generic::{GenlMessage, ctrl::{GenlCtrl, GenlCtrlCmd, nlas::GenlCtrlAttrs}};
use netlink_packet_nbd::{NbdCmd, NbdPayload, attributes::{NbdAttrs, NbdSocket, NbdSocketAttrs::SocketFd}, constants::{GENL_FAMILY_NAME, GENL_MCAST_GROUP_NAME}};
use netlink_sys::AsyncSocket;
use tokio::net::TcpStream;

use crate::nbd::{negotiate, protocol::TransmissionFlags};

mod nbd;

async fn connect_nbd() -> Result<(RawFd, u64, TransmissionFlags), Error> {
    let stream = TcpStream::connect("127.0.0.1:8002").await?;
    let client = negotiate(stream).await?;

    let (size, flags, fd) = client.export_name("").await?;
    println!("Bound to device with size {size} and flags {flags:?}");

    Ok((fd, size, flags))
}

async fn get_mcast_grp(handle: &mut GenetlinkHandle) -> Result<u32, Error> {
    let genlmsg = GenlMessage::from_payload(GenlCtrl {
        cmd: GenlCtrlCmd::GetFamily,
        nlas: vec![
            GenlCtrlAttrs::FamilyName(GENL_FAMILY_NAME.into()),
        ],
    });
    let mut nlhdr = NetlinkHeader::default();
    nlhdr.flags = NLM_F_REQUEST | NLM_F_ACK;
    let nlmsg = NetlinkMessage::new(nlhdr, genlmsg.into());

    let mut resps = handle.request(nlmsg).await?;
    if let Some(resp) = resps.next().await {
        let resp = resp?;
        match resp.payload {
            NetlinkPayload::Done(_) => todo!(),
            NetlinkPayload::Error(_) => todo!(),
            NetlinkPayload::Noop => todo!(),
            NetlinkPayload::Overrun(_) => todo!(),
            NetlinkPayload::InnerMessage(GenlMessage { payload, .. }) => {
                let GenlCtrlAttrs::McastGroups(nlas) = payload.nlas.iter().find(|elem| matches!(elem, GenlCtrlAttrs::McastGroups(_))).unwrap() else { unreachable!() };

                let mut mcast_grp_idx = None;
                'outer: for (idx, ele) in nlas.iter().enumerate() {
                    for ite in ele {
                        if let netlink_packet_generic::ctrl::nlas::McastGrpAttrs::Name(name) = ite
                            && name == GENL_MCAST_GROUP_NAME {
                                mcast_grp_idx = Some(idx);
                                break 'outer;
                            }
                    }
                }

                let nlas = &nlas[mcast_grp_idx.expect("no multicast group found")];
                for ite in nlas {
                    if let netlink_packet_generic::ctrl::nlas::McastGrpAttrs::Id(id) = ite {
                        return Ok(*id);
                    }
                }
            },
            _ => todo!(),
        }
    }

   Err(Error::msg("didnt find shit"))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (fd, size, flags) = connect_nbd().await?;

    let (conn, mut handle, _) = new_connection()?;

    let (mut conn2, _, mut recv) = new_connection()?;
    _ = conn2.socket_mut().socket_mut().bind_auto()?;
    let mcast_grp = get_mcast_grp(&mut handle).await?;
    conn2.socket_mut().socket_mut().add_membership(mcast_grp)?;

    tokio::spawn(conn);
    tokio::spawn(conn2);

    println!("Figured out {mcast_grp}");

    let genlmsg = GenlMessage::from_payload(NbdPayload {
        cmd: NbdCmd::Connect,
        nlas: vec![
            NbdAttrs::Index(0),
            NbdAttrs::SizeBytes(size),
            NbdAttrs::BlockSizeBytes(4096),
            NbdAttrs::DeadConnTimeout(10),
            NbdAttrs::ServerFlags(flags.into_bits().into()),
            NbdAttrs::Sockets(vec![
                NbdSocket(vec![
                    SocketFd(fd as u32),
                ]),
            ]),
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
    }
}
