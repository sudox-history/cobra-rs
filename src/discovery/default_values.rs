use std::time::Duration;
use std::net::Ipv4Addr;

pub const DEFAULT_ADDRESS: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
pub const DEFAULT_MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
pub const DEFAULT_PORT: u16 = 55669;

pub const DEFAULT_SEARCH_PACKAGE: [u8; 5] = [8, 100, 193, 210, 19];
pub const DEFAULT_ANSWER_PACKAGE: [u8; 5] = [65, 238, 212, 64, 80];

pub const DEFAULT_POOLING_RATE: Duration = Duration::from_secs(5);
