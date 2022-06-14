#[macro_export]
macro_rules! midpoint {
  ($a:expr,$b:expr) => {{
    ($a >> 1).wrapping_add($b >> 1).wrapping_add($a & $b & 0x1)
  }};
}
