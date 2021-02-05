use cobra_rs::transport::buffer::{ConcatBuffer, Chunk};
use bytes::{Buf, BufMut, BytesMut};
use tokio::net::{TcpStream, TcpListener};
use std::io::Read;
use std::ops::{DerefMut, Deref};


#[tokio::main]
async fn main() {
    let mut bytes = BytesMut::with_capacity(11);
    bytes.put_i32(10);
    bytes.put_i32(12);
    bytes.get_i32();
    bytes.get_i32();

    // 00 00 00 00 | 00 00 00 12 | 00 00 00
    //bytes.copy_within(4..8, 0);

    // ATTENTION!!!: Moving bytes to start (capacity-len, capacity]
    bytes.reserve(bytes.capacity());
    println!("{:?}", bytes.capacity());
}