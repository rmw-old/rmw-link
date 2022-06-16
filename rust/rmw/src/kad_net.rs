use crate::{kad::Kad, midpoint, recv::recv::Boot, typedef::ToAddr, util::udp::send_to, var::PING};
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
    send_to(&udp, &PING, addr);
  };
  let range = kad.lock().range();

  macro_rules! is_empty {
    ($run:expr) => {{
      sleep(3).await;
      let is_empty = kad.lock().is_empty();
      if is_empty {
        sleep(30).await;
      } else {
        // 以太坊p2p网络(二)：以太坊P2P节点发现算法原理剖析 https://blog.csdn.net/guidao13/article/details/82798422
        info!("连接更多的端口，直到没有新的，清理rocksdb; 查找随机节点；填充kad");

        let key = kad.lock().key;
        let node = &kad.lock().node;
        let max = node.len() - 1;

        for (pos,li) in node.iter().enumerate() {
          for i in li {
            let _addr = &i.addr;
            let _find = if pos == max { key } else {
              let rp = &range[pos];
              midpoint!(rp.start(),rp.end())
            }.to_be_bytes();
              /*
            if let Ok(Some(v)) = kv.addr_sk_encrypt(&addr.to_bytes(),&find) {
              //todo
              dbg!(v.len());
              let msg = [
                &[Cmd::FindNode as u8][..],
                &v
              ].concat();
              send_to(&udp,&msg,addr)
            }
              */
          }
        }

        /*
        let max = &kad.lock().node.len()-1;
        for (pos, li) in node.iter().enumerate() {
          let len  = li.len();
          if len != CAPACITY {
            let key = if pos == max {
              kad.key
            } else {
              let rp = &range[pos];
              midpoint!(rp.start(), rp.end())
            }
          }
        }
        sleep(60).await;
        */
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
