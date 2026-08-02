use netlink_packet_core::{DecodeError, Emitable};
use netlink_packet_generic::GenlFamily;

use crate::attributes::NbdAttrs;

pub mod attributes;
pub mod constants;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbdCmd {
    Connect,
    Disconnect,
    Reconfigure,
    LinkDead,
    Status,
}

impl From<NbdCmd> for u8 {
    fn from(cmd: NbdCmd) -> u8 {
        use NbdCmd::*;
        use constants::*;
        match cmd {
            Connect => CMD_CONNECT,
            Disconnect => CMD_DISCONNECT,
            Reconfigure => CMD_RECONFIGURE,
            LinkDead => CMD_LINK_DEAD,
            Status => CMD_STATUS,
        }
    }
}

impl TryFrom<u8> for NbdCmd {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use NbdCmd::*;
        use constants::*;
        Ok(match value {
            CMD_CONNECT => Connect,
            CMD_DISCONNECT => Disconnect,
            CMD_RECONFIGURE => Reconfigure,
            CMD_LINK_DEAD => LinkDead,
            CMD_STATUS => Status,
            cmd => {
                return Err(DecodeError::from(format!(
                    "Unknown nbd command: {cmd}"
                )))
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NbdPayload {
    /// Command code of this message
    pub cmd: NbdCmd,
    /// Netlink attributes in this message
    pub nlas: Vec<NbdAttrs>,
}

impl GenlFamily for NbdPayload {
    fn family_name() -> &'static str {
        constants::GENL_FAMILY_NAME
    }

    fn command(&self) -> u8 {
        self.cmd.into()
    }

    fn version(&self) -> u8 {
        constants::GENL_VERSION
    }
}

impl Emitable for NbdPayload {
    fn buffer_len(&self) -> usize {
        self.nlas.as_slice().buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.nlas.as_slice().emit(buffer)
    }
}

#[cfg(test)]
mod tests {
    
}
