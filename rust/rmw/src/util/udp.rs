use std::net::{ToSocketAddrs, UdpSocket};

pub fn send_to<Addr: ToSocketAddrs>(udp: &UdpSocket, msg: &[u8], addr: Addr) {
  err::log(udp.send_to(msg, addr))
}

#[derive(Debug)]
pub struct Recv<'a, Addr: ToSocketAddrs> {
  addr: Addr,
  udp: &'a UdpSocket,
}

impl<Addr: ToSocketAddrs> Recv<'_, Addr> {
  pub fn reply(&self, msg: &[u8]) {
    send_to(self.udp, msg, &self.addr)
  }
}
