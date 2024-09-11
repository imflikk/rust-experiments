use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use clap::Parser;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;


// References:
// TCP Sockets: https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.connect_timeout
// CLI arg parsing: https://docs.rs/clap/latest/clap/
// Converting strings to SocketAddr: https://stackoverflow.com/questions/28255861/convert-string-to-socketaddr
// Copilot helped with usage of Rayon for parallel processing

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP/Host of the target to scan
    #[arg(short, long)]
    address: String,

    /// Port(s) to scan
    #[arg(short, long)]
    ports: String,
}

fn main() {
    let args = Args::parse();


    let address = args.address;
    let port = args.ports;

    // Create a custom thread pool with a fixed number of threads
    let pool = ThreadPoolBuilder::new().num_threads(10).build().unwrap();


    pool.scope(|s| {
        // check if port is an integer within 0-65535. If includes a dash, check if it's a range
        if port.contains("-") {
            let range: Vec<&str> = port.split("-").collect();
            let start = range[0].parse::<u16>().unwrap();
            let end = range[1].parse::<u16>().unwrap();

            if start > end {
                println!("[-] Invalid port range");
                return;
            }

            (start..=end).into_par_iter().for_each_with(s, |scope, p| {
                let value = address.clone();
                scope.spawn(move |_| {
                    connect_to_port(&value, p);
                });
            });
        } else {
            let port = port.parse::<u16>().unwrap();
            connect_to_port(&address, port);
        }
    })
    

    
}

fn connect_to_port(address: &str, port: u16) {
    let address_ip: IpAddr = address.parse().expect("Invalid IP address");
    let final_address = SocketAddr::new(address_ip, port);

    match TcpStream::connect_timeout(&final_address, Duration::from_secs(1)) {
        Ok(_) => println!("[+] Port {} is open", port),
        Err(_) =>  {}
    }
}
