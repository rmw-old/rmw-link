use crate::{recv::Recv, send::Send};
use anyhow::Result;
use async_std::{channel::unbounded, task::spawn};
use db::Db;
use log::info;
use std::{
  mem::MaybeUninit,
  net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
};

pub struct Rmw {
  pub udp: UdpSocket,
  pub db: Db,
}

pub async fn rmw() -> Result<Rmw> {
  let addr = config::get!(
    udp / v4,
    UdpSocket::bind("0.0.0.0:0").unwrap().local_addr().unwrap()
  );
  let udp = UdpSocket::bind(addr)?;
  Ok(Rmw {
    udp,
    db: db::open(dir::ROOT.clone().join("db/duck"))?,
  })
}

const MTU: usize = 1472;

impl Rmw {
  pub async fn run(&self) -> Result<()> {
    let udp = &self.udp;
    let addr = udp.local_addr()?;
    info!("udp://{:?}", addr);
    let (send, recv) = unbounded();

    let recv = Recv {
      udp: udp.try_clone()?,
      recv,
    };

    spawn(async move { recv.recv().await });

    let mut buf: [u8; MTU] = unsafe { MaybeUninit::uninit().assume_init() };
    let (mut n, mut src);
    macro_rules! run {
      ($v:ident,$addr:expr) => {
        let send = Send::new(send, udp.try_clone()?, $addr);
        loop {
          (n, src) = udp.recv_from(&mut buf)?;
          if n > 0 {
            match src {
              SocketAddr::$v(src) => send.send(&buf[..n], src).await,
              _ => {}
            }
          }
        }
      };
    }

    match addr {
      SocketAddr::V4(_) => {
        if cfg!(feature = "upnp") && config::get!(upnp / v4, true) {
          spawn(upnp::upnp_daemon("rmw", addr.port()));
        }
        run!(
          V4,
          config::get!(
            boot / v4,
            //54.177.127.37
            vec![SocketAddrV4::new(Ipv4Addr::new(54, 177, 127, 37), 30110)]
          )
        );
      }
      SocketAddr::V6(_) => {
        run!(
          V6,
          config::get!(
            boot / v6,
            vec![SocketAddrV6::new(
              //  2600:1f1c:626:9200:7b9a:4420:876a:4550
              Ipv6Addr::new(0x2600, 0x1f1c, 0x626, 0x9200, 0x7b9a, 0x4420, 0x876a, 0x4550),
              30110,
              0,
              0
            )]
          )
        );
      }
    };
  }
}
