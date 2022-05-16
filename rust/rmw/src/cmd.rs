use num_enum::TryFromPrimitive;
use std::fmt::Debug;

#[derive(PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Cmd {
  Ping,
  DecryptionFail,
}
