#[derive(Debug)]
pub struct Id {
  pub time: u64,
  pub id: u8,
}

// unsafe impl Sync for Id {}
// unsafe impl Send for Id {}

impl Default for Id {
  fn default() -> Self {
    Self { time: 0, id: 255 }
  }
}

impl Id {
  pub fn get(&mut self) -> [u8; 8] {
    if self.id == 255 {
      self.id = 0;
      let now = time::ms() as _;
      let time = self.time;
      self.time = if time == now {
        time + (1 << 8)
      } else {
        now << 8
      };
    } else {
      self.id += 1;
    }
    (self.time + (self.id as u64)).to_be_bytes()
  }
}
