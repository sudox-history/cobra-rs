use bytes::{BytesMut, Buf};
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;

pub trait Chunk: DerefMut<Target=Vec<u8>> + Debug {
    fn header_len() -> usize;
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

    // fn try_read_partial_chunk(&mut self) -> Option<T> {
    //     //Note: this function must only be called if there is a chunk
    //     let (len, mut partial_chunk) = self.partial_chunk
    //         .take()
    //         .expect("attempt to read a non-existent chunk");
    //
    //     if len <= partial_chunk.len() + self.inner.len() {
    //         partial_chunk.extend_from_slice(&self.inner[..len - partial_chunk.len()]);
    //         Some(partial_chunk)
    //     } else {
    //         self.partial_chunk = Some((len, partial_chunk));
    //         None
    //     }
    // }

    // pub fn try_read_chunk(&mut self) -> Option<T> {
    //     match self.partial_chunk {
    //         Some((len, mut partial_chunk)) => {
    //             if len <= partial_chunk.len() + self.inner.len() {
    //                 partial_chunk.extend_from_slice(&self.inner[..len - partial_chunk.len()]);
    //                 Some(partial_chunk)
    //             } else {
    //                 self.partial_chunk = Some((len, partial_chunk));
    //                 None
    //             }
    //         },
    //
    //         None => {
    //             let len = self.try_read_header()?;
    //             if len <= self.inner.len() as u64 {
    //                 Some(T::)
    //             } else {
    //                 let start = self.read_pointer;
    //                 let end = self.write_pointer;
    //                 let slice = &self.inner[start..end];
    //                 self.read_pointer = 0;
    //                 self.write_pointer = 0;
    //                 self.part_frame = Some((len, Frame::from_slice(kind, slice)));
    //                 None
    //             }
    //         }
    //     }
    // }

    fn try_read_header(&mut self) -> Option<u64> {
        if self.inner.len() >= T::header_len() {
            Some(self.inner.get_uint(T::header_len()))
        } else {
            self.fragment();
            None
        }
    }

    fn fragment(&mut self) {
        // This action will move data to the start of the buffer
        // or cleans it if there is no data
        // Read .reserve() documentation for more details
        self.inner.reserve(self.inner.capacity());
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