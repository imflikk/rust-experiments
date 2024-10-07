use get_if_addrs::{get_if_addrs, IfAddr};
use std::error::Error;

pub fn get_local_ip_addresses() -> Result<Vec<String>, Box<dyn Error>> {

    match get_if_addrs() {
        Ok(interfaces) => {
            let mut ip_addresses = Vec::<String>::new();
            for iface in interfaces {
                // If interface is an IPv4 struct and not a loopback address, print it
                if let IfAddr::V4(_addr) = iface.clone().addr {
                    if !iface.is_loopback() {
                        //println!("{:?}", iface);
                        ip_addresses.push(iface.ip().to_string());
                    }
                }
                
            }
            Ok(ip_addresses)
        }
        Err(e) => Err(e)?,
    }
}