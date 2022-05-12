use crate::{cmd::Cmd, hash128_bytes, key::hash128_bytes, req::Req, typedef::ToAddr};
use addrbytes::ToBytes;
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

macro_rules! ping_syn {
  ($sk_hash:expr, $addr:expr, $pk:expr) => {{
    let now = time::sec().to_le_bytes();
    &[
      &[Cmd::Ping as u8][..],
      hash128_bytes!($sk_hash, &$addr.to_bytes(), &now, $pk),
      &now,
      $pk,
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

  pub async fn send(&self, msg: &[u8], addr: Addr) {
    let msg_len = msg.len();
    let addr = &addr;
    let udp = &self.udp;

    macro_rules! reply {
      ($bin:expr) => {{
        err::log(udp.send_to($bin, addr));
      }};
    }

    if msg_len == 0 {
      if self.map.renew(addr) {
        info!("{:?} > ping reply", addr);
        reply!(ping_syn!(&self.sk_hash, addr, &self.pk))
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      info!("{:?} {:?} {}", addr, &cmd, &msg[1..].len());
      match cmd {
        Cmd::Ping => match msg_len {
          1 => reply!(&[]),
          _ => {}
        },
      }
    }
  }
}
