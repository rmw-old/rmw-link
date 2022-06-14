use duckdb::{Connection, Result};
use std::{convert::AsRef, fs, path::Path};

#[derive(Clone, Debug)]
pub struct Db {
  pub conn: Connection,
}

pub fn open(path: impl AsRef<Path>) -> Result<Db> {
  let path = path.as_ref();
  if let Some(root) = path.parent() {
    fs::create_dir_all(root).unwrap();
  }
  Ok(Db {
    conn: Connection::open(path)?,
  })
}

/*
节点 权重
主动连接成功一次 ， 权重+1

node
  ip
  pk
  rank
*/
