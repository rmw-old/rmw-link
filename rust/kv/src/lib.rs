#![feature(macro_metavar_expr)]

pub mod cf;
pub mod db;

use addrbytes::FromBytes;
use anyhow::Result;
pub use db::Kv;
use id::Id;
use once_cell::unsync::Lazy;
use paste::paste;
use rocksdb::{Direction, IteratorMode};
use smallvec::SmallVec;

column_family!(pk_addr, addr_pk, addr_sk, alive_addr);

macro_rules! tx_cf {
  ($self:ident)=>{
    let cf = &$self.cf;
    let tx = $self.db.transaction();

    macro_rules! define {
      ($op:ident) => {
        paste! {
          #[allow(unused_macros)]
          macro_rules! $op {
            ($cf:ident,$$$$($$$$args:expr),*) => {{
              tx.[<$op _cf>](&cf.$cf,$$$$($$$$args),*)?
            }};
          }
        }
      };
      ($$($$op:ident),+) => {
        $$(define!($$op);)+
      };
    }

    define!(put, get, delete, get_pinned);
  }
}

#[derive(Debug)]
pub struct PkAddr<Addr: FromBytes<Addr>> {
  pub pk: Box<[u8]>,
  pub addr: Addr,
}

const KAD_LEN: usize = 32;
impl Kv {
  pub fn addr_range<Addr: FromBytes<Addr>>(
    &self,
    begin: u128,
    end: u128,
  ) -> SmallVec<[PkAddr<Addr>; KAD_LEN]> {
    let cf = &self.cf;
    let _end = end.to_be_bytes();
    let mut li = SmallVec::new();
    for (pk, addr) in self.db.iterator_cf(
      &cf.pk_addr,
      IteratorMode::From(&begin.to_be_bytes(), Direction::Forward),
    ) {
      let key: [u8; 16] = pk[..16].try_into().unwrap();
      if key > _end {
        break;
      }
      li.push(PkAddr {
        pk,
        addr: Addr::from_bytes(&addr),
      });
      if li.len() >= KAD_LEN {
        break;
      }
    }
    li
  }

  pub fn addr_pk_set(&self, addr: &[u8], pk: &[u8]) -> Result<()> {
    tx_cf!(self);

    if let Some(pre) = get!(pk_addr, pk) {
      if addr == pre {
        return Ok(());
      }
      delete!(addr_pk, pre);
    }

    if let Some(pre) = get!(addr_pk, addr) {
      delete!(pk_addr, pre);
    }

    put!(addr_pk, addr, pk);
    put!(pk_addr, pk, addr);
    Ok(())
  }
}

static mut ADDR_TIME_ID: Lazy<Id> = Lazy::new(Id::default);
impl Kv {
  pub fn addr_sk_set(&self, addr: &[u8], sk: &[u8]) -> Result<()> {
    tx_cf!(self);
    if let Some(id) = get!(addr_sk, addr) {
      delete!(addr_sk, id);
    }
    loop {
      let id = unsafe { ADDR_TIME_ID.get() };
      if get!(alive_addr, id).is_none() {
        put!(alive_addr, id, addr);
        break;
      }
    }
    put!(addr_sk, addr, sk);
    Ok(())
  }

  pub fn addr_sk_decrypt_encrypt(
    &self,
    addr: &[u8],
    iv: &[u8],
    msg: &[u8],
    func: impl Fn(&[u8]) -> Box<[u8]>,
  ) -> Result<Option<Box<[u8]>>> {
    tx_cf!(self);
    if let Some(sk) = get_pinned!(addr_sk, addr) {
      if let Some(msg) = xxblake3::decrypt(&sk, iv, msg) {
        return Ok(Some(xxblake3::encrypt(&sk, iv, &func(&msg))));
      }
    };
    Ok(None)
  }

  pub fn addr_sk_decrypt(&self, addr: &[u8], iv: &[u8], msg: &[u8]) -> Result<Option<Box<[u8]>>> {
    tx_cf!(self);
    let r = if let Some(sk) = get_pinned!(addr_sk, addr) {
      xxblake3::decrypt(&sk, iv, msg)
    } else {
      None
    };
    Ok(r)
  }

  pub fn addr_sk_encrypt(&self, addr: &[u8], iv: &[u8], msg: &[u8]) -> Result<Option<Box<[u8]>>> {
    tx_cf!(self);
    let r = get_pinned!(addr_sk, addr).map(|sk| xxblake3::encrypt(&sk, iv, msg));
    Ok(r)
  }
}
