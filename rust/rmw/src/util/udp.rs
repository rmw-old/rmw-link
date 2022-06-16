use std::net::{ToSocketAddrs, UdpSocket};

pub fn send_to<Addr: ToSocketAddrs>(udp: &UdpSocket, msg: &[u8], addr: Addr) {
  err::log(udp.send_to(msg, addr))
}

#[derive(Debug)]
pub struct Input<'a, Addr: ToSocketAddrs> {
  pub addr: Addr,
  pub udp: &'a UdpSocket,
  pub msg: &'a [u8],
}

impl<Addr: ToSocketAddrs> Input<'_, Addr> {
  pub fn reply(&self, msg: &[u8]) {
    send_to(self.udp, msg, &self.addr)
  }
}
