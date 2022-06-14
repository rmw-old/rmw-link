use std::time::Duration;

pub async fn sleep(n: u64) {
  async_std::task::sleep(Duration::from_secs(n)).await;
}
