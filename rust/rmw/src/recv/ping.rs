use crate::{
  hash128_bytes,
  key::hash128_bytes,
  typedef::ToAddr,
  util::udp::{send_to, Input},
  var::{self, EXPIRE, PING},
};
use async_std::task::{self, JoinHandle};
use ed25519_dalek_blake3::{Keypair, Signature, Signer};
use expire_map::ExpireMap;
use log::info;
use std::{
  mem::{swap, ManuallyDrop, MaybeUninit},
  net::{ToSocketAddrs, UdpSocket},
};
use x25519_dalek::StaticSecret;

pub const VERSION: &[u8] = &var::VERSION.to_le_bytes();

pub struct Ping<Addr: ToAddr> {
  pub expire: ExpireMap<Addr, (), u8>,
  pub secret: StaticSecret,
  pub key: Keypair,
  pub sk_hash: [u8; 16],
  pub timer: JoinHandle<()>,
}
impl<Addr: ToAddr> Drop for Ping<Addr> {
  fn drop(&mut self) {
    let mut timer = unsafe { MaybeUninit::uninit().assume_init() };
    swap(&mut timer, &mut self.timer);
    timer.cancel();
  }
}

macro_rules! pk {
  ($key:expr) => {
    &$key.public.as_bytes()[..keygen::PK_LEN]
  };
}

fn sk_hash<Addr: ToAddr>(hash: &[u8], now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
  hash128_bytes!(hash, now, &addr.to_bytes(), msg)
}

impl<Addr: ToAddr> Ping<Addr> {
  fn sk_hash(&self, now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
    sk_hash(&self.sk_hash, now, addr, msg)
  }

  fn pk(&self) -> &[u8] {
    pk!(self.key)
  }

  pub fn new(key: Keypair) -> Self {
    let secret: StaticSecret = (&key.secret).into();
    let (expire, timer) = ExpireMap::new(*EXPIRE, |&addr| info!("expire {:?}", addr));
    Self {
      expire,
      sk_hash: hash128_bytes(key.secret.as_bytes()),
      key,
      secret,
      timer,
    }
  }
  pub fn recv(&self, input: Input<Addr>) {}
  pub fn pong(&self, udp: &UdpSocket, addr: &Addr) -> bool {
    if self.expire.has(addr) {
      info!("{} > pong", addr);
      // 1 + 2 + 30  = 33
      send_to(udp, &[&PING, VERSION, self.pk()].concat(), addr);
      true
    } else {
      false
    }
  }
}
