pub const COMPRESSED: u16 = 1 << 0;
pub const ENCRYPTED: u16 = 1 << 1;
pub const FRAGMENTED: u16 = 1 << 2;
pub const LAST_FRAG: u16 = 1 << 3;
pub const ACK_REQ: u16 = 1 << 4;
pub const KNOWN_MASK: u16 = COMPRESSED | ENCRYPTED | FRAGMENTED | LAST_FRAG | ACK_REQ;
