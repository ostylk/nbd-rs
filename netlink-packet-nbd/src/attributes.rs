use netlink_packet_core::{Emitable, NLA_F_NESTED, Nla, emit_u32, emit_u64};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NbdAttrs {
    Index(u32),
    SizeBytes(u64),
    BlockSizeBytes(u64),
    Timeout(u64),
    ServerFlags(u64),
    ClientFlags(u64),
    Sockets(Vec<NbdSocket>),
    DeadConnTimeout(u64),
    DeviceList(Vec<NbdDevice>),
    BackendIdentifier(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NbdSocket(pub Vec<NbdSocketAttrs>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NbdSocketAttrs {
    SocketFd(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NbdDevice(pub Vec<NbdDeviceAttrs>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NbdDeviceAttrs {
    Index(u32),
    Connected(u8),
}

impl Nla for NbdAttrs {
    fn value_len(&self) -> usize {
        use NbdAttrs::*;
        match self {
            Index(v) => size_of_val(v),
            SizeBytes(v) => size_of_val(v),
            BlockSizeBytes(v) => size_of_val(v),
            Timeout(v) => size_of_val(v),
            ServerFlags(v) => size_of_val(v),
            ClientFlags(v) => size_of_val(v),
            Sockets(nlas) => nlas.as_slice().buffer_len(),
            DeadConnTimeout(v) => size_of_val(v),
            DeviceList(nlas) => nlas.as_slice().buffer_len(),
            BackendIdentifier(s) => s.len() + 1,
        }
    }

    fn kind(&self) -> u16 {
        use NbdAttrs::*;
        use crate::constants::*;
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
        use NbdAttrs::*;
        match self {
            Index(v) => emit_u32(buffer, *v).unwrap(),
            SizeBytes(v) => emit_u64(buffer, *v).unwrap(),
            BlockSizeBytes(v) => emit_u64(buffer, *v).unwrap(),
            Timeout(v) => emit_u64(buffer, *v).unwrap(),
            ServerFlags(v) => emit_u64(buffer, *v).unwrap(),
            ClientFlags(v) => emit_u64(buffer, *v).unwrap(),
            Sockets(nlas) => nlas.as_slice().emit(buffer),
            DeadConnTimeout(v) => emit_u64(buffer, *v).unwrap(),
            DeviceList(nlas) => nlas.as_slice().emit(buffer),
            BackendIdentifier(s) => {
                buffer[..s.len()].copy_from_slice(s.as_bytes());
                buffer[s.len()] = 0;
            }
        }
    }
}

impl Nla for NbdSocket {
    fn value_len(&self) -> usize {
        self.0.as_slice().buffer_len()
    }

    fn kind(&self) -> u16 {
        use crate::constants::*;
        SOCK_ITEM | NLA_F_NESTED
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        self.0.as_slice().emit(buffer);
    }
}

impl Nla for NbdSocketAttrs {
    fn value_len(&self) -> usize {
        use NbdSocketAttrs::*;
        match self {
            SocketFd(v) => size_of_val(v),
        }
    }

    fn kind(&self) -> u16 {
        use NbdSocketAttrs::*;
        use crate::constants::*;
        match self {
            SocketFd(_) => SOCK_FD,
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        use NbdSocketAttrs::*;
        match self {
            SocketFd(v) => emit_u32(buffer, *v).unwrap(),
        }
    }
}

impl Nla for NbdDevice {
    fn value_len(&self) -> usize {
        self.0.as_slice().buffer_len()
    }

    fn kind(&self) -> u16 {
        use crate::constants::*;
        DEVICE_ITEM | NLA_F_NESTED
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        self.0.as_slice().emit(buffer);
    }
}

impl Nla for NbdDeviceAttrs {
    fn value_len(&self) -> usize {
        use NbdDeviceAttrs::*;
        match self {
            Index(v) => size_of_val(v),
            Connected(v) => size_of_val(v),
        }
    }

    fn kind(&self) -> u16 {
        use NbdDeviceAttrs::*;
        use crate::constants::*;
        match self {
            Index(_) => DEVICE_INDEX,
            Connected(_) => DEVICE_ITEM,
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        use NbdDeviceAttrs::*;
        match self {
            Index(v) => emit_u32(buffer, *v).unwrap(),
            Connected(v) => buffer[0] = *v,
        }
    }
}