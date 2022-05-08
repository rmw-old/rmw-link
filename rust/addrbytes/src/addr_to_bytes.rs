use std::net::{SocketAddrV4, SocketAddrV6};

//use bytes::{BufMut, Bytes, BytesMut};

pub trait ToBytes {
  fn to_bytes(&self) -> Box<[u8]>;
}

impl ToBytes for SocketAddrV4 {
  fn to_bytes(&self) -> Box<[u8]> {
    let o = self.ip().octets();
    let p = self.port().to_le_bytes();
    [o[0], o[1], o[2], o[3], p[0], p[1]].into()
  }
}

impl ToBytes for SocketAddrV6 {
  fn to_bytes(&self) -> Box<[u8]> {
    let o = self.ip().octets();
    let p = self.port().to_le_bytes();
    [
      o[0], o[1], o[2], o[3], o[4], o[5], o[6], o[7], o[8], o[9], o[10], o[11], o[12], o[13],
      o[14], o[15], p[0], p[1],
    ]
    .into()
  }
}
