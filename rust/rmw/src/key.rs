use ed25519_dalek_blake3::{Keypair, SecretKey};
use log::info;
use twox_hash::xxh3::hash128;

fn keypair(seed: &[u8]) -> Keypair {
  let secret = SecretKey::from_bytes(seed).unwrap();
  let public = (&secret).into();
  Keypair { public, secret }
}

pub fn hash128_bytes(data: &[u8]) -> [u8; 16] {
  hash128(data).to_le_bytes()
}

#[macro_export]
macro_rules! hash128_bytes {
  ($sk_hash:expr,$($i:expr),*) => {
    {
      use $crate::key::{hash128_bytes};
      hash128_bytes(&[&$sk_hash[..], $($i),*].concat())
    }
  };
}

pub fn new(kv: &kv::Kv) -> Keypair {
  keypair(&kv.get_or_create("sk", || {
    info!("首次运行，生成秘钥中，请稍等几分钟 ···");
    let seed: Box<[u8]> = keygen::seed_new().into();
    info!("秘钥完成初始化");
    seed
  }))
}
/*
#[derive(Clone)]
pub struct Key {
  pub ed25519: Keypair,
  pub ed25519_pk: [u8; 30],
  pub ed25519_sk_hash: [u8; 16],
  pub x25519_sk: x25519_dalek::StaticSecret,
  pub x25519_pk: x25519_dalek::PublicKey,
}

impl Key {
  pub fn new() -> Self {
    let ed25519 = keypair(&config::get!(sk, {
      println!("首次运行，生成秘钥中，请稍等几分钟 ···");
      let seed: Box<[u8]> = keygen::seed_new().into();
      seed
    }));
    Self {
      ed25519,
      ed25519_pk: ed25519.public.as_bytes()[..keygen::PK_LEN]
        .try_into()
        .unwrap(),
      ed25519_sk_hash: hash128(ed25519.secretas_bytes()).to_le_bytes(),
      x25519_sk: (&ed25519.secret).into(),
      x25519_pk: (&ed25519.public).into(),
    }
  }
}


#[macro_export]
macro_rules! sk_addr_hash {
  ($addr:expr,$($i:expr),*) => {
    crate::sk_hash!(&$addr.to_bytes(),$($i),*)
  }
}
*/
