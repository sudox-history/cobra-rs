use std::ops::{Deref, DerefMut};

use bytes::{BufMut, BytesMut};

use crate::mem::Chunk;
use crate::sync::Kind;

const LEN_BYTES: usize = 2;
const KIND_BYTES: usize = 1;

#[derive(Debug, Default)]
pub struct Frame {
    data: BytesMut,
}

impl Frame {
    pub fn new<T: Deref<Target=[u8]>>(kind: u8, body: T) -> Self {
        let size = Frame::header_len() + KIND_BYTES + body.len();
        let mut data = BytesMut::with_capacity(size);
        data.put_uint((body.len() + KIND_BYTES) as u64, LEN_BYTES);
        data.put_uint(kind as u64, KIND_BYTES);
        data.put_slice(&body);
        Frame {
            data
        }
    }

    pub fn get_data(self) -> Vec<u8> {

        // TODO maybe excess move or copy
        self.data[KIND_BYTES..].to_vec()
    }
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.data[0]
    }
}

impl Chunk for Frame {
    fn header_len() -> usize {
        LEN_BYTES
    }

    fn with_capacity(capacity: usize) -> Self {
        Frame {
            data: BytesMut::with_capacity(capacity),
        }
    }
}

impl Deref for Frame {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
