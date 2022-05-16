use crate::{key, recv::Recv};
use anyhow::Result;
use async_std::task::spawn;
use ed25519_dalek_blake3::Keypair;
use log::info;
use std::{
  mem::MaybeUninit,
  net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs, UdpSocket},
};

pub struct Rmw {
  pub udp: UdpSocket,
  pub key: Keypair,
}

fn rmw<Addr: ToSocketAddrs>(addr: Addr) -> Result<Rmw> {
  let udp = UdpSocket::bind(addr)?;
  Ok(Rmw {
    udp,
    key: key::new(),
  })
}

pub fn v4() -> Result<Rmw> {
  let addr = config::get!(
    udp / v4,
    UdpSocket::bind("0.0.0.0:0").unwrap().local_addr().unwrap()
  );
  rmw(addr)
}

const MTU: usize = 1472;

impl Rmw {
  pub async fn run(&self) -> Result<()> {
    let udp = &self.udp;
    let addr = udp.local_addr()?;
    info!("udp://{:?}", addr);

    let mut buf: [u8; MTU] = unsafe { MaybeUninit::uninit().assume_init() };
    let (mut n, mut src);
    macro_rules! run {
      ($v:ident,$addr:expr) => {
        let recv = Recv::new(self.key.clone(), udp.try_clone()?, $addr);

        loop {
          (n, src) = udp.recv_from(&mut buf)?;
          match src {
            SocketAddr::$v(src) => recv.recv(&buf[..n], src),
            _ => {}
          };
        }
      };
    }

    match addr {
      SocketAddr::V6(_) => {
        run!(
          V6,
          config::get!(
            boot / v6,
            vec![SocketAddrV6::new(
              //  2600:1f1c:626:9200:7b9a:4420:876a:4550
              Ipv6Addr::new(0x2600, 0x1f1c, 0x626, 0x9200, 0x7b9a, 0x4420, 0x876a, 0x4550),
              4910,
              0,
              0
            )]
          )
        );
      }
      SocketAddr::V4(_) => {
        if cfg!(feature = "upnp") && config::get!(upnp / v4, true) {
          spawn(upnp::upnp_daemon("rmw", addr.port()));
        }
        run!(
          V4,
          config::get!(
            boot / v4,
            //54.177.127.37
            vec![SocketAddrV4::new(Ipv4Addr::new(54, 177, 127, 37), 4910)]
          )
        );
      }
    };
  }
}
