use bytes::{BytesMut, Buf};
use std::ops::{Deref, DerefMut};

pub trait Chunk: DerefMut<Target=Vec<u8>> {
    fn header_len() -> usize;
    fn with_capacity_filled(capacity: usize) -> Self;
}

pub struct ConcatBuffer<T: Chunk> {
    inner: BytesMut,
    partial_chunk: Option<(usize, T)>,
}

impl<T: Chunk> ConcatBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        ConcatBuffer {
            inner: BytesMut::with_capacity(capacity),
            partial_chunk: None,
        }
    }

    pub fn try_read_chunk(&mut self) -> Option<T> {
        match self.partial_chunk.take() {
            Some((body_len, chunk)) =>
                self.try_read_partial_chunk(body_len, chunk),

            None =>
                self.try_read_full_chunk(),
        }
    }

    fn try_read_partial_chunk(&mut self, body_len: usize, mut chunk: T) -> Option<T> {
        if chunk.len() <= body_len + self.inner.len() {
            self.inner.copy_to_slice(&mut chunk[body_len..]);
            Some(chunk)
        } else {
            self.partial_chunk = Some((body_len, chunk));
            None
        }
    }

    fn try_read_full_chunk(&mut self) -> Option<T> {
        let body_len = self.try_read_header()?;
        let mut chunk = T::with_capacity_filled(body_len);

        if body_len <= self.inner.len() {
            self.inner.copy_to_slice(&mut chunk);
            Some(chunk)
        } else {
            let partial_body_len = self.inner.len();

            self.inner.copy_to_slice(&mut chunk[..partial_body_len]);
            self.fragment();

            self.partial_chunk = Some((partial_body_len, chunk));
            None
        }
    }

    fn try_read_header(&mut self) -> Option<usize> {
        if self.inner.len() >= T::header_len() {
            Some(self.inner.get_uint(T::header_len()) as usize)
        } else {
            self.fragment();
            None
        }
    }

    fn fragment(&mut self) {
        // This action will move data to the start of the buffer.
        // If there is no data, it will also move the cursor to the start.
        // Read .reserve() documentation for more details
        self.inner.reserve(self.inner.capacity() - self.inner.len() + 1);
    }
}

impl<T: Chunk> Default for ConcatBuffer<T> {
    fn default() -> Self {
        ConcatBuffer {
            inner: BytesMut::with_capacity(256usize.pow(T::header_len() as u32) - 1),
            partial_chunk: None,
        }
    }
}

impl<T: Chunk> Deref for ConcatBuffer<T> {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Chunk> DerefMut for ConcatBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
