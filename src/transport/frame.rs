use crate::transport::pool::Kind;
use crate::transport::buffer::Chunk;
use std::ops::{Deref, DerefMut};
use bytes::BufMut;

const LEN_BYTES: usize = 2;
const KIND_BYTES: usize = 1;
const HEADER_BYTES: usize = LEN_BYTES + KIND_BYTES;

#[derive(Debug, Default)]
pub struct Frame {
    data: Vec<u8>,
}

impl Frame {
    pub fn new<T: Deref<Target=[u8]>>(kind: u8, body: T) -> Self {
        let size = Frame::header_len() + KIND_BYTES + body.len();
        let mut data = Vec::with_capacity(size);
        data.put_uint(body.len() as u64, Frame::header_len());
        data.put_uint(kind as u64, KIND_BYTES);
        data.put_slice(&body);
        Frame {
            data
        }
    }

    pub fn get_data(mut self) -> Vec<u8> {
        self.data.split_off(Frame::header_len() + KIND_BYTES)
    }

    pub(crate) fn to_vec(self) -> Vec<u8> {
        self.data
    }
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
