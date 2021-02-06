use cobra_rs::transport::buffer::*;
use std::ops::{Deref, DerefMut};
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
struct TestChunk {
    inner: Vec<u8>,
}

impl Chunk for TestChunk {
    fn header_len() -> usize {
        2
    }

    fn with_capacity_filled(capacity: usize) -> Self {
        TestChunk {
            inner: vec![0; capacity],
        }
    }
}

impl Deref for TestChunk {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestChunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// [0 1](255)[0 2](255 255)[0 3](255 255 255)
#[tokio::test]
async fn example() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_uint(1, TestChunk::header_len());
    buffer.put_u8(10);

    buffer.put_uint(2, TestChunk::header_len());
    buffer.put_u8(11);
    buffer.put_u8(12);

    buffer.put_uint(3, TestChunk::header_len());
    buffer.put_u8(13);
    buffer.put_u8(14);
    buffer.put_u8(15);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![10]);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![11, 12]);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![13, 14, 15]);
}

// [0 1](1)
#[tokio::test]
async fn partial_header_input() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_u8(0);
    buffer.put_u8(1);

    buffer.put_u8(1);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![1]);
}

// [1 1](0..0)
#[tokio::test]
async fn big_partial_header_input() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_u8(1);
    buffer.put_u8(1);

    for _ in 0..257 {
        buffer.put_u8(0);
    }

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![0; 257]);
}

// [0 10](0 1 2 3 4 5 6 7 8 9)
#[tokio::test]
async fn none_response() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_u8(0);
    assert!(buffer.try_read_chunk().is_none());
    buffer.put_u8(10);
    assert!(buffer.try_read_chunk().is_none());

    for i in 0..9 {
        buffer.put_u8(i);
        assert!(buffer.try_read_chunk().is_none());
    }
    buffer.put_u8(9);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

// [0 2](1 2)[0 2](3 4)
#[tokio::test]
async fn partial_body_read() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(1);
    buffer.put_u8(2);

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(3);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![1, 2]);
    assert!(buffer.try_read_chunk().is_none());

    buffer.put_u8(4);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![3, 4]);
}

// [0 2](1 2)[0 2](3 4)
#[tokio::test]
async fn partial_head_read() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_int(2, TestChunk::header_len());

    buffer.put_u8(1);
    buffer.put_u8(2);

    buffer.put_u8(0);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![1, 2]);
    assert!(buffer.try_read_chunk().is_none());

    buffer.put_u8(2);

    buffer.put_u8(3);
    buffer.put_u8(4);

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![3, 4]);
}

// [0 1](0)[0 2](0 1)..[0 255](0..255)
#[tokio::test]
async fn stress_test() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    for capacity in 0..255 {
        buffer.put_int(capacity, TestChunk::header_len());
        for i in 0..capacity {
            buffer.put_u8(i as u8);
        }
    }

    for capacity in 0..255 {
        let v: Vec<u8> = (0u8..capacity).collect();
        assert_eq!(buffer.try_read_chunk()
                       .unwrap()
                       .inner, v);
    }
}

// [255 255](0..0)[255 255)[1..1]
#[tokio::test]
async fn huge_chunks() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_int(65535, TestChunk::header_len());
    for _ in 0..65535 {
        buffer.put_u8(0);
    }

    buffer.put_int(65535, TestChunk::header_len());
    for _ in 0..65535 {
        buffer.put_u8(1);
    }

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![0; 65535]);
    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![1; 65535]);
}

// [0 0]()[0 0]()[0 0]()
#[tokio::test]
async fn zero_len_chunks() {
    let mut buffer: ConcatBuffer<TestChunk> = ConcatBuffer::default();

    buffer.put_int(0, TestChunk::header_len());
    buffer.put_int(0, TestChunk::header_len());
    buffer.put_int(0, TestChunk::header_len());

    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![]);
    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![]);
    assert_eq!(buffer.try_read_chunk()
                   .unwrap()
                   .inner, vec![]);
}
