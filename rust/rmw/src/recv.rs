use crate::{cmd::Cmd, hash128_bytes, key::hash128_bytes, pool::spawn, typedef::ToAddr};
use ed25519_dalek_blake3::{Keypair, Signature, Signer};
use expire_map::ExpireMap;
use std::net::UdpSocket;
use time::sec;
use twox_hash::xxh3::{hash128, hash64};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

pub struct Recv<Addr: ToAddr> {
  pub udp: UdpSocket,
  pub map: ExpireMap<Addr, u8>,
  pub key: Keypair,
  pub sk_hash: [u8; 16],
  pub expire: u8,
  pub secret: StaticSecret,
}

const PING_TOKEN_LEADING_ZERO: u32 = 16;

macro_rules! pk {
  ($key:expr) => {
    &$key.public.as_bytes()[..keygen::PK_LEN]
  };
}

fn send_to<Addr: ToAddr>(udp: &UdpSocket, msg: &[u8], addr: Addr) {
  err::log(udp.send_to(msg, addr))
}

fn sk_hash<Addr: ToAddr>(hash: &[u8], now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
  hash128_bytes!(hash, now, &addr.to_bytes(), msg)
}

impl<Addr: ToAddr> Recv<Addr> {
  pub fn new(key: Keypair, udp: UdpSocket, boot: Vec<Addr>) -> Self {
    let expire = config::get!(net / timeout / ping, 21u8);
    let map = ExpireMap::new(expire, 60);
    {
      let map = map.clone();
      let udp = udp.try_clone().unwrap();

      spawn(move || {
        for addr in boot {
          map.add(addr);
          send_to(&udp, &[Cmd::Ping as u8], addr);
        }
      });
    }
    let secret: StaticSecret = (&key.secret).into();
    Self {
      udp,
      map,
      sk_hash: hash128_bytes(key.secret.as_bytes()),
      key,
      expire: expire / 3,
      secret,
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
        reply!(Cmd::Ping, &self.pk(), &sec().to_le_bytes())
      }
    } else if let Ok(cmd) = Cmd::try_from(msg[0]) {
      println!("{} {:?} > {}", addr, &cmd, &msg.len());
      match cmd {
        Cmd::Ping => match msg_len {
          1 => reply!(&[]),
          39 => {
            let now = sec().to_le_bytes();
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
                      &crate::util::leading_zero::find(PING_TOKEN_LEADING_ZERO, &time_hash),
                    ]
                    .concat(),
                    src,
                  ),
                );
              });
            }
          }
          msg_len if msg_len >= 119 => {
            let udp = self.udp.try_clone().unwrap();
            let rpk: [u8; 30] = msg[1..31].try_into().unwrap();
            println!(">> {}", 3);
            let sign: [u8; 64] = msg[31..95].try_into().unwrap();
            println!(">> {}", 4);
            let time_hash_token = &msg[95..];
            let hash: [u8; 16] = time_hash_token[..16].try_into().unwrap();
            println!(">> {}", 5);
            let time_bytes = time_hash_token[16..24].try_into().unwrap();
            println!(">> {}", 6);
            let key = self.key.clone();
            let hash_token = hash64(&time_hash_token);
            let expire = self.expire as _;
            let sk = self.sk_hash;
            let secret = self.secret.clone();
            spawn(move || {
              let now = sec();
              let time = u64::from_le_bytes(time_bytes);
              let pk = pk!(key);
              if (time <= now)
                && ((now - time) <= expire)
                && (sk_hash(&sk, &time_bytes, &src, pk) == hash)
                && (hash_token.leading_zeros() >= PING_TOKEN_LEADING_ZERO)
              {
                let rpk = keygen::public_key_from_bytes(&rpk);
                println!(">> {}", 7);
                if let Ok(_) = rpk.verify_strict(&pk, &Signature::from_bytes(&sign).unwrap()) {
                  println!(">> {}", 8);
                  let xpk: X25519PublicKey = (&rpk).into();
                  let xsecret = secret.diffie_hellman(&xpk);
                  send_to(
                    &udp,
                    &[
                      &[Cmd::Ping as u8][..],
                      pk,
                      &hash128(xsecret.as_bytes()).to_le_bytes(),
                    ]
                    .concat(),
                    src,
                  )
                }
              }
            })
          }
          47 => {
            if self.map.has(addr) {
              let rpk: [u8; 30] = msg[1..31].try_into().unwrap();
              let hash: [u8; 16] = msg[31..].try_into().unwrap();
              let secret = self.secret.clone();
              spawn(move || {
                let rpk = keygen::public_key_from_bytes(&rpk);
                let xpk: X25519PublicKey = (&rpk).into();
                let xsecret = secret.diffie_hellman(&xpk);
                if hash128(xsecret.as_bytes()).to_le_bytes() == hash {
                  print!("connect success");
                }
              })
            }
          }
          _ => {}
        },
      }
    }
  }
  pub fn sk_hash(&self, now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
    sk_hash(&self.sk_hash, now, addr, msg)
  }
}
