pub const GENL_FAMILY_NAME: &str = "nbd";
pub const GENL_VERSION: u8 = 1;
pub const GENL_MCAST_GROUP_NAME: &str = "nbd_mc_group";

pub const ATTR_UNSPEC: u16 = 0;
pub const ATTR_INDEX: u16 = 1;
pub const ATTR_SIZE_BYTES: u16 = 2;
pub const ATTR_BLOCK_SIZE_BYTES: u16 = 3;
pub const ATTR_TIMEOUT: u16 = 4;
pub const ATTR_SERVER_FLAGS: u16 = 5;
pub const ATTR_CLIENT_FLAGS: u16 = 6;
pub const ATTR_SOCKETS: u16 = 7;
pub const ATTR_DEAD_CONN_TIMEOUT: u16 = 8;
pub const ATTR_DEVICE_LIST: u16 = 9;
pub const ATTR_BACKEND_IDENTIFIER: u16 = 10;

/*
 * This is the format for multiple devices with NBD_ATTR_DEVICE_LIST
 *
 * [NBD_ATTR_DEVICE_LIST]
 *   [NBD_DEVICE_ITEM]
 *     [NBD_DEVICE_INDEX]
 *     [NBD_DEVICE_CONNECTED]
 */
pub const DEVICE_ITEM_UNSPEC: u16 = 0;
pub const DEVICE_ITEM: u16 = 1;

pub const DEVICE_UNSPEC: u16 = 0;
pub const DEVICE_INDEX: u16 = 1;
pub const DEVICE_CONNECTED: u16 = 2;

/*
 * This is the format for multiple sockets with NBD_ATTR_SOCKETS
 *
 * [NBD_ATTR_SOCKETS]
 *   [NBD_SOCK_ITEM]
 *     [NBD_SOCK_FD]
 *   [NBD_SOCK_ITEM]
 *     [NBD_SOCK_FD]
 */
pub const SOCK_ITEM_UNSPEC: u16 = 0;
pub const SOCK_ITEM: u16 = 1;

pub const SOCK_UNSPEC: u16 = 0;
pub const SOCK_FD: u16 = 1;

pub const CMD_UNSPEC: u8 = 0;
pub const CMD_CONNECT: u8 = 1;
pub const CMD_DISCONNECT: u8 = 2;
pub const CMD_RECONFIGURE: u8 = 3;
pub const CMD_LINK_DEAD: u8 = 4;
pub const CMD_STATUS: u8 = 5;
