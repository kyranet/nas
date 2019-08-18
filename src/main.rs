extern crate regex;

use std::env;
use std::io::{Error, ErrorKind};

mod server;
mod util;

fn main() -> std::io::Result<()> {
    if let Some(ip) = util::get_ipv4_address() {
        let port = match env::var("NAS_PORT") {
            Ok(p) => p,
            Err(_) => "8080".to_owned(),
        };
        Ok(server::start(ip, port))
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Failed to parse Ipv4"))
    }
}
