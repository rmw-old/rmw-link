use crate::{cmd::Cmd, req::Req};
use addrbytes::ToBytes;
use async_std::{channel::Sender, task::spawn};
use expire_map::ExpireMap;
use std::net::UdpSocket;
pub trait ToAddr = 'static
  + ToBytes
  + std::net::ToSocketAddrs
  + std::hash::Hash
  + std::marker::Send
  + std::fmt::Debug
  + std::fmt::Display
  + std::marker::Sync
  + Ord
  + Copy
  + Clone;

pub struct Send<Addr: ToAddr> {
  pub send: Sender<Req>,
  pub udp: UdpSocket,
  pub map: ExpireMap<Addr, u8>,
}

impl<Addr: ToAddr> Send<Addr> {
  pub fn new(send: Sender<Req>, udp: UdpSocket, boot: Vec<Addr>) -> Self {
    let map = ExpireMap::new(config::get!(net / timeout / ping, 7u8), 60);
    let u = udp.try_clone().unwrap();
    let m = map.clone();
    spawn(async move {
      for addr in boot {
        dbg!(addr);
        m.add(addr);
        err::log(u.send_to(&[Cmd::Ping as u8], addr));
      }
    });
    Self { udp, send, map }
  }

  pub async fn send(&self, msg: &[u8], addr: Addr) {
    // &[Cmd::Ping as u8]
    // Cmd::from_u8

    let msg_len = msg.len();
    let udp = &self.udp;

    macro_rules! reply {
      ($bin:expr) => {{
        err::log(udp.send_to($bin, addr));
      }};
    }

    if msg_len == 0 {
      if self.map.has(&addr) {
        dbg!("exist", addr);
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      dbg!(&cmd, &msg[1..]);
      match cmd {
        Cmd::Ping => {
          if msg_len == 1 {
            reply!(&[]);
          }
        }
      }
    }
  }
}
