use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use reqwest::blocking::get;
use tee::TeeReader;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <url>", args[0]);
        return;
    }

    let url = &args[1];
    let response = get(url).unwrap();

    let file_name = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("chase.bin");

    let file = File::create(file_name).unwrap();

    let size = response.content_length().unwrap_or(0);
    println!(
        "Downloading: {:.2} MB (Total: {:.2} MB)",
        size as f64 / 1_000_000.0,
        size
    );

    let received_bytes = Arc::new(Mutex::new(0));
    let received_bytes_clone = received_bytes.clone();

    let mut tee_reader = TeeReader::new(response, file);
    let mut buffer = [0; 1024];

    let thread = thread::spawn(move || {
        loop {
            let size = tee_reader.read(&mut buffer).unwrap();
            if size == 0 {
                break;
            }
            let mut downloaded = received_bytes.lock().unwrap();
            *downloaded += size as u64;
        }
    });

    loop {
        let received = received_bytes_clone.lock().unwrap();

        println!(
            "Progress: {:.2}% ({:.2}/{:.2} MB)",
            (*received as f64 / size as f64) * 100.0,
            *received as f64 / 1_000_000.0,
            size as f64 / 1_000_000.0,
        );

        if *received == size {
            break;        
        }

        thread::sleep(Duration::from_secs(1));
    }

    thread.join().unwrap();
    println!("\nDownload completed!");
}