use std::usize;

pub const FRAME_LEN_BYTES: usize = 2;
pub const FRAME_KIND_BYTES: usize = 1;
pub const FRAME_HEADER_BYTES: usize = FRAME_LEN_BYTES + FRAME_KIND_BYTES;