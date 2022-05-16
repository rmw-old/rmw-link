#![feature(btree_drain_filter)]
#![feature(trait_alias)]

use async_std::task::{sleep, spawn};
use num::traits::AsPrimitive;
use parking_lot::RwLock;
use rand::{thread_rng, Rng};
use std::marker::Copy;
use std::{collections::BTreeMap, hash::Hash, sync::Arc, time::Duration};
use time::sec;
//use tokio::{spawn, time::sleep};
pub trait SyncSend = 'static + Sync + Send + Copy;
pub trait AsU64 = AsPrimitive<u64> + Sync + Send;

#[derive(Debug, Clone)]
pub struct ExpireMap<K: SyncSend + Eq + std::cmp::Ord, V: SyncSend, U: AsU64 = u16>(
  Arc<RwLock<BTreeMap<K, (V, U)>>>,
);

impl<K: SyncSend + Eq + Hash + std::cmp::Ord, V: SyncSend, U: AsU64> ExpireMap<K, V, U>
where
  u64: AsPrimitive<U>,
{
  pub fn add(&self, key: K, val: V) {
    self.0.write().insert(key, (val, sec().as_()));
  }

  pub fn renew(&self, key: &K) -> Option<V> {
    if let Some(val) = self.0.write().get_mut(key) {
      (*val).1 = sec().as_();
      Some(val.0)
    } else {
      None
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
    let timeout: u64 = timeout.as_();
    let mut interval: u64 = 1 + timeout;
    let expire_map = Arc::new(RwLock::new(BTreeMap::<_, (V, U)>::new()));
    let map = expire_map.clone();
    let simple = 3;
    let timer = spawn(async move {
      loop {
        sleep(Duration::from_secs(interval)).await;
        let now: u64 = sec();
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
              if now.wrapping_sub(v.1.as_()) > timeout {
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
