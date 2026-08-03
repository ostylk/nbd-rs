use anyhow::{Result, anyhow, ensure};
use bitfields::{bitfield, bitflag};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::nbd::Negotiation::{NewStyle, OldStyle};

const INIT_PASSWD: &[u8; 8] = b"NBDMAGIC";
const OLD_MAGIC: &[u8; 8] = &0x00420281861253_u64.to_be_bytes();
const NEW_MAGIC: &[u8; 8] = b"IHAVEOPT";
const REPLY_MAGIC: u64 = 0x3e889045565a9;

#[bitfield(u16)]
pub struct HandshakeFlags {
    /// Server supports the fixed newstyle protocol
    fixed_newstyle: bool,
    /// Servers supports omitting zeroes at the end of NBD_OPT_EXPORT_NAME
    no_zeroes: bool,

    #[bits(14)]
    _pad: u16,
}

#[bitfield(u32)]
pub struct ClientFlags {
    /// Client wants to use the fixed newstyle protocol
    fixed_newstyle: bool,
    // Client want the server to omit zeroes in NBD_OPT_EXPORT_NAME
    no_zeroes: bool,

    #[bits(30)]
    _pad: u32,
}

#[bitfield(u16)]
pub struct TransmissionFlags {
    has_flags: bool,
    read_only: bool,
    send_flush: bool,
    send_fua: bool,
    rotational: bool,
    send_trim: bool,
    send_write_zeroes: bool,
    send_df: bool,
    can_multi_conn: bool,
    send_resize: bool,
    send_cache: bool,
    send_fast_zero: bool,
    block_status_payload: bool,

    #[bits(3)]
    _pad: u8,
}

#[bitfield(u32)]
pub struct OldstyleFlags {
    #[bits(16)]
    transmission: TransmissionFlags,
    #[bits(16)]
    handshake: HandshakeFlags,
}

#[bitflag(u32)]
pub enum OptionName {
    #[base]
    ExportName = 1,
    Abort = 2,
    List = 3,
    Info = 6,
    Go = 7,
}

#[bitflag(u32)]
pub enum ReplyType {
    #[base]
    Ack = 1,
    Server = 2,
    Info = 3,
    MetaContext = 4,

    ErrUnsup = 0x8000_0001,
    ErrPolicy = 0x8000_0002,
    ErrInvalid = 0x8000_0003,
    ErrPlatform = 0x8000_0004,
    ErrTlsRequired = 0x8000_0005,
    ErrUnknown = 0x8000_0006,
    ErrShutdown = 0x8000_0007,
    ErrBlockSizeRequired = 0x8000_0008,
    ErrTooBig = 0x8000_0009,
    ErrExtHeaderRequired = 0x8000_000a,
}

#[derive(Debug)]
pub enum Reply {
    Ack,
    Server(String),
    Info,
    MetaContext,

    ErrUnsup(String),
    ErrPolicy(String),
    ErrInvalid(String),
    ErrPlatform(String),
    ErrTlsRequired(String),
    ErrUnknown(String),
    ErrShutdown(String),
    ErrBlockSizeRequired(String),
    ErrTooBig(String),
    ErrExtHeaderRequired(String),
}

