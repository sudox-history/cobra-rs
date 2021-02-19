use std::ops::{Deref, DerefMut};

use bytes::{BufMut, BytesMut};

use crate::mem::Chunk;
use crate::sync::Kind;

const HEADER_LEN_BYTES: usize = 2;
const HEADER_KIND_BYTES: usize = 1;
const HEADER_BYTES: usize = HEADER_LEN_BYTES + HEADER_KIND_BYTES;

/// Simple stream-based protocol communication unit
///
/// Implements [`Chunk`] and [`Kind`] trates
///
/// [`Chunk`]: crate::mem::Chunk
/// [`Kind`]: crate::sync::kind
pub struct Frame {
    inner: BytesMut,
}

impl Frame {
    /// Creates new frame
    ///
    /// # Note
    ///
    /// This operation is O (n) due to copying
    pub fn create(kind: u8, body: &[u8]) -> Self {
        let total_len = HEADER_BYTES + body.len();

        let mut frame = Frame { inner: BytesMut::with_capacity(total_len) };

        frame.put_header(kind);
        frame.put_body(body);

        frame
    }

    fn put_header(&mut self, kind: u8) {
        self.inner.put_uint((self.inner.capacity() - HEADER_LEN_BYTES) as u64, HEADER_LEN_BYTES);
        self.inner.put_uint(kind as u64, HEADER_KIND_BYTES);
    }

    fn put_body(&mut self, body: &[u8]) {
        self.inner.put_slice(body)
    }

    /// Returns body of frame
    ///
    /// # Note
    ///
    /// This operation is O (1) because only some of the internal
    /// indexes are updated
    pub fn get_body(mut self) -> BytesMut {
        self.inner.split_off(HEADER_BYTES)
    }
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.inner[HEADER_LEN_BYTES]
    }
}

impl Chunk for Frame {
    fn header_len() -> usize {
        HEADER_LEN_BYTES
    }

    fn with_capacity(capacity: usize) -> Self {
        Frame { inner: BytesMut::with_capacity(capacity) }
    }
}

impl Deref for Frame {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
