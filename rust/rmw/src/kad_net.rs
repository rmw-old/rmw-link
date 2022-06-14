use crate::{
  cmd::Cmd,
  kad::{Kad, CAPACITY},
  recv::Boot,
  typedef::ToAddr,
  util::udp::send_to,
};
use expire_map::ExpireMap;
use kv::Kv;
use log::info;
use parking_lot::Mutex;
use std::{net::UdpSocket, sync::Arc};
use time::r#async::sleep;

pub async fn kad_net<Addr: ToAddr + addrbytes::FromBytes<Addr>>(
  kad: Arc<Mutex<Kad<Addr>>>,
  boot: impl Boot<Addr> + Sync,
  udp: UdpSocket,
  kv: Arc<Kv>,
  ping: ExpireMap<Addr, (), u8>,
) {
  let send = |addr| {
    ping.add(addr, ());
    send_to(&udp, &[Cmd::Ping as u8], addr);
  };
  let range = kad.lock().range();

  macro_rules! is_empty {
    ($run:expr) => {{
      sleep(3).await;
      let is_empty = kad.lock().is_empty();
      if is_empty {
        sleep(30).await;
      } else {
        info!("连接更多的端口，直到没有新的，清理rocksdb");
        for (pos, i) in (&kad.lock()).node.iter().enumerate() {
          if i.len() != CAPACITY {
            let rp = &range[pos];
            dbg!((pos, rp.start(), rp.end()));
          }
        }
        break;
      }
    }};
  }
  macro_rules! boot {
    () => {
      boot().iter().for_each(|addr| send(*addr))
    };
  }

  loop {
    let mut empty = true;
    for i in &range {
      let li = kv.addr_range::<Addr>(*i.start(), *i.end());
      if !li.is_empty() {
        empty = false;
        // todo 快速重连，利用列目录
        for node in li {
          send(node.addr);
        }
      }
    }
    if empty {
      boot!();
    }

    is_empty!({
      if !empty {
        boot!();
        is_empty!({ sleep(30).await })
      } else {
        sleep(30).await;
      }
    });
  }
}
