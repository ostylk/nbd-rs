use anyhow::{Result, anyhow, ensure};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

mod protocol;
use protocol::*;

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
                Reply::Ack => break,
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

pub async fn negotiate(stream: &mut TcpStream) -> Result<Client> {
    let mut buffer = [0u8; 8];

    // Read magic header
    stream.read_exact(&mut buffer).await?;
    ensure!(&buffer == INIT_PASSWD);

    // Read second magic header to determine negotiation style
    stream.read_exact(&mut buffer).await?;
    if &buffer == NEW_MAGIC {
        // Receive handshake flags
        let handshake = stream.read_u16().await?;
        let handshake = HandshakeFlags::from_bits(handshake);
        // We only support the fixed newstyle negotiation
        ensure!(handshake.fixed_newstyle());

        // Only set client flags that are supported by server
        let flags = ClientFlagsBuilder::new()
            .with_fixed_newstyle(true)
            .with_no_zeroes(handshake.no_zeroes())
            .build();
        // Write client flags
        stream.write_u32(flags.into_bits()).await?;

        // Done
        Ok(Client { flags })
    } else if &buffer == OLD_MAGIC {
        Err(anyhow!("We do not support the oldstyle negotiation."))
    } else {
        Err(anyhow!("Invalid second magic number, unknown negotiation style"))
    }
}