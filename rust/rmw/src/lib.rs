#![feature(map_first_last)]
#![feature(unboxed_closures)]
#![feature(new_uninit)]
#![feature(btree_drain_filter)]
#![feature(int_log)]
#![feature(trait_alias)]
#![feature(once_cell)]

mod doh;
mod kad;
mod kad_net;
mod key;
mod pool;
mod recv;
mod reply;
mod typedef;
mod util;
mod var;

pub mod rmw;
