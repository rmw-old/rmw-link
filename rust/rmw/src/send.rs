use crate::{cmd::Cmd, hash128_bytes, key::hash128_bytes, req::Req, typedef::ToAddr};
use async_std::{channel::Sender, task::spawn};
use ed25519_dalek_blake3::Keypair;
use expire_map::ExpireMap;
use log::info;
use std::net::UdpSocket;

pub struct Send<Addr: ToAddr> {
  pub send: Sender<Req>,
  pub udp: UdpSocket,
  pub map: ExpireMap<Addr, u8>,
  pub sk_hash: [u8; 16],
  pub pk: [u8; keygen::PK_LEN],
}

const PING: [u8; 1] = [Cmd::Ping as u8];

macro_rules! ping_syn {
  ($sk_hash:expr, $addr:expr, $pk:expr) => {{
    let now = time::sec().to_le_bytes();
    &[
      &PING[..],
      hash128_bytes!($sk_hash, &now, &$addr.to_bytes(), $pk),
      &now,
      $pk,
    ]
    .concat()
  }};
}

macro_rules! ping_ack {
  ($sk_hash:expr, $addr:expr, $msg:expr) => {{
    let now = time::sec().to_le_bytes();
    &[
      &PING[..],
      &now,
      hash128_bytes!($sk_hash, &now, &$addr.to_bytes(), &$msg[25..]),
      &$msg[1..25],
    ]
    .concat()
  }};
}

impl<Addr: ToAddr> Send<Addr> {
  pub fn new(send: Sender<Req>, key: &Keypair, udp: UdpSocket, boot: Vec<Addr>) -> Self {
    let map = ExpireMap::new(config::get!(net / timeout / ping, 7u8), 60);
    let u = udp.try_clone().unwrap();
    let m = map.clone();
    spawn(async move {
      for addr in boot {
        m.add(addr);
        err::log(u.send_to(&[Cmd::Ping as u8], addr));
      }
    });
    Self {
      udp,
      send,
      map,
      pk: key.public.as_bytes()[..keygen::PK_LEN].try_into().unwrap(),
      sk_hash: hash128_bytes(key.secret.as_bytes()),
    }
  }

  pub async fn send(&self, msg: &[u8], src: Addr) {
    let msg_len = msg.len();
    let addr = &src;
    let udp = &self.udp;

    macro_rules! reply {
      ($bin:expr) => {{
        err::log(udp.send_to($bin, src));
      }};
    }

    if msg_len == 0 {
      if self.map.renew(addr) {
        println!("{} > ping reply", addr);
        reply!(ping_syn!(&self.sk_hash, addr, &self.pk))
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      println!("{} {:?} > {}", addr, &cmd, &msg.len());
      match cmd {
        Cmd::Ping => match msg_len {
          1 => reply!(&[]),
          55 => {
            reply!(ping_ack!(&self.sk_hash, addr, msg));
          }
          _ => {}
        },
      }
    }
  }
}
