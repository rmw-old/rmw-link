use ed25519_dalek_blake3::{Keypair, SecretKey};
use twox_hash::xxh3::hash128;

fn keypair(seed: &[u8]) -> Keypair {
  let secret = SecretKey::from_bytes(seed).unwrap();
  let public = (&secret).into();
  Keypair { public, secret }
}

pub fn hash128_bytes(data: &[u8]) -> [u8; 16] {
  hash128(data).to_le_bytes()
}

pub fn new() -> Keypair {
  keypair(&config::get!(sk, {
    println!("首次运行，生成秘钥中，请稍等几分钟 ···");
    let seed: Box<[u8]> = keygen::seed_new().into();
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
macro_rules! sk_hash {
  ($($i:expr),*) => {
    {
      use crate::crypto::{hash128_bytes,SK_HASH};
      &hash128_bytes(&[&SK_HASH[..], $($i),*].concat())[..]
    }
  };
}

#[macro_export]
macro_rules! sk_addr_hash {
  ($addr:expr,$($i:expr),*) => {
    crate::sk_hash!(&$addr.to_bytes(),$($i),*)
  }
}
*/
