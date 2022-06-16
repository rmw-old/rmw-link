use std::lazy::SyncLazy;

pub const VERSION: u16 = 1;
pub const PING: [u8; 4] = [0, 0, 0, 0];
pub static EXPIRE: SyncLazy<u8> = SyncLazy::new(|| config::get!(net / timeout / ping, 21u8));
