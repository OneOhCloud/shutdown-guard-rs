//! Advanced example with file cleanup
//!
//! Run with: cargo run --example file_cleanup

use shutdown_guard_rs::ShutdownGuard;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting file cleanup example...");

    let guard = ShutdownGuard::new();

    // Create a temporary log file
    let log_path = PathBuf::from("./temp_app.log");
    let counter = Arc::new(AtomicU64::new(0));

    // Clone for the cleanup callback
    let log_path_cleanup = log_path.clone();
    let counter_cleanup = Arc::clone(&counter);

    // Register cleanup callback
    guard.register(Box::new(move || {
        println!("🧹 Shutdown detected! Performing file cleanup...");

        // Write final statistics to log
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_path_cleanup) {
            let count = counter_cleanup.load(Ordering::SeqCst);
            writeln!(file, "\n=== SHUTDOWN ===").ok();
            writeln!(file, "Total operations: {}", count).ok();
            writeln!(file, "Timestamp: {}", chrono::Local::now()).ok();
            println!("✅ Final statistics written to log");
        }

        // Give time for I/O to complete
        thread::sleep(Duration::from_millis(500));

        println!("✅ File cleanup completed!");
    }));

    // Start monitoring
    guard.start().expect("Failed to start monitoring");
    println!("✅ Shutdown monitoring active\n");

    // Simulate application work
    let mut file = File::create(&log_path).expect("Failed to create log file");
    writeln!(file, "Application started at {}", chrono::Local::now()).ok();
    println!("📝 Log file created: {:?}\n", log_path);

    println!(
        "📢 Application is logging operations. Shutdown the system to see cleanup in action.\n"
    );

    // Simulate periodic work
    loop {
        thread::sleep(Duration::from_secs(5));
        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;

        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_path) {
            writeln!(file, "Operation {} at {}", count, chrono::Local::now()).ok();
        }

        println!("📝 Logged operation #{}", count);
    }
}
