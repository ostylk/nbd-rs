use std::os::fd::{IntoRawFd, RawFd};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::nbd::{
    definitions::{
        ClientFlags, ClientFlagsBuilder, HandshakeFlags, INIT_PASSWD, InfoType, NEW_MAGIC,
        OLD_MAGIC, OptionType, TransmissionFlags,
    },
    payload::{InfoPayload, InfoReply, Reply, Request},
};
use anyhow::{Context, Result, anyhow, ensure};

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    flags: ClientFlags,
}

#[derive(Debug, Default)]
pub struct ExportInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub size: Option<u64>,
    pub min_bs: Option<u32>,
    pub pref_bs: Option<u32>,
    pub max_bs: Option<u32>,
    pub flags: Option<TransmissionFlags>,
}

impl Client {
    pub async fn negotiate(mut stream: TcpStream) -> Result<Self> {
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
            Err(anyhow!(
                "Invalid second magic number, unknown negotiation style"
            ))
        }
    }

    pub fn into_fd(self) -> Result<RawFd> {
        Ok(self.stream.into_std()?.into_raw_fd())
    }

    #[allow(unused)]
    pub async fn export_name(mut self, name: &str) -> Result<(u64, TransmissionFlags, RawFd)> {
        self.stream.write_all(NEW_MAGIC).await?;
        self.stream
            .write_u32(OptionType::ExportName.into_bits())
            .await?;
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

        Ok((size, flags, self.into_fd()?))
    }

    pub async fn go(&mut self, export_name: &str) -> Result<(u64, TransmissionFlags)> {
        let payload = InfoPayload::new(export_name, &[InfoType::Export]);

        let req = Request::new(OptionType::Go, payload);
        req.write(&mut self.stream).await?;

        let mut size = None;
        let mut flags = None;
        loop {
            match Reply::recv(&mut self.stream).await? {
                Reply::Ack => break,
                Reply::Info(reply) => {
                    match reply {
                        InfoReply::Export(e_size, e_flags) => {
                            size = Some(e_size);
                            flags = Some(e_flags);
                        }
                        _ => { /* ignore */ }
                    }
                }
                Reply::Err(reply_type, msg) => {
                    return Err(anyhow!("nbd server error: {reply_type:?}: {msg}"));
                }
                _ => {
                    panic!("Unexpected reply")
                }
            }
        }

        let size = size.context("nbd server did not send export size!")?;
        let flags = flags.context("nbd server did not send transmission flags")?;

        Ok((size, flags))
    }

    pub async fn list_exports(&mut self) -> Result<Vec<String>> {
        let req = Request::new(OptionType::List, ());
        req.write(&mut self.stream).await?;

        let mut exports = Vec::new();
        loop {
            match Reply::recv(&mut self.stream).await? {
                Reply::Ack => break,
                Reply::Server(name) => exports.push(name),
                Reply::Err(reply_type, msg) => {
                    return Err(anyhow!("nbd server error: {reply_type:?}: {msg}"));
                }
                _ => {
                    panic!("Unexpected reply");
                }
            }
        }

        Ok(exports)
    }

    pub async fn info_export(&mut self, export_name: &str) -> Result<ExportInfo> {
        let payload = InfoPayload::new(
            export_name,
            &[
                InfoType::Export,
                InfoType::Name,
                InfoType::Description,
                InfoType::BlockSize,
            ],
        );

        let req = Request::new(OptionType::Info, payload);
        req.write(&mut self.stream).await?;

        let mut info = ExportInfo::default();
        loop {
            match Reply::recv(&mut self.stream).await? {
                Reply::Ack => break,
                Reply::Info(reply) => {
                    match reply {
                        InfoReply::Export(size, flags) => {
                            info.size = Some(size);
                            info.flags = Some(flags);
                        }
                        InfoReply::Name(name) => {
                            info.name = Some(name);
                        }
                        InfoReply::Description(desc) => {
                            info.description = Some(desc);
                        }
                        InfoReply::BlockSize(min, pref, max) => {
                            info.min_bs = Some(min);
                            info.pref_bs = Some(pref);
                            info.max_bs = Some(max);
                        }
                        InfoReply::Unknown => { /* ignore */ }
                    }
                }
                Reply::Err(reply_type, msg) => {
                    return Err(anyhow!("nbd server error: {reply_type:?}: {msg}"));
                }
                _ => {
                    panic!("Unexpected reply")
                }
            }
        }
        Ok(info)
    }
}
