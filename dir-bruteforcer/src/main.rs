use clap::Parser;
use reqwest;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// Intended to be a clone of dirbuster/gobuster/feroxbuster as a way of practicing making web requests in rust

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP/URL of the target to scan
    #[arg(short, long)]
    url: String,

    /// Wordlist to use for bruteforcing
    #[arg(short, long)]
    wordlist: String,

    /// Number of threads to use
    #[arg(short, long, default_value = "10")]
    threads: usize,

    /// Enable Debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    
    let args = Args::parse();

    let url = args.url;
    let wordlist = args.wordlist;
    let threads = args.threads;
    let debug = args.debug;

    if debug {
        println!();
        println!("URL: {}", url);
        println!("Wordlist: {}", wordlist);
        println!("Threads: {}", threads);
        println!("Debug Log: {}", debug);
        println!();
    }

    
    if let Ok(lines) = read_lines(wordlist) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            let final_url = url.clone() + &line;
            
            make_request(&final_url).await;
        }
    }

}

async fn make_request(url: &str) {
    println!("URL: {url}");

    // HTTP requests reference: https://rust-lang-nursery.github.io/rust-cookbook/web/clients/requests.html
    let res = reqwest::get(url).await.unwrap();

    // Extract the status and headers before consuming the response body
    let status = res.status();
    // let headers = res.headers().clone();
    
    // Now read the body
    if status.is_success() {
        let body = res.text().await.unwrap(); // Read the body as text
        println!("Status: {}", status);
        //println!("Body:\n{}", body);
    } else {
        println!("Failed to fetch the URL. Status: {}", status);
    }
}


// Reference: https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
