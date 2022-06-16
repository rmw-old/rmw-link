mod ping;

use crate::{
  hash128_bytes,
  kad::Kad,
  kad_net::kad_net,
  key,
  key::hash128_bytes,
  pool::spawn,
  typedef::ToAddr,
  util::udp::send_to,
  var::{self, PING},
};
use addrbytes::FromBytes;
use addrbytes::VecFromBytes;
use anyhow::Result;
use async_std::task::{self, JoinHandle};
use ed25519_dalek_blake3::{Keypair, Signature, Signer};
use expire_map::ExpireMap;
use kv::Kv;
use log::info;
use parking_lot::Mutex;
use std::{
  mem::{self, ManuallyDrop},
  net::UdpSocket,
  sync::Arc,
};
use time::sec;
use twox_hash::xxh3::{hash128, hash64};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

pub struct Recv<Addr: ToAddr> {
  pub timer: ManuallyDrop<[JoinHandle<()>; 2]>,
  pub udp: UdpSocket,
  pub ping: ExpireMap<Addr, (), u8>,
  pub key: Keypair,
  pub sk_hash: [u8; 16],
  pub expire: u8,
  pub secret: StaticSecret,
  //pub ip_sk: Arc<Cache<Addr, SharedSecret>,
  pub kv: Arc<Kv>,
  pub db: db::Db,
  pub kad: Arc<Mutex<Kad<Addr>>>,
}

const PING_TOKEN_LEADING_ZERO: u32 = 16;
pub const VERSION: &[u8] = &var::VERSION.to_le_bytes();

macro_rules! pk {
  ($key:expr) => {
    &$key.public.as_bytes()[..keygen::PK_LEN]
  };
}

