use crate::{
  kad::Kad,
  kad_net::kad_net,
  pool::spawn,
  recv::ping::Ping,
  typedef::ToAddr,
  util::udp::{send_to, Input},
  var::{self, PING},
};
use addrbytes::FromBytes;
use addrbytes::VecFromBytes;
use anyhow::Result;
use async_std::task::{self, JoinHandle};
use kv::Kv;
use log::info;
use parking_lot::Mutex;
use std::{
  mem::{self, ManuallyDrop},
  net::UdpSocket,
  sync::Arc,
};

pub struct Recv<Addr: ToAddr> {
  pub udp: UdpSocket,
  pub ping: Ping<Addr>,
}

pub const VERSION: &[u8] = &var::VERSION.to_le_bytes();
pub trait Boot<Addr> = Fn() -> Vec<Addr> + 'static + Send;

impl<Addr: ToAddr + FromBytes<Addr> + VecFromBytes<Addr>> Recv<Addr> {
  pub fn new(db: db::Db, kv: Kv, udp: UdpSocket, boot: impl Boot<Addr> + Sync) -> Self {
    let kv = Arc::new(kv);
    let ping = Ping::new(kv, udp.try_clone().unwrap(), boot);

    Self {
      //kv,
      //db,
      udp,
      ping,
    }
  }

  pub fn recv(&self, msg: &[u8], addr: Addr) -> Result<()> {
    let udp = &self.udp;

    match msg.len() {
      0 => {
        // TODO if kad as return xxx 心跳
        self.ping.pong(udp, &addr);
      }
      msg_len if msg_len >= 4 => {
        let input = Input {
          addr,
          udp: &self.udp,
          msg: &msg[4..],
        };
        match u32::from_le_bytes(msg[..4].try_into().unwrap()) {
          0 => self.ping.recv(input),
          id => {
            dbg!(id);
          }
        }
      }
      _ => {
        log::warn!("{} > {:?}", addr, msg);
      }
    }
    Ok(())
  }
}
