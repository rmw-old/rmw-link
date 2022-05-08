use duckdb::{Config, Connection, Result};
use std::{convert::AsRef, fs, path::Path};

pub struct Db {
  pub conn: Connection,
}

pub fn open<P: AsRef<Path>>(path: P) -> Result<Db> {
  let path = path.as_ref();
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  Ok(Db {
    conn: Connection::open_with_flags(
      path,
      Config::default().threads(config::get!(db / threads, num_cpus::get() as _))?,
    )?,
  })
}