fn sk_hash<Addr: ToAddr>(hash: &[u8], now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
  hash128_bytes!(hash, now, &addr.to_bytes(), msg)
}

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
    let key = key::new(&kv);
    let expire = config::get!(net / timeout / ping, 21u8);
    let (ping, timer_expire_map) = ExpireMap::new(expire, |&addr| info!("expire {:?}", addr));

    let kad = Arc::new(Mutex::new(Kad::new(key.public.as_bytes())));
    let kv = Arc::new(kv);

    let secret: StaticSecret = (&key.secret).into();
    Self {
      timer: ManuallyDrop::new([
        /*
        task::spawn(
        async move { ip_sk_expire.monitor(2, 0, Duration::from_secs(3)).await },
        ),
        */
        timer_expire_map,
        task::spawn(kad_net(
          kad.clone(),
          boot,
          udp.try_clone().unwrap(),
          kv.clone(),
          ping.clone(),
        )),
      ]),
      kv,
      db,
      kad,
      udp,
      //ip_sk,
      ping,
      sk_hash: hash128_bytes(key.secret.as_bytes()),
      key,
      expire: expire / 3,
      secret,
    }
  }

  fn pk(&self) -> &[u8] {
    pk!(self.key)
  }

  pub fn recv(&self, msg: &[u8], src: Addr) -> Result<()> {
    let msg_len = msg.len();
    let addr = &src;
    let udp = &self.udp;
    //dbg!(self.cf.pk.put(&[1, 2, 3], &[5, 6]))?;
    //dbg!(self.cf.pk.get(&[1, 2, 3]))?;

    macro_rules! send_to {
      ($udp:expr,$bin:expr) => {{
        send_to(&$udp,$bin,src);
      }};
      ($udp:expr,$($bin:expr),+) => {{
        send_to!(
          $udp,
          &[
          $($bin),+
          ].concat()
        )
      }};
    }

    macro_rules! reply {
      ($bin:expr) => {{
        send_to!(udp,$bin)
      }};
      ($cmd:expr,$($bin:expr),+) => {{
        send_to!(udp,$cmd,$($bin),+)
      }}
    }

    if msg_len == 0 {
      if self.ping.has(addr) {
        println!("{} > ping reply", addr);
        // 1 + 2 + 30  = 33
        reply!(&PING, VERSION, self.pk())
      }
      // TODO if kad as return xxx 心跳
    } else if msg_len > 4 {
      /*
             println!("{} {:?} > {}", addr, &cmd, &msg.len());

             macro_rules! kad_add {
             ($kad:expr,$pk_bytes:expr,$kv:expr,$xsecret:expr) => {{
             let src_bytes = &src.to_bytes();
             if $kad.lock().add(&$pk_bytes, src) {
             err::log($kv.addr_pk_set(src_bytes, &$pk_bytes));
             }
             err::log($kv.addr_sk_set(src_bytes, $xsecret));
             }};
             }

             match cmd {
             Cmd::Ping => match msg_len {
             1 => reply!(&[]),
             33 => {
             let pk: [u8; 30] = msg[3..33].try_into().unwrap();
             if pk != self.pk() {
             let now = sec().to_le_bytes();
          // 1 + 16 + 8 = 25
          reply!(Cmd::Ping, &self.sk_hash(&now, addr, &pk), &now)
          }
          }
          25 => {
          if self.ping.has(addr) {
          let udp = self.udp.try_clone().unwrap();
          let key = self.key.clone();
          let hash_time: [u8; 24] = msg[1..].try_into().unwrap();
          spawn(move || {
          send_to!(
          udp,
          Cmd::Ping,
          pk!(key),
          &key.sign(&hash_time).to_bytes(),
          &hash_time,
          &crate::util::leading_zero::find(PING_TOKEN_LEADING_ZERO, &hash_time)
          );
          });
          }
          }
          msg_len if msg_len >= 119 => {
          let udp = self.udp.try_clone().unwrap();
          let rpk_bytes: [u8; 30] = msg[1..31].try_into().unwrap();
          let sign: [u8; 64] = msg[31..95].try_into().unwrap();
          let hash_time: [u8; 24] = msg[95..119].try_into().unwrap();
          let key = self.key.clone();
          let hash_token = hash64(&msg[95..]);
          let expire = self.expire as _;
          let sk = self.sk_hash;
          let secret = self.secret.clone();
          let kv = self.kv.clone();
          let kad = self.kad.clone();

          spawn(move || {
          let pk = pk!(key);
          let hash: [u8; 16] = hash_time[..16].try_into().unwrap();
          let time_bytes = hash_time[16..].try_into().unwrap();
          let now = sec();
          let time = u64::from_le_bytes(time_bytes);
          if (time <= now)
          && ((now - time) <= expire)
          && (sk_hash(&sk, &time_bytes, &src, &rpk_bytes) == hash)
          && (hash_token.leading_zeros() >= PING_TOKEN_LEADING_ZERO)
          {
          let rpk = keygen::public_key_from_bytes(&rpk_bytes);
          if rpk
          .verify_strict(&hash_time, &Signature::from_bytes(&sign).unwrap())
          .is_ok()
          {
          let xpk: X25519PublicKey = (&rpk).into();
          let xsecret = secret.diffie_hellman(&xpk);
          let xsecret = xsecret.as_bytes();

          kad_add!(kad, rpk_bytes, kv, xsecret);

          send_to!(udp, Cmd::Ping, pk, &hash128(xsecret).to_le_bytes())
        }
        }
      })
      }
      47 => {
        if self.ping.pop(addr) {
          let rpk_bytes: [u8; 30] = msg[1..31].try_into().unwrap();
          let hash: [u8; 16] = msg[31..].try_into().unwrap();
          let kv = self.kv.clone();
          let secret = self.secret.clone();
          let kad = self.kad.clone();

          spawn(move || {
            let rpk = keygen::public_key_from_bytes(&rpk_bytes);
            let xpk: X25519PublicKey = (&rpk).into();
            let xsecret = secret.diffie_hellman(&xpk);
            let xsecret = xsecret.as_bytes();
            if hash128(xsecret).to_le_bytes() == hash {
              kad_add!(kad, rpk_bytes, kv, xsecret);
            }
          })
        }
      }
      _ => {}
      },
      Cmd::FindNode => {
        match msg_len {
          25 => {
            let kad = self.kad.clone();
            let kv = self.kv.clone();
            let udp = self.udp.try_clone().unwrap();
            let msg: [u8; 24] = msg[1..].try_into().unwrap();
            spawn(move || {
              let addr = src.to_bytes();
              if let Ok(Some(r)) =
                kv.addr_sk_decrypt_encrypt(&addr, &msg, |node| kad.lock().find(node))
                {
                  send_to!(udp, Cmd::FindedNode, &r)
                };
            });
          }
          _ => {}
        }
        //if let Some(r) = self.kv.addr_sk_encrypt()
        //let xsecret = addr();
      }
      Cmd::FindedNode => {
        let kv = self.kv.clone();
        if let Ok(Some(msg)) = kv.addr_sk_decrypt(&src.to_bytes(), &msg[1..]) {
          let li = Addr::vec_from_bytes(msg);
          let kad = &self.kad.lock();
          for i in li {
            if !kad.addr.contains_key(&i) {
              dbg!(("todo ping", i));
            }
          }
        }
      }
      }
      */
    }
    Ok(())
  }
  pub fn sk_hash(&self, now: &[u8], addr: &Addr, msg: &[u8]) -> [u8; 16] {
    sk_hash(&self.sk_hash, now, addr, msg)
  }
}
