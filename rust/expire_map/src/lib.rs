#![feature(hash_drain_filter)]
#![feature(trait_alias)]

use async_std::task::{sleep, spawn};
use num::traits::{AsPrimitive, WrappingSub};
use parking_lot::RwLock;
use rand::{thread_rng, Rng};
use std::marker::Copy;
use std::{collections::HashMap, hash::Hash, sync::Arc, time::Duration};
use time::sec;
//use tokio::{spawn, time::sleep};
pub trait SyncSend =
  'static + std::marker::Sync + std::marker::Send + Eq + Hash + std::fmt::Debug + Copy;
pub trait Num = AsPrimitive<u64>
  + WrappingSub
  + std::cmp::PartialOrd
  + SyncSend
  + Copy
  + std::fmt::Debug
  + std::ops::AddAssign;

#[derive(Debug, Clone)]
pub struct ExpireMap<K: SyncSend, U: Num>(Arc<RwLock<HashMap<K, U>>>);

impl<K: SyncSend, U: Num> ExpireMap<K, U>
where
  u64: AsPrimitive<U>,
{
  pub fn add(&self, key: K) {
    self.0.write().insert(key, sec().as_());
  }

  pub fn renew(&self, key: &K) -> bool {
    if let Some(val) = self.0.write().get_mut(key) {
      *val = sec().as_();
      true
    } else {
      false
    }
  }

  pub fn has(&self, key: &K) -> bool {
    self.0.read().contains_key(key)
  }

  pub fn pop(&self, key: &K) -> bool {
    if self.0.read().contains_key(key) {
      self.0.write().remove(key);
      true
    } else {
      false
    }
  }

  pub fn new(timeout: U, max_interval: u64) -> (Self, async_std::task::JoinHandle<()>) {
    let mut interval: u64 = 1 + timeout.as_();
    let expire_map = Arc::new(RwLock::new(HashMap::new()));
    let map = expire_map.clone();
    let simple = 3;

    let timer = spawn(async move {
      loop {
        sleep(Duration::from_secs(interval)).await;
        let now: U = sec().as_();
        loop {
          let mut deleted: u8 = 0;
          let len: usize = map.read().len();
          let skip = if len > simple {
            thread_rng().gen_range(0..len - simple)
          } else {
            0
          };
          let _ = map
            .write()
            .drain_filter(|_, v| {
              if now.wrapping_sub(v) > timeout {
                deleted += 1;
                true
              } else {
                false
              }
            })
            .skip(skip)
            .take(simple);
          if deleted == 0 {
            if interval < max_interval {
              interval += 1;
            }
            break;
          };
          interval /= 2;
        }
      }
    });
    (ExpireMap(expire_map), timer)
  }
}
