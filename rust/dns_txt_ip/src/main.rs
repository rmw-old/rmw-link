use addrbytes::ToBytes;
use anyhow::Result;
use std::net::{SocketAddrV4, SocketAddrV6};
use z85::encode;

fn main() -> Result<()> {
  let v4: SocketAddrV4 = "54.177.127.37:4910".parse()?;
  let v4 = encode(v4.to_bytes());

  let v6: SocketAddrV6 = "[2600:1f1c:626:9200:7b9a:4420:876a:4550]:4910".parse()?;
  let v6 = encode(v6.to_bytes());

  dbg!(&v4.len());
  dbg!(&v6.len());
  dbg!(v4);
  dbg!(v6);

  Ok(())
}
