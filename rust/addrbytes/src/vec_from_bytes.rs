use std::net::{SocketAddrV4, SocketAddrV6};

pub trait VecFromBytes<T> {
  fn vec_from_bytes(li: impl AsRef<[u8]>) -> Vec<T>;
}

impl VecFromBytes<SocketAddrV4> for SocketAddrV4 {
  fn vec_from_bytes(li: impl AsRef<[u8]>) -> Vec<Self> {
    let mut r = vec![];
    let li = li.as_ref();
    let len = li.len();
    let mut n = 0;
    loop {
      n += 6;
      if n > len {
        break;
      }
      r.push(SocketAddrV4::new(
        [li[n - 6], li[n - 5], li[n - 4], li[n - 3]].into(),
        u16::from_le_bytes([li[n - 2], li[n - 1]]),
      ))
    }
    r
  }
}

impl VecFromBytes<SocketAddrV6> for SocketAddrV6 {
  fn vec_from_bytes(li: impl AsRef<[u8]>) -> Vec<Self> {
    let mut r = vec![];
    let li = li.as_ref();
    let len = li.len();
    let mut n = 0;
    loop {
      n += 18;
      if n > len {
        break;
      }
      r.push(SocketAddrV6::new(
        [
          li[n - 18],
          li[n - 17],
          li[n - 16],
          li[n - 15],
          li[n - 14],
          li[n - 13],
          li[n - 12],
          li[n - 11],
          li[n - 10],
          li[n - 9],
          li[n - 8],
          li[n - 7],
          li[n - 6],
          li[n - 5],
          li[n - 4],
          li[n - 3],
        ]
        .into(),
        u16::from_le_bytes([li[n - 2], li[n - 1]]),
        0,
        0,
      ))
    }
    r
  }
}
