//! Linux platform-specific implementation using D-Bus and systemd-logind

use crate::ShutdownCallback;
use dbus::blocking::Connection;
use dbus::Message;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

/// Starts monitoring for Linux shutdown events via systemd-logind
pub fn start_monitoring(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || {
        if let Err(e) = monitor_systemd_signals(callbacks) {
            eprintln!("Failed to monitor systemd signals: {}", e);
        }
    });

    Ok(())
}

fn monitor_systemd_signals(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the system bus
    let conn = Connection::new_system()?;

    // Add a match rule for PrepareForShutdown signal
    conn.add_match_no_cb(
        "type='signal',\
         interface='org.freedesktop.login1.Manager',\
         member='PrepareForShutdown'",
    )?;

    // Listen for incoming messages
    loop {
        // Process messages with a timeout
        if let Some(msg) = conn.process(Duration::from_millis(1000))? {
            if is_shutdown_signal(&msg) {
                // Execute all registered callbacks
                let callbacks_lock = callbacks.read();
                for callback in callbacks_lock.iter() {
                    callback();
                }
            }
        }
    }
}

fn is_shutdown_signal(msg: &Message) -> bool {
    msg.interface()
        .map(|i| i.as_cstr().to_str().unwrap_or("") == "org.freedesktop.login1.Manager")
        .unwrap_or(false)
        && msg
            .member()
            .map(|m| m.as_cstr().to_str().unwrap_or("") == "PrepareForShutdown")
            .unwrap_or(false)
}
