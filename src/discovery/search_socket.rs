use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::UdpSocket;

pub struct SearchSocket {
    socket: UdpSocket,
    addr: SocketAddrV4,
    multi_addr: SocketAddrV4,
    port: u16,
}

impl SearchSocket {
    pub async fn new(addr: Ipv4Addr, multi_addr: Ipv4Addr, port: u16) -> std::io::Result<Self> {
        let addr = SocketAddrV4::new(addr, port);
        let multi_addr = SocketAddrV4::new(multi_addr, port);
        let socket = Self::get_socket(&addr, &multi_addr).await?;

        Ok(SearchSocket {
            socket,
            addr,
            multi_addr,
            port,
        })
    }

    pub async fn send(&self, data: Vec<u8>) -> std::io::Result<()> {
        self.socket.send_to(&data, self.multi_addr).await?;
        Ok(())
    }

    pub async fn read(&self) -> std::io::Result<(Vec<u8>, SocketAddr)> {
        let mut buffer = vec![0; 5];
        self.socket.readable().await.unwrap();
        let (_, addr) = self.socket.try_recv_from(&mut buffer)?;
        Ok((buffer, addr))
    }

    async fn get_socket(
        addr: &SocketAddrV4,
        multi_addr: &SocketAddrV4,
    ) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind(addr).await?;

        socket.set_multicast_loop_v4(true)?;
        socket.join_multicast_v4(*multi_addr.ip(), *addr.ip())?;

        Ok(socket)
    }
}
