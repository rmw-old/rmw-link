use addrbytes::ToBytes;
pub trait ToAddr = 'static
  + ToBytes
  + std::net::ToSocketAddrs
  + std::hash::Hash
  + std::marker::Send
  + std::fmt::Debug
  + std::fmt::Display
  + std::marker::Sync
  + Ord
  + Copy
  + Clone;
