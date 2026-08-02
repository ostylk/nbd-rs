use netlink_packet_core::{DecodeError, Emitable};
use netlink_packet_generic::GenlFamily;

pub mod constants;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenlNbdCmd {
    Connect,
    Disconnect,
    Reconfigure,
    LinkDead,
    Status,
}

impl From<GenlNbdCmd> for u8 {
    fn from(cmd: GenlNbdCmd) -> u8 {
        use GenlNbdCmd::*;
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

impl TryFrom<u8> for GenlNbdCmd {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use GenlNbdCmd::*;
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
pub struct GenlNbd {
    /// Command code of this message
    pub cmd: GenlNbdCmd,
    /// Netlink attributes in this message
    pub nlas: Vec<GenlNbdAttrs>,
}

impl GenlFamily for GenlNbd {
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

impl Emitable for GenlNbd {
    fn buffer_len(&self) -> usize {
        todo!()
    }

    fn emit(&self, buffer: &mut [u8]) {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenlNbdAttrs {

}

#[cfg(test)]
mod tests {
    
}
