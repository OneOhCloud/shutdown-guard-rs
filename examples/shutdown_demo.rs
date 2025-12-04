use shutdown_guard_rs::ShutdownGuard;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, thread};

fn main() {
    println!("Starting Shutdown Guard...");
    println!("Working directory: {:?}", env::current_dir().unwrap());

    let output_file = PathBuf::from("shutdown_time.txt");
    println!("Output file: {:?}", output_file);

    let guard = ShutdownGuard::new();
    let output_file_clone = output_file.clone();

    guard.register(Box::new(move || {
        let timestamp = chrono::Local::now();
        let message = format!(
            "Shutdown time: {}\nTimestamp: {}\n",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            timestamp.timestamp()
        );

        println!("\n{:=<50}", "");
        println!("Shutdown signal detected!");
        println!("{:=<50}", "");
        println!("{}", message);

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file_clone)
        {
            Ok(mut file) => {
                if let Err(e) = write!(file, "{}", message) {
                    eprintln!("Write failed: {}", e);
                } else {
                    if let Err(e) = file.sync_all() {
                        eprintln!("Sync failed: {}", e);
                    } else {
                        println!("Written and synced to: {:?}", output_file_clone);
                    }
                }
            }
            Err(e) => {
                eprintln!("Open failed: {}", e);
            }
        }

        println!("{:=<50}\n", "");
    }));

    match guard.start() {
        Ok(_) => println!("Shutdown monitoring started"),
        Err(e) => {
            eprintln!("Failed to start monitoring: {}", e);
            return;
        }
    }

    println!("\n{:=<50}", "");
    println!("Running...");
    println!("Shutdown will trigger callback");
    println!("Ctrl+C will not trigger callback");
    println!("{:=<50}\n", "");

    let mut counter = 0;
    loop {
        thread::sleep(Duration::from_secs(5));
        counter += 1;
        let now = chrono::Local::now();
        println!("[{}] Running... count: {}", now.format("%H:%M:%S"), counter);
    }
}
