use bitfields::{bitfield, bitflag};

pub const INIT_PASSWD: &[u8; 8] = b"NBDMAGIC";
pub const OLD_MAGIC: &[u8; 8] = &0x00420281861253_u64.to_be_bytes();
pub const NEW_MAGIC: &[u8; 8] = b"IHAVEOPT";
pub const REPLY_MAGIC: u64 = 0x3e889045565a9;

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

#[bitflag(u32)]
#[derive(Debug)]
pub enum OptionType {
    #[base]
    ExportName = 1,
    Abort = 2,
    List = 3,
    PeekExport = 4,
    Starttls = 5,
    Info = 6,
    Go = 7,
    StructuredReply = 8,
    ListMetaContext = 9,
    SetMetaContext = 10,
    ExtendedHeaders = 11,
}

#[bitflag(u32)]
#[derive(Debug)]
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

#[bitflag(u16)]
#[derive(Debug)]
pub enum InfoType {
    #[base]
    Export = 0,
    Name = 1,
    Description = 2,
    BlockSize = 3,
}