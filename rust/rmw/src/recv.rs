use crate::{req::Req, typedef::ToAddr};
use async_std::channel::Receiver;
use std::net::UdpSocket;

pub struct Recv<Addr: ToAddr> {
  pub udp: UdpSocket,
  pub recv: Receiver<Req<Addr>>,
}

impl<Addr: ToAddr> Recv<Addr> {
  pub async fn recv(&self) {
    while let Ok(_req) = self.recv.recv().await {}
  }
}
