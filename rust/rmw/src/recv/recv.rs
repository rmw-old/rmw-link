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
  pub kv: Arc<Kv>,
  pub db: db::Db,
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
      kv,
      db,
      kad,
      udp,
      ping,
    }
  }

  pub fn recv(&self, msg: &[u8], addr: Addr) -> Result<()> {
    let msg_len = msg.len();
    let udp = &self.udp;

    if msg_len == 0 {
      self.ping.pong(udp, &addr);
      // TODO if kad as return xxx 心跳
    } else if msg_len >= 4 {
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
}