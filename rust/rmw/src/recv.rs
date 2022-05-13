use crate::{cmd::Cmd, hash128_bytes, key::hash128_bytes, pool::POOL, typedef::ToAddr};
use async_std::task::spawn;
use ed25519_dalek_blake3::Keypair;
use expire_map::ExpireMap;
use log::info;
use rayon::ThreadPool;
use std::net::UdpSocket;

pub struct Recv<Addr: ToAddr> {
  pub udp: UdpSocket,
  pub map: ExpireMap<Addr, u8>,
  pub sk_hash: [u8; 16],
  pub pk: [u8; keygen::PK_LEN],
}

impl<Addr: ToAddr> Recv<Addr> {
  pub fn new(key: &Keypair, udp: UdpSocket, boot: Vec<Addr>) -> Self {
    let map = ExpireMap::new(config::get!(net / timeout / ping, 7u8), 60);
    let u = udp.try_clone().unwrap();
    let m = map.clone();

    POOL.spawn(move || {
      for addr in boot {
        m.add(addr);
        err::log(u.send_to(&[Cmd::Ping as u8], addr));
      }
    });

    Self {
      udp,
      map,
      pk: key.public.as_bytes()[..keygen::PK_LEN].try_into().unwrap(),
      sk_hash: hash128_bytes(key.secret.as_bytes()),
    }
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
      if self.map.renew(addr) {
        println!("{} > ping reply", addr);
        let now = time::sec().to_le_bytes();
        let pk = &self.pk;
        reply!(
          Cmd::Ping,
          hash128_bytes!(&self.sk_hash, &now, &addr.to_bytes(), pk),
          &now,
          pk
        )
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      println!("{} {:?} > {}", addr, &cmd, &msg.len());
      match cmd {
        Cmd::Ping => match msg_len {
          1 => reply!(&[]),
          55 => {
            let now = time::sec().to_le_bytes();
            reply!(
              Cmd::Ping,
              &msg[1..25],
              hash128_bytes!(&self.sk_hash, &now, &addr.to_bytes(), &msg[25..]),
              &now
            )
          }
          49 => {
            //send!(PingPk)
            /*
            reply!(Cmd::Ping,
            &self.pk
            )
            */
            //reply!(ping_pk!(&self.pk, addr, msg));
          }
          _ => {}
        },
      }
    }
  }
}
