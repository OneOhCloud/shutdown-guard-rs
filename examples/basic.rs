//! Basic example demonstrating how to use shutdown-guard
//!
//! Run with: cargo run --example basic

use shutdown_guard_rs::ShutdownGuard;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting shutdown guard example...");

    // Create a new shutdown guard
    let guard = ShutdownGuard::new();

    // Register a cleanup callback
    guard.register(Box::new(|| {
        println!("🧹 Cleanup callback 1: Saving application state...");
        thread::sleep(Duration::from_secs(1));
        println!("✅ Application state saved!");
    }));

    // Register another callback
    guard.register(Box::new(|| {
        println!("🧹 Cleanup callback 2: Closing database connections...");
        thread::sleep(Duration::from_secs(1));
        println!("✅ Database connections closed!");
    }));

    // Register a third callback
    guard.register(Box::new(|| {
        println!("🧹 Cleanup callback 3: Flushing logs...");
        thread::sleep(Duration::from_secs(1));
        println!("✅ Logs flushed!");
    }));

    println!("Registered {} cleanup callbacks", guard.callback_count());

    // Start monitoring for shutdown events
    match guard.start() {
        Ok(_) => println!("✅ Shutdown monitoring started successfully!"),
        Err(e) => {
            eprintln!("❌ Failed to start shutdown monitoring: {}", e);
            return;
        }
    }

    println!("\n📢 Application is running. Try shutting down your system to see the cleanup callbacks in action.");
    println!("📢 Press Ctrl+C to exit (note: Ctrl+C won't trigger shutdown callbacks, only system shutdown will).\n");

    // Keep the application running
    loop {
        thread::sleep(Duration::from_secs(10));
        println!(
            "⏰ Application still running... ({})",
            chrono::Local::now().format("%H:%M:%S")
        );
    }
}
