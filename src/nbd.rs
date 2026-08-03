use std::os::fd::{AsRawFd, IntoRawFd, RawFd};

use anyhow::{Result, anyhow, ensure};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpStream};

pub mod protocol;
use protocol::*;

#[derive(Debug)]
pub enum Reply {
    Ack,
    Server(String),
    Info,

    Err(ReplyType, String),
    Skipped,
}

impl Reply {
    pub async fn recv(stream: &mut TcpStream) -> Result<Reply> {
        ensure!(stream.read_u64().await? == REPLY_MAGIC);

        let _option = stream.read_u32().await?;
        let reply_type = stream.read_u32().await?;
        
        let length = stream.read_u32().await?;
        let mut buffer = vec![0u8; length as usize];
        if length > 0 {
            stream.read_exact(buffer.as_mut_slice()).await?;
        }

        use ReplyType::*;
        let reply = if reply_type == Ack.into_bits() {
            Reply::Ack
        } else if reply_type == Server.into_bits() {
            Reply::Server(String::from_utf8(buffer)?)
        } else if reply_type == Info.into_bits() {
            todo!()
        } else if reply_type >= ErrUnsup.into_bits() && reply_type <= ErrExtHeaderRequired.into_bits() {
            Reply::Err(ReplyType::from_bits(reply_type), String::from_utf8(buffer)?)
        } else {
            Reply::Skipped
        };

        Ok(reply)
    }
}

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    flags: ClientFlags,
}

impl Client {
    pub async fn export_name(mut self, name: &str) -> Result<(u64, TransmissionFlags, RawFd)> {
        self.stream.write_all(NEW_MAGIC).await?;
        self.stream.write_u32(OptionType::ExportName.into_bits()).await?;
        self.stream.write_u32(name.len().try_into()?).await?;
        self.stream.write_all(name.as_bytes()).await?;

        // Special case, here no reply header is sent, just this
        let size = self.stream.read_u64().await?;
        let flags = TransmissionFlags::from_bits(self.stream.read_u16().await?);
        if !self.flags.no_zeroes() {
            // Discard 124 bytes of zeroes
            let mut buf = [0u8; 124];
            self.stream.read_exact(&mut buf).await?;
        }

        let fd = self.stream.into_std()?.into_raw_fd();
        Ok((size, flags, fd))
    }

    pub async fn abort(mut self) -> Result<()> {
        self.stream.write_all(NEW_MAGIC).await?;
        self.stream.write_u32(OptionType::Abort.into_bits()).await?;
        self.stream.write_u32(0).await?;

        // The reply is actually not guaranteed, so we don't care
        Ok(())
    }

    pub async fn list_exports(&mut self) -> Result<Vec<String>> {
        self.stream.write_all(NEW_MAGIC).await?;
        self.stream.write_u32(OptionType::List.into_bits()).await?;
        self.stream.write_u32(0).await?;

        let mut exports = Vec::new();
        loop {
            match Reply::recv(&mut self.stream).await? {
                Reply::Ack => break,
                Reply::Server(name) => exports.push(name),
                Reply::Err(reply_type, msg) => {
                    return Err(anyhow!("Failed export listing with: {reply_type:?}: {msg}"));
                },
                _ => {
                    panic!("Unexpected reply");
                },
            }
        }

        Ok(exports)
    }
}

pub async fn negotiate(mut stream: TcpStream) -> Result<Client> {
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
        Ok(Client { stream, flags })
    } else if &buffer == OLD_MAGIC {
        Err(anyhow!("We do not support the oldstyle negotiation."))
    } else {
        Err(anyhow!("Invalid second magic number, unknown negotiation style"))
    }
}