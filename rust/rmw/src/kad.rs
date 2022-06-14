use crate::typedef::ToAddr;
use array_init::array_init;
use smallvec::SmallVec;
use std::{
  cmp::min,
  collections::{BTreeMap, BTreeSet},
  io::Write,
  ops::RangeInclusive,
};

pub const LEN: usize = 128;
pub const CAPACITY: usize = 32;
const NODE_LEN: usize = LEN - 5; // 1+2+4+8+16 = 31 , 相同的key不可能出现

#[derive(Debug, Clone, PartialOrd, Ord, Eq)]
pub struct Node<Addr: ToAddr> {
  pub depth: u8,
  pub addr: Addr,
}

impl<Addr: ToAddr> PartialEq for Node<Addr> {
  fn eq(&self, other: &Self) -> bool {
    self.addr == other.addr
  }
}

macro_rules! bin {
  ($li:expr) => {{
    let mut bin = vec![];
    for i in $li.iter() {
      bin.write_all(&i.addr.to_bytes()).unwrap();
    }
    bin.into()
  }};
}

#[derive(Debug)]
pub struct Kad<Addr: ToAddr> {
  pub cache: SmallVec<[Box<[u8]>; NODE_LEN]>,
  pub key: u128,
  pub node: SmallVec<[BTreeSet<Node<Addr>>; NODE_LEN]>,
  pub addr: BTreeMap<Addr, u8>,
}

impl<Addr: ToAddr> Kad<Addr> {
  pub fn find(&self, key: &[u8]) -> Box<[u8]> {
    let depth = (u128::from_be_bytes(key[..16].try_into().unwrap()) & self.key) as usize;
    self.cache[min(depth, self.node.len() - 1)].clone()
  }

  pub fn new(key: &[u8]) -> Self {
    Self {
      key: u128::from_be_bytes(key[..16].try_into().unwrap()),
      node: SmallVec::from_elem(BTreeSet::new(), 1),
      cache: SmallVec::from_elem(unsafe { Box::<[u8]>::new_uninit_slice(0).assume_init() }, 1),
      addr: BTreeMap::new(),
    }
  }

  pub fn add(&mut self, pk: &[u8], addr: Addr) -> bool {
    macro_rules! cache_update {
      ($pos:expr,$li:expr) => {{
        self.cache[$pos] = bin!($li);
      }};
    }

    let pk = u128::from_be_bytes(pk[..16].try_into().unwrap());
    let depth = (self.key ^ pk).leading_zeros() as u8;
    let node = &mut self.node;
    let max = node.len() - 1;
    if let Some(&d) = self.addr.get(&addr) {
      if d == depth {
        return false;
      }
      node[min(d as usize, max)].remove(&Node { addr, depth: d });
    }

    let max_u8 = max as u8;
    let insert;
    if depth >= max_u8 {
      let pos = max as usize;
      let li = &mut node[pos];
      if li.len() == CAPACITY {
        let mut new: BTreeSet<Node<Addr>> = li.drain_filter(|i| i.depth > max_u8).collect();
        //dbg!((max, self.cache.len() - 1, &li, &new));
        cache_update!(pos, li);
        let result = new.insert(Node { depth, addr });
        if result {
          self.addr.insert(addr, depth);
        }
        self.cache.push(bin!(new));
        node.push(new);
        return result;
      }
      insert = max;
    } else {
      insert = depth as usize;
    }

    let li = &mut node[insert];
    if li.len() < CAPACITY && li.insert(Node { depth, addr }) {
      cache_update!(insert, li);
      self.addr.insert(addr, depth);
      return true;
    }

    false
  }

  pub fn is_empty(&self) -> bool {
    self.node.len() == 1 && self.node[0].len() == 0
  }

  pub fn compact(&mut self) {
    let node = &mut self.node;
    let mut max = node.len() - 1;
    loop {
      if max > 0 && (node[max].len() + node[max - 1].len()) <= CAPACITY {
        let map = node.pop().unwrap();
        max -= 1;
        let li = &mut node[max];
        li.extend(map);
        self.cache.pop();
        self.cache[max] = bin!(li);
        continue;
      }
      break;
    }
  }

  pub fn range(&self) -> [RangeInclusive<u128>; LEN] {
    let key = self.key;
    let mut lower = key;
    let mut upper = key;
    let mut one = 0;

    let mut li = array_init(|n| {
      let n = n + 1;
      let lower_next;
      let upper_next;
      if n == LEN {
        lower_next = 0;
        upper_next = u128::MAX;
      } else {
        one = (one << 1) + 1;
        lower_next = (lower >> n) << n;
        upper_next = lower_next + one;
      };

      let r;
      if lower_next == lower {
        r = (upper + 1)..=upper_next;
        upper = upper_next;
      } else {
        r = lower_next..=(lower - 1);
        lower = lower_next;
      }
      r
    });

    li.reverse();
    li
  }
}
