use crate::req::Req;
use async_std::channel::Receiver;
use std::net::UdpSocket;

pub struct Recv {
  pub udp: UdpSocket,
  pub recv: Receiver<Req>,
}

impl Recv {
  pub async fn recv(&self) {
    while let Ok(_req) = self.recv.recv().await {}
  }
}
