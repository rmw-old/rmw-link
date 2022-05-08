#![feature(backtrace)]
#![feature(backtrace_frames)]

use std::io::Write;

pub fn trace() {
  #[cfg(debug_assertions)]
  {
    use std::backtrace::Backtrace;
    for f in Backtrace::capture().frames() {
      let s = format!("{:?}", f);
      if s.contains("\"./") {
        println!("{}", s);
      }

      // TODO
    }
  }
}

pub fn ok<T, Err: std::fmt::Display>(result: Result<T, Err>) -> Result<T, Err> {
  match result {
    Err(err) => {
      trace();
      let _ = std::io::stderr().write(err.to_string().as_ref());
      Err(err)
    }
    Ok(val) => Ok(val),
  }
}

pub fn log<T>(result: Result<T, impl std::fmt::Display>) {
  if let Err(err) = result {
    trace();
    let _ = std::io::stderr().write(err.to_string().as_ref());
  }
}

/*
#[macro_export]
macro_rules! errtip {
($var:expr, $tip:ident) => {
match $var {
Ok(r) => Ok(r),
Err(err) => {
log::error!("{:?} {:?}", &$tip, err);
Err(err)
}
}
};
}
*/
