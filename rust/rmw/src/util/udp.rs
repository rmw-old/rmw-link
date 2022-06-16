use std::net::{ToSocketAddrs, UdpSocket};

pub fn send_to<Addr: ToSocketAddrs>(udp: &UdpSocket, msg: &[u8], addr: Addr) {
  err::log(udp.send_to(msg, addr))
}

#[derive(Debug)]
struct Repy<Addr: ToSocketAddrs> {
  src: Addr,
  udp: UdpSocket,
}
