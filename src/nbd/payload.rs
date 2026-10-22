use anyhow::{Result, ensure};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::nbd::definitions::{
    InfoType, NEW_MAGIC, OptionType, REPLY_MAGIC, ReplyType, TransmissionFlags,
};

#[derive(Debug)]
pub struct Request<T: Payload> {
    kind: OptionType,
    payload: T,
}

pub trait Payload {
    fn size(&self) -> usize;
    async fn write(&self, stream: &mut TcpStream) -> Result<()>;
}

#[derive(Debug)]
pub struct InfoPayload {
    export_name: String,
    req_info: Vec<InfoType>,
}

#[derive(Debug)]
pub enum Reply {
    Ack,
    Server(String),
    Info(InfoReply),

    Err(ReplyType, String),
    Skipped,
}

#[derive(Debug)]
pub enum InfoReply {
    /// The export size and transmission flags used for the kernel
    Export(u64, TransmissionFlags),
    /// The name of the export
    Name(String),
    /// Textual description of the export
    Description(String),
    /// The server's advertised block sizes. Minimum, preferred, maximum.
    BlockSize(u32, u32, u32),
    /// Unknown info type
    Unknown,
}

impl<T: Payload> Request<T> {
    pub fn new(kind: OptionType, payload: T) -> Self {
        Request { kind, payload }
    }

    pub async fn write(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write_all(NEW_MAGIC).await?;
        stream.write_u32(self.kind.into_bits()).await?;
        stream.write_u32(self.payload.size().try_into()?).await?;
        self.payload.write(stream).await?;

        Ok(())
    }
}

async fn parse_info(mut buffer: &[u8]) -> Result<InfoReply> {
    let info_type = buffer.read_u16().await?;
    let reply = if info_type == InfoType::Export.into_bits() {
        let export_size = buffer.read_u64().await?;
        let flags = buffer.read_u16().await?;
        InfoReply::Export(export_size, TransmissionFlags::from_bits(flags))
    } else if info_type == InfoType::Name.into_bits() {
        InfoReply::Name(String::from_utf8(buffer.to_vec())?)
    } else if info_type == InfoType::Description.into_bits() {
        InfoReply::Description(String::from_utf8(buffer.to_vec())?)
    } else if info_type == InfoType::BlockSize.into_bits() {
        let min = buffer.read_u32().await?;
        let pref = buffer.read_u32().await?;
        let max = buffer.read_u32().await?;
        InfoReply::BlockSize(min, pref, max)
    } else {
        InfoReply::Unknown
    };

    Ok(reply)
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
            Reply::Info(parse_info(&buffer).await?)
        } else if reply_type >= ErrUnsup.into_bits()
            && reply_type <= ErrExtHeaderRequired.into_bits()
        {
            Reply::Err(ReplyType::from_bits(reply_type), String::from_utf8(buffer)?)
        } else {
            Reply::Skipped
        };

        Ok(reply)
    }
}

impl InfoPayload {
    pub fn new(export_name: &str, req_info: &[InfoType]) -> Self {
        InfoPayload {
            export_name: export_name.into(),
            req_info: req_info.into(),
        }
    }
}

impl Payload for () {
    fn size(&self) -> usize {
        0
    }

    async fn write(&self, _: &mut TcpStream) -> Result<()> {
        Ok(())
    }
}

impl Payload for InfoPayload {
    fn size(&self) -> usize {
        // data payload contains
        // - 32 bit, length of export name
        // - String
        // - 16 bit, number of info requests
        // - 16 bit * n, info requests
        4 + self.export_name.len() + 2 + size_of::<InfoType>() * self.req_info.len()
    }

    async fn write(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write_u32(self.export_name.len().try_into()?).await?;
        stream.write_all(self.export_name.as_bytes()).await?;
        stream.write_u16(self.req_info.len().try_into()?).await?;
        for req in self.req_info.iter() {
            stream.write_u16(req.into_bits()).await?;
        }

        Ok(())
    }
}
