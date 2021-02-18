use std::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut, BytesMut};

/// Unbreakable piece of memory
pub trait Chunk: DerefMut<Target=BytesMut> {
    /// Returns number of bytes that must be reserved for data length
    ///
    /// # Implementation note
    ///
    /// You can store only 256^n inside a chunk, where n
    /// is the number of bytes returned by this function
    ///
    /// See [`max_body_len`] for more information
    ///
    /// [`max_body_len`]: crate::mem::Chunk::max_body_len
    fn header_len() -> usize;

    /// Returns the chunk with the requested allocated capacity
    ///
    /// # Implementation note
    ///
    /// You **don't have to** fill in the chunk
    fn with_capacity(capacity: usize) -> Self;

    /// Returns maximum data length can be stored inside chunk
    fn max_body_len() -> usize {
        256_usize.pow(Self::header_len() as u32)
    }
}

/// A buffer for restoring memory chunks from an undefined byte stream
///
/// [`ConcatBuf`] implements [`DerefMut`] to [`BytesMut`]
///
/// [`ConcatBuf`]: crate::sync::ConcatBuf
/// [`BytesMut`]: bytes::BytesMut
/// [`DerefMut`]: std::ops::DerefMut
pub struct ConcatBuf<T: Chunk> {
    inner: BytesMut,
    partial_chunk: Option<(usize, T)>,
}

impl<T: Chunk> ConcatBuf<T> {
    /// Creates new buffer with specified capacity
    ///
    /// # Note
    ///
    /// Panics if there is not enough capacity to store one chunk
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity < T::header_len() + T::max_body_len() {
            panic!("attempt to allocate buffer with insufficient memory")
        }

        ConcatBuf {
            inner: BytesMut::with_capacity(capacity),
            partial_chunk: None,
        }
    }

    fn create_chunk(body_len: usize) -> T {
        let capacity = T::header_len() + body_len;
        let mut chunk = T::with_capacity(capacity);

        // Copying header to resulting chunk
        chunk.put_uint(body_len as u64, T::header_len());

        unsafe {
            // SAFETY: We don't use uninitialized data
            chunk.set_len(capacity);
        }

        chunk
    }

    /// Tries to read chunk
    ///
    /// # Note
    ///
    /// You should call this function until it returns [`None`]
    ///
    /// [`None`]: std::option::Option::None
    pub fn try_read_chunk(&mut self) -> Option<T> {
        match self.partial_chunk.take() {
            Some((current_len, chunk)) =>
                self.try_read_partial_chunk(current_len, chunk),

            None =>
                self.try_read_full_chunk(),
        }
    }

    fn try_read_partial_chunk(&mut self, current_len: usize, mut chunk: T) -> Option<T> {
        if chunk.len() <= current_len + self.inner.len() {
            self.inner.copy_to_slice(&mut chunk[current_len..]);
            Some(chunk)
        } else {
            self.partial_chunk = Some((current_len, chunk));
            None
        }
    }

    fn try_read_full_chunk(&mut self) -> Option<T> {
        let body_len = self.try_read_header()?;
        let mut chunk: T = ConcatBuf::create_chunk(body_len);

        if body_len <= self.inner.len() {
            self.inner.copy_to_slice(&mut chunk[T::header_len()..]);
            Some(chunk)
        } else {
            let current_len = self.inner.len() + T::header_len();

            self.inner.copy_to_slice(&mut chunk[T::header_len()..current_len]);
            self.fragment();

            self.partial_chunk = Some((current_len, chunk));
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
        // This action will move (using memmove) data to the start of the buffer.
        // If there is no data, it will also move the cursor to the start.
        // Read .reserve() documentation for more details
        self.inner.reserve(self.inner.capacity() - self.inner.len() + 1);
    }
}

impl<T: Chunk> Default for ConcatBuf<T> {
    fn default() -> Self {
        ConcatBuf {
            inner: BytesMut::with_capacity(
                (T::header_len() + 256_usize.pow(T::header_len() as u32) - 1) * 2
            ),
            partial_chunk: None,
        }
    }
}

impl<T: Chunk> Deref for ConcatBuf<T> {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Chunk> DerefMut for ConcatBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
