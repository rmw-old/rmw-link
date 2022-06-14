use array_init::array_init;
use std::{
  lazy::SyncLazy,
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use std::{thread, time::Duration};
use trust_dns_resolver::{
  config::{ResolverConfig, ResolverOpts},
  Resolver,
};

static DOH: SyncLazy<[Resolver; 2]> = SyncLazy::new(|| {
  let li = [
    ResolverConfig::from_parts(
      None,
      vec![],
      trust_dns_resolver::config::NameServerConfigGroup::from_ips_https(
        &[
          IpAddr::V4(Ipv4Addr::new(223, 6, 6, 6)),
          IpAddr::V4(Ipv4Addr::new(223, 5, 5, 5)),
          IpAddr::V6(Ipv6Addr::new(0x2400, 0x3200, 0, 0, 0, 0, 0, 0x0001)),
          IpAddr::V6(Ipv6Addr::new(0x2400, 0x3200, 0xbaba, 0, 0, 0, 0, 0x0001)),
        ],
        443,
        "dns.alidns.com".into(),
        true,
      ),
    ),
    ResolverConfig::from_parts(
      None,
      vec![],
      trust_dns_resolver::config::NameServerConfigGroup::from_ips_https(
        &[
          IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)),
          IpAddr::V4(Ipv4Addr::new(208, 67, 220, 220)),
          IpAddr::V6(Ipv6Addr::new(0x2620, 0x0119, 0x0035, 0, 0, 0, 0, 0x0035)),
          IpAddr::V6(Ipv6Addr::new(0x2620, 0x0119, 0x0053, 0, 0, 0, 0, 0x0053)),
        ],
        443,
        "doh.opendns.com".into(),
        true,
      ),
    ),
  ];
  array_init(|n| Resolver::new(li[n].clone(), ResolverOpts::default()).unwrap())
});

pub fn addr(host: &str) -> Vec<u8> {
  loop {
    for doh in DOH.iter() {
      if let Ok(response) = err::ok(doh.txt_lookup(host)) {
        for li in response {
          for txt in li.txt_data() {
            if let Ok(txt) = z85::decode(txt) {
              if !txt.is_empty() {
                return txt;
              }
            }
          }
        }
      }
    }
    log::error!("{} TXT", host);
    thread::sleep(Duration::from_secs(10));
  }
}
