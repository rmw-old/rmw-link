pub const ID: [u8; 2] = {
  #[cfg(target_os = "windows")]
  #[cfg(target_arch = "x86")]
  {
    1u16
  }
  #[cfg(target_os = "windows")]
  #[cfg(target_arch = "x86_64")]
  {
    2u16
  }
  #[cfg(target_os = "windows")]
  #[cfg(target_arch = "aarch64")]
  {
    3u16
  }
  #[cfg(target_os = "macos")]
  #[cfg(target_arch = "x86_64")]
  {
    4u16
  }
  #[cfg(target_os = "macos")]
  #[cfg(target_arch = "aarch64")]
  {
    5u16
  }
  #[cfg(target_os = "ios")]
  #[cfg(target_arch = "aarch64")]
  {
    6u16
  }
  #[cfg(target_os = "android")]
  #[cfg(target_arch = "aarch64")]
  {
    7u16
  }
  #[cfg(target_os = "linux")]
  #[cfg(target_arch = "x86")]
  {
    8u16
  }
  #[cfg(target_os = "linux")]
  #[cfg(target_arch = "x86_64")]
  {
    9u16
  }
  #[cfg(target_os = "linux")]
  #[cfg(target_arch = "aarch64")]
  {
    10u16
  }
}
.to_le_bytes();
