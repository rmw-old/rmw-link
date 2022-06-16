use crate::{typedef::ToAddr, util::udp::Input, var::expire};
use expire_map::ExpireMap;
use std::net::ToSocketAddrs;

pub struct Ping<Addr: ToAddr> {
  pub expire: ExpireMap<Addr, (), u8>,
}

impl<Addr: ToAddr> Ping<Addr> {
  pub fn new() -> Self {
    let (expire, timer_expire_map) = ExpireMap::new(expire, |&addr| info!("expire {:?}", addr));
    Self { expire }
  }
  pub fn recv(input: Input<Addr>) {}
  pub fn has(&self, addr: &Addr) -> bool {
    self.expire.has(addr)
  }
}
