use lazy_static::lazy_static;
use log::error;
use rmw_str::Str;
use std::{fs, path::PathBuf};

lazy_static! {
  pub static ref ROOT: PathBuf = dir::ROOT.clone().join("conf");
}

pub fn get<T: Str>(file: &str, init: fn() -> T) -> T {
  let path = ROOT.clone().join(file);
  let _init = || {
    let r = init();
    let mut dir = path.clone();
    dir.pop();
    fs::create_dir_all(dir).unwrap();
    fs::write(&path, &r.encode()).unwrap();
    r
  };

  match fs::read(&path) {
    Ok(buf) => {
      match T::decode(&buf) {
        Ok(r) => {
          //if buf != txt {
          //  fs::write(&path, &buf).unwrap();
          //}
          r
        }
        Err(err) => {
          error!("{}", err);
          _init()
        }
      }
    }
    Err(_) => _init(),
  }
}

#[macro_export]
macro_rules! get {
  ($file:expr, $init:expr) => {
    config::get(const_str::replace!(stringify!($file), " ", ""), || $init)
  };
}
