use crate::{typedef::ToAddr, util::udp::Input, var::EXPIRE};
use async_std::task::{self, JoinHandle};
use expire_map::ExpireMap;
use log::info;
use std::{
  mem::{swap, ManuallyDrop, MaybeUninit},
  net::ToSocketAddrs,
};

pub struct Ping<Addr: ToAddr> {
  pub expire: ExpireMap<Addr, (), u8>,
  pub timer: ManuallyDrop<JoinHandle<()>>,
}

impl<Addr: ToAddr> Ping<Addr> {
  pub fn new() -> Self {
    let (expire, timer) = ExpireMap::new(*EXPIRE, |&addr| info!("expire {:?}", addr));
    Self {
      expire,
      timer: ManuallyDrop::new(timer),
    }
  }
  pub fn recv(&self, input: Input<Addr>) {}
  pub fn has(&self, addr: &Addr) -> bool {
    self.expire.has(addr)
  }
}

impl<Addr: ToAddr> Drop for Ping<Addr> {
  fn drop(&mut self) {
    let mut timer = unsafe { MaybeUninit::uninit().assume_init() };
    swap(&mut timer, &mut *self.timer);
    timer.cancel();
  }
}
