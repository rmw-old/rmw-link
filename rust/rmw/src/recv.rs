use crate::{cmd::Cmd, hash128_bytes, key::hash128_bytes, pool::spawn, typedef::ToAddr};
use ed25519_dalek_blake3::{Keypair, Signer};
use expire_map::ExpireMap;
use std::net::UdpSocket;

pub struct Recv<Addr: ToAddr> {
  pub udp: UdpSocket,
  pub map: ExpireMap<Addr, u8>,
  pub key: Keypair,
  pub sk_hash: [u8; 16],
}

macro_rules! pk {
  ($key:expr) => {
    &$key.public.as_bytes()[..keygen::PK_LEN]
  };
}

impl<Addr: ToAddr> Recv<Addr> {
  pub fn new(key: Keypair, udp: UdpSocket, boot: Vec<Addr>) -> Self {
    let map = ExpireMap::new(config::get!(net / timeout / ping, 21u8), 60);
    {
      let udp = udp.try_clone().unwrap();
      let map = map.clone();

      spawn(move || {
        for addr in boot {
          map.add(addr);
          err::log(udp.send_to(&[Cmd::Ping as u8], addr));
        }
      });
    }
    Self {
      udp,
      map,
      sk_hash: hash128_bytes(key.secret.as_bytes()),
      key,
    }
  }

  fn pk(&self) -> &[u8] {
    pk!(self.key)
  }

  pub fn recv(&self, msg: &[u8], src: Addr) {
    let msg_len = msg.len();
    let addr = &src;
    let udp = &self.udp;

    macro_rules! reply {
      ($bin:expr) => {{
        err::log(udp.send_to($bin, src));
      }};
      ($cmd:expr,$($bin:expr),*) => {{
        reply!(
          &[
          &[$cmd as u8][..],
          $($bin),*
          ].concat()
        )
      }};
    }

    if msg_len == 0 {
      if self.map.has(addr) {
        println!("{} > ping reply", addr);
        reply!(Cmd::Ping, &self.pk(), &time::sec().to_le_bytes())
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      println!("{} {:?} > {}", addr, &cmd, &msg.len());
      match cmd {
        Cmd::Ping => match msg_len {
          1 => reply!(&[]),
          39 => {
            let now = time::sec().to_le_bytes();
            let pk: [u8; 30] = msg[1..31].try_into().unwrap();
            if pk != self.pk() {
              reply!(Cmd::Ping, &self.sk_hash(&now, addr, &pk), &now)
            }
          }
          25 => {
            if self.map.has(addr) {
              let udp = self.udp.try_clone().unwrap();
              let key = self.key.clone();
              let time_hash: [u8; 24] = msg[1..].try_into().unwrap();
              spawn(move || {
                err::log(
                  udp.send_to(
                    &[
                      &[Cmd::Ping as u8][..],
                      pk!(key),
                      &key.sign(&time_hash).to_bytes(),
                      &time_hash,
                      &crate::util::leading_zero::find(8, &time_hash),
                    ]
                    .concat(),
                    src,
                  ),
                );
              });
            }
          }
          msg_len if msg_len >= 119 => {}
          47 => {}
          _ => {}
        },
      }
    }
  }
  pub fn sk_hash(&self, now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
    hash128_bytes!(&self.sk_hash, now, &addr.to_bytes(), msg)
  }
}
