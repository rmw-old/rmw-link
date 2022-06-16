use crate::util::udp::Input;
use std::net::ToSocketAddrs;

pub fn ping<Addr: ToSocketAddrs>(input: Input<Addr>) {}
