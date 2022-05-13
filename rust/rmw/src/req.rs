use std::net::SocketAddr;

use crate::typedef::ToAddr;
use enum_dispatch::enum_dispatch;
use speedy::{Error, Readable, Writable};

pub type BoxResult = Result<Option<Box<[u8]>>, Error>;

#[derive(PartialEq, Debug, Readable, Writable)]
#[speedy(tag_type=u8)]
#[enum_dispatch(On)]
#[repr(u8)]
pub enum Req<Addr: ToAddr> {
  Ping(Recv<Addr>),
}

#[derive(PartialEq, Debug, Readable, Writable)]
pub struct Recv<Addr: ToAddr> {
  msg: Box<[u8]>,
  src: Addr,
}

/*
#[enum_dispatch]
pub trait On {
  fn on(&self) -> BoxResult;
}
impl On for Recv {
  fn on(&self) -> BoxResult {
    dbg!(self);
    //log(api.v4.send_to(self.path.as_ref(), self.addr));
    Ok(None)
  }
}
*/
