use clap::Parser;
use reqwest;
use futures::stream::{StreamExt};
use tokio::task;
use colored::Colorize;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::error::Error;
use std::time::{Instant};

// Intended to be a clone of dirbuster/gobuster/feroxbuster as a way of practicing making web requests in rust
// TODO:
//  - Add filters for status code, size, and maybe something else
//  - Add option for custom headers (User agent, etc.)

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

// Create static slice of successful codes to filter for later
static SUCCESSFUL_HTTP_CODES: &[&str] = &["200", "301", "302"];
// Potential codes indicating the page might exist, but is blocked or forbidden
static POTENTIAL_HTTP_CODES: &[&str] = &["403"];


// Read lines from a wordlist file
async fn process_urls_concurrently(url: &str, wordlist: &str, thread_count: usize) -> Result<(), Box<dyn Error>> {
    if let Ok(lines) = read_lines(wordlist) {
        // Create a stream of tasks with a concurrency limit for our threads variable
        let tasks = futures::stream::iter(
            lines.flatten().map(|line| {
                let final_url = url.to_string() + &line;
                
                // Spawn async task for each URL
                async move {
                    make_request(&final_url).await;
                }
            })
        ).buffer_unordered(thread_count); // Limit concurrency to `concurrency_limit`

        // Execute all tasks and wait for them to complete
        tasks.for_each(|_| async {}).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    
    let args = Args::parse();

    let start = Instant::now();

    let url = args.url;
    let wordlist = args.wordlist;
    let threads = args.threads;
    let debug = args.debug;

    if debug {
        println!();
        println!("URL: {}", url);
        println!("Wordlist: {}", wordlist);
        println!("'Successful' Status codes: {:?}", SUCCESSFUL_HTTP_CODES);
        println!("Threads: {}", threads);
        println!("Debug Log: {}", debug);
        println!();
    }

    

    // Call the async function to process URLs
    let _ = process_urls_concurrently(&url, &wordlist, threads).await;

    // if let Ok(lines) = read_lines(wordlist) {
    //     // Consumes the iterator, returns an (Optional) String
    //     for line in lines.flatten() {
    //         let final_url = url.clone() + &line;
            
    //         make_request(&final_url).await;
    //     }
    // }


    let duration = start.elapsed();

    println!("Scan completed in {} seconds", duration.as_secs());

}

async fn make_request(url: &str) {
    //println!("URL: {url}");

    // HTTP requests reference: https://rust-lang-nursery.github.io/rust-cookbook/web/clients/requests.html
    let res = reqwest::get(url)
        .await
        .unwrap();

    
    // Extract the status and headers before consuming the response body
    let status = res.status();
    let headers = res.headers().clone();

    let length = &headers["content-length"].to_str().unwrap();

    if SUCCESSFUL_HTTP_CODES.contains(&(status.as_str())) {
        println!("{} - {} - {}", url, status.as_str().green(), length);
    } else if POTENTIAL_HTTP_CODES.contains(&(status.as_str())) {
        println!("{} - {} - {}", url, status.as_str().yellow(), length);
    }
    
    // read the body
    // if status.is_success() {
    //     let body = res.text()
    //         .await
    //         .unwrap(); // Read the body as text

    //     println!("Status: {}", status);
    //     //println!("Body:\n{}", body);
    // } else {
    //     println!("Failed to fetch the URL. Status: {}", status);
    // }
}


// Reference: https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
