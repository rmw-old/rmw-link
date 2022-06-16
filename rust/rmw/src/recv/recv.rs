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
use twox_hash::xxh3::{hash128, hash64};

pub struct Recv<Addr: ToAddr> {
  pub timer: ManuallyDrop<[JoinHandle<()>; 1]>,
  pub udp: UdpSocket,
  pub ping: Ping<Addr>,
  //pub kv: Arc<Kv>,
  //pub db: db::Db,
  pub kad: Arc<Mutex<Kad<Addr>>>,
}

pub const VERSION: &[u8] = &var::VERSION.to_le_bytes();

impl<Addr: ToAddr> Drop for Recv<Addr> {
  fn drop(&mut self) {
    let mut timer = unsafe { mem::MaybeUninit::uninit().assume_init() };
    mem::swap(&mut timer, &mut *self.timer);
    timer.map(|i| task::spawn(i.cancel()));
  }
}

pub trait Boot<Addr> = Fn() -> Vec<Addr> + 'static + Send;

impl<Addr: ToAddr + FromBytes<Addr> + VecFromBytes<Addr>> Recv<Addr> {
  pub fn new(db: db::Db, kv: Kv, udp: UdpSocket, boot: impl Boot<Addr> + Sync) -> Self {
    let kv = Arc::new(kv);
    let ping = Ping::new(kv.clone());
    let kad = Arc::new(Mutex::new(Kad::new(ping.key.public.as_bytes())));

    Self {
      timer: ManuallyDrop::new([
        /*
        task::spawn(
        async move { ip_sk_expire.monitor(2, 0, Duration::from_secs(3)).await },
        ),
        */
        task::spawn(kad_net(
          kad.clone(),
          boot,
          udp.try_clone().unwrap(),
          kv.clone(),
          ping.expire.clone(),
        )),
      ]),
      //kv,
      //db,
      kad,
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
