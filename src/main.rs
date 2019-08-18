extern crate regex;

mod util;

fn main() {
    if let Some(ip) = util::get_ipv4_address() {
        println!("Ipv4 is: {}", ip);
    } else {
        println!("Failed to parse Ipv4");
    }
}
