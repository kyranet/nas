use std::process::Command;
use std::net::Ipv4Addr;
use regex::Regex;

#[cfg(windows)]
pub fn get_ipv4_address() -> Option<Ipv4Addr> {
    let output = Command::new("ipconfig")
        .output()
        .expect("failed to execute `ipconfig`");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r#"(\d{0,3}\.\d{0,3}\.\d{0,3}\.\d{0,3})"#).unwrap();
    for cap in re.captures_iter(&stdout) {
        if let Some(host) = cap.get(1) {
            let host = host.as_str();
            if host != "127.0.0.1" {
                if let Ok(addr) = host.parse::<Ipv4Addr>() {
                    return Some(addr)
                }
            }
        }
    }

    None
}

#[cfg(not(windows))]
pub fn get_ipv4_address() -> Option<Ipv4Addr> {
    let output = Command::new("ifconfig")
        .output()
        .expect("failed to execute `ifconfig`");

    let stdout = String::from_utf8_lossy(output.stdout);
    let re = Regex::new(r#"inet[: ](\d{0,3}\.\d{0,3}\.\d{0,3}\.\d{0,3})"#).unwrap();
    for cap in re.captures_iter(&stdout) {
        if let Some(host) = cap.get(1) {
            let host = host.as_str();
            if host != "127.0.0.1" {
                if let Ok(addr) = host.parse::<Ipv4Addr>() {
                    return Some(addr)
                }
            }
        }
    }

    None
}
