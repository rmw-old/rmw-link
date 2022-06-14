use std::net::{SocketAddrV4, SocketAddrV6};

pub trait FromBytes<T> {
  fn from_bytes(bytes: impl AsRef<[u8]>) -> T;
}

impl FromBytes<SocketAddrV4> for SocketAddrV4 {
  fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
    let bytes = bytes.as_ref();
    SocketAddrV4::new(
      [bytes[0], bytes[1], bytes[2], bytes[3]].into(),
      u16::from_le_bytes([bytes[4], bytes[5]]),
    )
  }
}

impl FromBytes<SocketAddrV6> for SocketAddrV6 {
  fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
    let bytes = bytes.as_ref();
    SocketAddrV6::new(
      [
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
      ]
      .into(),
      u16::from_le_bytes([bytes[16], bytes[17]]),
      0,
      0,
    )
  }
}
