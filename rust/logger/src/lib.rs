use colored::Colorize;
use log::Level::{Error, Warn};

pub fn init() -> fern::Dispatch {
  fern::Dispatch::new()
    .format(|out, message, record| {
      let line = record.line().unwrap_or(0);
      let tip = (format_args!(
        "{} {} {} {}{}",
        record.level(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        record.target(),
        record.file().unwrap_or(""),
        if line > 0 {
          format!(":{}", line)
        } else {
          "".to_string()
        }
      ))
      .to_string();

      out.finish(format_args!(
        "{}\n{}\n",
        match record.level() {
          Error => tip.red(),
          Warn => tip.yellow(),
          _ => tip.bright_black(),
        },
        message,
      ))
    })
    .level(log::LevelFilter::Info)
    .chain(std::io::stdout())
  // .chain(fern::log_file("output.log")?)
  //.apply()
  //.unwrap()
}
