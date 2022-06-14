#![feature(map_first_last)]
#![feature(trait_alias)]

use async_std::task::{sleep, spawn};
use num::traits::AsPrimitive;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::{
  collections::BTreeMap,
  hash::Hash,
  ops::Bound::{Excluded, Included},
  sync::Arc,
  time::Duration,
};
use std::{fmt::Debug, marker::Copy};
use time::sec;
//use tokio::{spawn, time::sleep};
pub trait SyncSend = 'static + Sync + Send + Copy;
pub trait AsU64 = AsPrimitive<u64> + Sync + Send;

#[derive(Debug, Clone)]
pub struct ExpireMap<K: SyncSend + Eq + std::cmp::Ord + Debug, V: SyncSend, U: AsU64 = u16>(
  Arc<RwLock<BTreeMap<K, (V, U)>>>,
);

const SIMPLE: usize = 2;

impl<K: SyncSend + Eq + Hash + std::cmp::Ord + Debug, V: SyncSend, U: AsU64> ExpireMap<K, V, U>
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
    let map = &self.0;
    if map.read().contains_key(key) {
      map.write().remove(key);
      true
    } else {
      false
    }
  }

  pub fn new(
    timeout: U,
    on_expire: impl Fn(&K) + Send + 'static,
  ) -> (Self, async_std::task::JoinHandle<()>) {
    let timeout: u64 = timeout.as_();
    let interval: u64 = 1 + timeout;
    let expire_map = Arc::new(RwLock::new(BTreeMap::<_, (V, U)>::new()));
    let map = expire_map.clone();
    let timer = spawn(async move {
      let mut cursor: Option<K> = None;
      const CAPACITY: usize = 8 * SIMPLE;
      loop {
        sleep(Duration::from_secs(interval)).await;
        let now: u64 = sec();

        let end = if let Some(kv) = map.read().last_key_value() {
          *kv.0
        } else {
          continue;
        };

        let mut to_remove = SmallVec::<[_; CAPACITY]>::new();
        {
          let mut m = map.write();
          let mut n = 0;
          let mut stop = SIMPLE;
          for (&k, &(_, v)) in m.range((
            if let Some(p) = cursor {
              Excluded(p)
            } else {
              Included(*m.first_key_value().unwrap().0)
            },
            Included(end),
          )) {
            n += 1;
            if now.wrapping_sub(v.as_()) > timeout {
              to_remove.push(k);
              stop = stop.wrapping_add(CAPACITY);
            }
            if n >= stop {
              cursor = Some(k);
              break;
            }
          }
          if n < SIMPLE {
            cursor = None;
          }
          for i in &to_remove {
            m.remove(i);
          }
        }
        if to_remove.is_empty() {
          break;
        } else {
          to_remove.iter().for_each(&on_expire);
        }
      }
    });
    (ExpireMap(expire_map), timer)
  }
}
