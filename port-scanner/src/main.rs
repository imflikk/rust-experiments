use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use clap::Parser;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};


// References:
// TCP Sockets: https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.connect_timeout
// CLI arg parsing: https://docs.rs/clap/latest/clap/
// Converting strings to SocketAddr: https://stackoverflow.com/questions/28255861/convert-string-to-socketaddr
// Copilot helped with usage of Rayon for parallel processing


// TODO
// - Read more of the rust book because I don't know how the fuck some of this works even after reading various posts
// - Add parsing for comma separated ports
// - Add better output formats
// - Add error handling for invalid IP addresses

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP/Host of the target to scan
    #[arg(short, long)]
    address: String,

    /// Port(s) to scan
    #[arg(short, long)]
    ports: String,

    /// Number of threads to use
    #[arg(short, long, default_value = "10")]
    threads: usize,
}

fn main() {
    let args = Args::parse();

    let start = Instant::now();


    let address = args.address;
    let port = args.ports;
    let threads = args.threads;

    // Create a custom thread pool with a fixed number of threads
    let pool = ThreadPoolBuilder::new().num_threads(threads).build().unwrap();

    // I know this is to safely access the open_ports vector, but I'm not sure how it works
    let open_ports = Arc::new(Mutex::new(Vec::new()));


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
                let address_clone = address.clone();
                let open_ports_clone = Arc::clone(&open_ports);
                scope.spawn(move |_| {
                    if connect_to_port(&address_clone, p) {
                        let mut open_ports = open_ports_clone.lock().unwrap();
                        open_ports.push(p);
                    }
                });
            });
        } else {
            let port = port.parse::<u16>().unwrap();
            connect_to_port(&address, port);
        }
    });

    // Print out the open ports
    let mut open_ports = open_ports.lock().unwrap();
    if open_ports.is_empty() {
        println!("No open ports found");
    } else {
        open_ports.sort();
        println!("Open ports: {:?}", open_ports);
    }

    let duration = start.elapsed();

    println!("Scan completed in {} seconds", duration.as_secs());
    

    
}

fn connect_to_port(address: &str, port: u16) -> bool {
    let address_ip: IpAddr = address.parse().expect("Invalid IP address");
    let final_address = SocketAddr::new(address_ip, port);

    match TcpStream::connect_timeout(&final_address, Duration::from_secs(1)) {
        Ok(_) => {
            println!("[+] Port {} is open", port);
            true
        }
        Err(_) => false,
    }
}
