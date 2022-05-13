use crate::typedef::ToAddr;
//use enum_dispatch::enum_dispatch;
//use speedy::{Error, Readable, Writable};
//use anyhow::Result;
//pub type BoxResult = Result<Option<Box<[u8]>>>;

//#[derive(PartialEq, Debug, Readable, Writable)]
//#[speedy(tag_type=u8)]
//#[enum_dispatch(On)]
pub enum Req<Addr: ToAddr> {
  PingPk(Recv<Addr>),
}

pub struct Recv<Addr: ToAddr> {
  pub msg: Box<[u8]>,
  pub src: Addr,
}