impl Reply {
    pub async fn recv(stream: &mut TcpStream) -> Result<Reply> {
        ensure!(stream.read_u64().await? == REPLY_MAGIC);

        //let _option = OptionName::from_bits(stream.read_u32().await?);
        let option = stream.read_u32().await?;
        println!("Option {option}");
        let reply_type = ReplyType::from_bits(stream.read_u32().await?);
        
        let length = stream.read_u32().await?;
        let mut buffer = vec![0u8; length as usize];
        if length > 0 {
            stream.read_exact(buffer.as_mut_slice()).await?;
        }

        Ok(match reply_type {
            ReplyType::Ack => Reply::Ack,
            ReplyType::Server => Reply::Server(String::from_utf8(buffer)?),
            ReplyType::Info => todo!(),
            ReplyType::MetaContext => Reply::MetaContext, // TODO: this is not yet parsed
            ReplyType::ErrUnsup => Reply::ErrUnsup(String::from_utf8(buffer)?),
            ReplyType::ErrPolicy => Reply::ErrPolicy(String::from_utf8(buffer)?),
            ReplyType::ErrInvalid => Reply::ErrInvalid(String::from_utf8(buffer)?),
            ReplyType::ErrPlatform => Reply::ErrPlatform(String::from_utf8(buffer)?),
            ReplyType::ErrTlsRequired => Reply::ErrTlsRequired(String::from_utf8(buffer)?),
            ReplyType::ErrUnknown => Reply::ErrUnknown(String::from_utf8(buffer)?),
            ReplyType::ErrShutdown => Reply::ErrShutdown(String::from_utf8(buffer)?),
            ReplyType::ErrBlockSizeRequired => Reply::ErrBlockSizeRequired(String::from_utf8(buffer)?),
            ReplyType::ErrTooBig => Reply::ErrTooBig(String::from_utf8(buffer)?),
            ReplyType::ErrExtHeaderRequired => Reply::ErrExtHeaderRequired(String::from_utf8(buffer)?),
        })
    }

    pub fn is_error(&self) -> bool {
        match self {
            Reply::Ack => false,
            Reply::Server(_) => false,
            Reply::Info => false,
            Reply::MetaContext => false,
            Reply::ErrUnsup(_) => true,
            Reply::ErrPolicy(_) => true,
            Reply::ErrInvalid(_) => true,
            Reply::ErrPlatform(_) => true,
            Reply::ErrTlsRequired(_) => true,
            Reply::ErrUnknown(_) => true,
            Reply::ErrShutdown(_) => true,
            Reply::ErrBlockSizeRequired(_) => true,
            Reply::ErrTooBig(_) => true,
            Reply::ErrExtHeaderRequired(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    flags: ClientFlags,
}

impl Client {
    pub fn flags(&self) -> ClientFlags {
        self.flags
    }

    pub async fn list_exports(&self, stream: &mut TcpStream) -> Result<Vec<String>> {
        stream.write_all(NEW_MAGIC).await?;
        stream.write_u32(OptionName::List.into_bits()).await?;
        stream.write_u32(0).await?;

        let mut exports = Vec::new();
        loop {
            match Reply::recv(stream).await? {
                Reply::Ack => {
                    break
                    println!("Got ack'd. All fine.")
                },
                Reply::Server(name) => exports.push(name),
                rep if rep.is_error() => {
                    panic!("Failed export listing with: {rep:?}");
                },
                _ => {
                    panic!("Weird reply");
                },
            }
        }

        Ok(exports)
    }
}

#[derive(Debug)]
pub enum Negotiation {
    OldStyle(u64, OldstyleFlags),
    NewStyle(Client),
}

pub async fn negotiate(stream: &mut TcpStream, mut client_flags: ClientFlags) -> Result<Negotiation> {
    let mut buffer = [0u8; 8];

    // Read magic header
    stream.read_exact(&mut buffer).await?;
    ensure!(&buffer == INIT_PASSWD);

    // Read second magic header to determine negotiation style
    stream.read_exact(&mut buffer).await?;
    if &buffer == OLD_MAGIC {
        // Size of export in bytes
        let export_size = stream.read_u64().await?;
        // Flags
        let flags = stream.read_u32().await?;
        let flags = OldstyleFlags::from_bits(flags);

        // Done
        Ok(OldStyle(export_size, flags))
    } else if &buffer == NEW_MAGIC {
        // Receive handshake flags
        let handshake = stream.read_u16().await?;
        let handshake = HandshakeFlags::from_bits(handshake);

        // Only set client flags that are supported by server
        client_flags.set_fixed_newstyle(client_flags.fixed_newstyle() && handshake.fixed_newstyle());
        client_flags.set_no_zeroes(client_flags.no_zeroes() && handshake.no_zeroes());
        // Write client flags
        stream.write_u32(client_flags.into_bits()).await?;

        // Done
        Ok(NewStyle(Client { flags: client_flags }))
    } else {
        Err(anyhow!("Invalid second magic number, unknown negotation style"))
    }
}