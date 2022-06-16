use crate::{doh, recv::recv::Recv};
use addrbytes::VecFromBytes;
use anyhow::Result;
use async_std::task::spawn;
use paste::paste;
use std::{
  mem::MaybeUninit,
  net::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs, UdpSocket},
};

pub struct Rmw {
  pub udp: UdpSocket,
}

fn rmw<Addr: ToSocketAddrs>(addr: Addr) -> Result<Rmw> {
  // ipv6 转换 http://www.gestioip.net/cgi-bin/subnet_calculator.cgi

  let udp = UdpSocket::bind(addr)?;

  Ok(Rmw { udp })
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
    log::info!("udp://{:?}", addr);

    let mut buf: [u8; MTU] = unsafe { MaybeUninit::uninit().assume_init() };
    let (mut n, mut src);

    macro_rules! run {
      ($v:ident) => {{
        let v = &stringify!($v)[1..];
        let host = v.to_owned()
          + &".".to_owned()
          + &config::get!(net / boot, "rmw0.tk".to_string()).as_str();

        let root = dir::ROOT.clone().join("db").join(v);
        let recv = Recv::new(
          db::open(root.join("duck"))?,
          kv::Kv::new(root.join("kv")),
          udp.try_clone()?,
          move || paste! {[<SocketAddr $v>]::vec_from_bytes}(doh::addr(&host)),
        );

        loop {
          (n, src) = udp.recv_from(&mut buf)?;
          match src {
            SocketAddr::$v(src) => err::log(recv.recv(&buf[..n], src)),
            _ => {}
          };
        }
      }};
    }
    match addr {
      SocketAddr::V6(_) => run!(V6),
      SocketAddr::V4(_) => {
        if cfg!(feature = "upnp") && config::get!(upnp / v4, true) {
          spawn(upnp::upnp_daemon("rmw", addr.port()));
        }
        run!(V4)
      }
    };
  }
}
