use netlink_packet_core::{NLA_F_NESTED, Nla, emit_u32, emit_u64};

use crate::constants::{ATTR_BACKEND_IDENTIFIER, ATTR_BLOCK_SIZE_BYTES, ATTR_CLIENT_FLAGS, ATTR_DEAD_CONN_TIMEOUT, ATTR_DEVICE_LIST, ATTR_INDEX, ATTR_SERVER_FLAGS, ATTR_SIZE_BYTES, ATTR_SOCKETS, ATTR_TIMEOUT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenlNbdAttrs {
    Index(u32),
    SizeBytes(u64),
    BlockSizeBytes(u64),
    Timeout(u64),
    ServerFlags(u64),
    ClientFlags(u64),
    Sockets(Vec<Vec<NbdSocketAttrs>>),
    DeadConnTimeout(u64),
    DeviceList(Vec<Vec<NbdDeviceAttrs>>),
    BackendIdentifier(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NbdSocketAttrs {
    SocketFd(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NbdDeviceAttrs {
    Index(u32),
    Connected(u8),
}

impl Nla for GenlNbdAttrs {
    fn value_len(&self) -> usize {
        use GenlNbdAttrs::*;
        match self {
            Index(v) => size_of_val(v),
            SizeBytes(v) => size_of_val(v),
            BlockSizeBytes(v) => size_of_val(v),
            Timeout(v) => size_of_val(v),
            ServerFlags(v) => size_of_val(v),
            ClientFlags(v) => size_of_val(v),
            Sockets(_v) => todo!("value_len for SOCKETS"),
            DeadConnTimeout(v) => size_of_val(v),
            DeviceList(_v) => todo!("value_len for DEVICE_LIST"),
            BackendIdentifier(s) => s.len() + 1,
        }
    }

    fn kind(&self) -> u16 {
        use GenlNbdAttrs::*;
        match self {
            Index(_) => ATTR_INDEX,
            SizeBytes(_) => ATTR_SIZE_BYTES,
            BlockSizeBytes(_) => ATTR_BLOCK_SIZE_BYTES,
            Timeout(_) => ATTR_TIMEOUT,
            ServerFlags(_) => ATTR_SERVER_FLAGS,
            ClientFlags(_) => ATTR_CLIENT_FLAGS,
            Sockets(_) => ATTR_SOCKETS | NLA_F_NESTED,
            DeadConnTimeout(_) => ATTR_DEAD_CONN_TIMEOUT,
            DeviceList(_) => ATTR_DEVICE_LIST | NLA_F_NESTED,
            BackendIdentifier(_) => ATTR_BACKEND_IDENTIFIER,
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        use GenlNbdAttrs::*;
        match self {
            Index(v) => emit_u32(buffer, *v).unwrap(),
            SizeBytes(v) => emit_u64(buffer, *v).unwrap(),
            BlockSizeBytes(v) => emit_u64(buffer, *v).unwrap(),
            Timeout(v) => emit_u64(buffer, *v).unwrap(),
            ServerFlags(v) => emit_u64(buffer, *v).unwrap(),
            ClientFlags(v) => emit_u64(buffer, *v).unwrap(),
            Sockets(_) => todo!("SERIALIZE SOCKET LIST"),
            DeadConnTimeout(v) => emit_u64(buffer, *v).unwrap(),
            DeviceList(_) => todo!("SERIALIZE DEVICE LIST"),
            BackendIdentifier(s) => {
                buffer[..s.len()].copy_from_slice(s.as_bytes());
                buffer[s.len()] = 0;
            }
        }
    }
}