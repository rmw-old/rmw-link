use anyhow::{Error, Result};
use async_std::task::block_on;

fn main() -> Result<()> {
  logger::init()
    .level_for("rmw", log::LevelFilter::Trace)
    .apply()?;
  block_on(async {
    let rmw = rmwlib::rmw::v4()?;
    rmw.run().await?;
    Ok::<_, Error>(())
  })?;
  Ok(())
}
