use std::lazy::SyncLazy;

pub static POOL: SyncLazy<rayon::ThreadPool> =
  SyncLazy::new(|| rayon::ThreadPoolBuilder::new().build().unwrap());
