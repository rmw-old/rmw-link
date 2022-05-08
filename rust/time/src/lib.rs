use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn now() -> Duration {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn ms() -> u128 {
  now().as_millis()
}

pub fn sec() -> u64 {
  now().as_secs()
}

pub fn sec_to_bytes() -> [u8; 8] {
  sec().to_le_bytes()
}
