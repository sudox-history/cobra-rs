use crate::transport::pool::Kind;
use crate::transport::buffer::Chunk;
use std::ops::{Deref, DerefMut};

const LEN_BYTES: usize = 2;
const KIND_BYTES: usize = 1;
const HEADER_BYTES: usize = LEN_BYTES + KIND_BYTES;

#[derive(Debug, Default)]
pub struct Frame {
    data: Vec<u8>,
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.data[0]
    }
}

impl Chunk for Frame {
    fn header_len() -> usize {
        HEADER_BYTES
    }

    fn with_capacity_filled(capacity: usize) -> Self {
        Frame {
            data: vec![0; capacity],
        }
    }
}

impl Deref for Frame {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
