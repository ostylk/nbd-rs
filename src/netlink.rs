use anyhow::{Context, Result, anyhow};
use futures::{Stream, StreamExt};
use genetlink::{GenetlinkHandle, new_connection};
use netlink_packet_core::{
    NLM_F_ACK, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload,
};
use netlink_packet_generic::GenlMessage;
use netlink_packet_nbd::{NbdPayload, constants::GENL_MCAST_GROUP_NAME};
use tokio::task::JoinHandle;

pub struct Client {
    handle: GenetlinkHandle,
    #[allow(unused)]
    task: JoinHandle<()>,
}

impl Client {
    pub fn connect() -> Result<Client> {
        let (conn, handle, _) = new_connection()?;
        let task = tokio::spawn(conn);

        Ok(Client { task, handle })
    }

    pub async fn resolve_nbd_mcast(&self) -> Result<u32> {
        let family = self.handle.resolve_family::<NbdPayload>().await?;
        let mcast_group = family
            .multicast_groups
            .get(GENL_MCAST_GROUP_NAME)
            .context("No nbd multicast group found. Is the `nbd` kernel module loaded?")?;

        Ok(*mcast_group)
    }

    pub async fn request(
        &mut self,
        payload: GenlMessage<NbdPayload>,
    ) -> Result<impl Stream<Item = Result<GenlMessage<NbdPayload>>>> {
        let mut nlhdr = NetlinkHeader::default();
        nlhdr.flags = NLM_F_REQUEST | NLM_F_ACK;
        let nlmsg = NetlinkMessage::new(nlhdr, payload.into());

        let resps = self.handle.request(nlmsg).await?;
        Ok(resps.filter_map(|rep| async move {
            match rep {
                Ok(NetlinkMessage {
                    payload: NetlinkPayload::InnerMessage(msg),
                    ..
                }) => Some(Ok(msg)),
                Ok(NetlinkMessage {
                    payload: NetlinkPayload::Error(err),
                    ..
                }) => Some(Err(anyhow!(err))),
                Ok(_) => None,
                Err(err) => Some(Err(anyhow!(err))),
            }
        }))
    }
}
