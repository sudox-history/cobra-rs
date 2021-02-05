use cobra_rs::transport::buffer::{ConcatBuffer, Chunk};
use bytes::{Buf, BufMut, BytesMut};
use tokio::net::{TcpStream, TcpListener};
use std::io::Read;
use std::ops::{DerefMut, Deref};


#[tokio::main]
async fn main() {
    let mut bytes = BytesMut::with_capacity(4);
    bytes.put_u8(123);
    bytes.put_u8(123);
    bytes.put_u8(123);
    bytes.get_u8();

    // 00 00 00 00 | 00 00 00 12 | 00 00 00
    //bytes.copy_within(4..8, 0);

    // ATTENTION!!!: Moving bytes to start (capacity-len, capacity]
    bytes.reserve(bytes.capacity() - bytes.len() + 1);
    println!("{:?}", bytes.capacity());
}