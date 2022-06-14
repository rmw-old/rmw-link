use num_enum::TryFromPrimitive;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Cmd {
  Ping,
  FindNode,
  FindedNode,
  //Heartbeat,
  //DecryptionFail,
}
