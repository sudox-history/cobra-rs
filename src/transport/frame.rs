use bytes::{Buf, BufMut};

use crate::transport::pool::Kind;
use crate::transport::constants::{
    FRAME_LEN_BYTES, FRAME_KIND_BYTES, FRAME_HEADER_BYTES
};

#[derive(Debug, Default)]
pub struct Frame {
    kind: u8,
    pub data: Vec<u8>,
}

impl Frame {
    pub fn from_vec(kind: u8, data: Vec<u8>) -> Self {
        Frame {
            kind,
            data
        }
    }

    pub fn from_slice(kind: u8, data: &[u8]) -> Self {
        Frame {
            kind,
            data: data.to_vec(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.len());

        // Frame header
        vec.put_uint(self.len() as u64, FRAME_LEN_BYTES);
        vec.put_uint(self.kind  as u64, FRAME_KIND_BYTES);
        
        // Frame data
        vec.copy_from_slice(&self.data);
        
        vec
    }

    fn len(&self) -> usize {
        FRAME_LEN_BYTES + FRAME_KIND_BYTES + self.data.len()
    }
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.kind
    }
}

//...[start_pointer...end_pointer)...
// frame::Buff

