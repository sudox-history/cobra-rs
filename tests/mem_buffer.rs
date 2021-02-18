use std::ops::{Deref, DerefMut};

use bytes::{BufMut, BytesMut};

use cobra_rs::mem::*;

#[derive(Debug)]
struct TestChunk {
    inner: BytesMut,
}

impl TestChunk {
    fn as_bytes(&self) -> &[u8] {
        &self.inner[Self::header_len()..]
    }
}

impl Chunk for TestChunk {
    fn header_len() -> usize {
        2
    }

    fn with_capacity(capacity: usize) -> Self {
        TestChunk {
            inner: BytesMut::with_capacity(capacity),
        }
    }
}

impl Deref for TestChunk {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestChunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// [0 1](1)[0 2](1 2)[0 3](1 2 3)
#[tokio::test]
async fn simple_chunks() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_uint(1, TestChunk::header_len());
    buffer.put_u8(1);

    buffer.put_uint(2, TestChunk::header_len());
    buffer.put_u8(1);
    buffer.put_u8(2);

    buffer.put_uint(3, TestChunk::header_len());
    buffer.put_u8(1);
    buffer.put_u8(2);
    buffer.put_u8(3);

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1]);
    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1, 2]);
    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1, 2, 3]);
}

// [0 5](1 2 3 4 5)
#[tokio::test]
async fn partial_chunk() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_u8(0);
    assert!(buffer.try_read_chunk().is_none());

    buffer.put_u8(5);
    assert!(buffer.try_read_chunk().is_none());

    for i in 1..5 {
        buffer.put_u8(i);
        assert!(buffer.try_read_chunk().is_none());
    }

    buffer.put_u8(5);
    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1, 2, 3, 4, 5]);
}

// [0 2](1 2)[0 2](3 4)
#[tokio::test]
async fn next_chunk_partial_header() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(1);
    buffer.put_u8(2);

    buffer.put_u8(0);

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1, 2]);
    assert!(buffer.try_read_chunk().is_none());

    buffer.put_u8(2);

    buffer.put_u8(3);
    buffer.put_u8(4);

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![3, 4]);
}

// [0 2](1 2)[0 2](3 4)
#[tokio::test]
async fn next_chunk_partial_body() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(1);
    buffer.put_u8(2);

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(3);

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1, 2]);
    assert!(buffer.try_read_chunk().is_none());

    buffer.put_u8(4);

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![3, 4]);
}

// [255 255](0){65535}[255 255](1){65535}
#[tokio::test]
async fn max_len_chunks() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(65535, TestChunk::header_len());
    for _ in 0..65535 {
        buffer.put_u8(0);
    }

    buffer.put_int(65535, TestChunk::header_len());
    for _ in 0..65535 {
        buffer.put_u8(1);
    }

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![0; 65535]);
    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![1; 65535]);
}

// [0 0]()[0 0]()
#[tokio::test]
async fn zero_len_chunks() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(0, TestChunk::header_len());
    buffer.put_int(0, TestChunk::header_len());

    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![]);
    assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), vec![]);
}

// [0 1](1)
#[tokio::test]
async fn buffer_cleaning() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(1, TestChunk::header_len());
    buffer.put_u8(1);

    buffer.try_read_chunk();
    assert_eq!(buffer.len(), 0);

    let pointer = buffer.as_ptr();
    buffer.try_read_chunk();

    unsafe {
        assert_eq!(buffer.as_ptr(), pointer.sub(3));
    }
}

// [0 1](1)[0..
#[tokio::test]
async fn header_moving() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    buffer.put_int(1, TestChunk::header_len());
    buffer.put_u8(1);

    buffer.put_u8(1);

    buffer.try_read_chunk();
    assert_eq!(buffer.len(), 1);

    let pointer = buffer.as_ptr();
    buffer.try_read_chunk();

    unsafe {
        assert_eq!(buffer.as_ptr(), pointer.sub(3));
    }

    assert_eq!(buffer[0], 1);
}

// [0 1](0)[0 2](0 1)..[0 255](0..255)
#[tokio::test]
async fn stress_test() {
    let mut buffer: ConcatBuf<TestChunk> = ConcatBuf::default();

    for capacity in 0..255 {
        buffer.put_int(capacity, TestChunk::header_len());
        for i in 0..capacity {
            buffer.put_u8(i as u8);
        }
    }

    for capacity in 0..255 {
        let v: Vec<u8> = (0_u8..capacity).collect();
        assert_eq!(buffer.try_read_chunk().unwrap().as_bytes(), v);
    }
}