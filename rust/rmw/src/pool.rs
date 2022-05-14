use std::lazy::SyncLazy;

pub static POOL: SyncLazy<rayon::ThreadPool> =
  SyncLazy::new(|| rayon::ThreadPoolBuilder::new().build().unwrap());

pub fn spawn<OP>(op: OP)
where
  OP: 'static + FnOnce() + Send,
{
  POOL.spawn(op)
}
