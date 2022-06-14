use crate::typedef::ToAddr;
use std::net::UdpSocket;

pub fn send_to<Addr: ToAddr>(udp: &UdpSocket, msg: &[u8], addr: Addr) {
  err::log(udp.send_to(msg, addr))
}
