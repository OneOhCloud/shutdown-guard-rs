//! Linux platform-specific implementation using signal handlers or D-Bus

use crate::ShutdownCallback;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(feature = "dbus-support")]
use dbus::blocking::Connection;
#[cfg(feature = "dbus-support")]
use dbus::Message;
#[cfg(feature = "dbus-support")]
use std::time::Duration;

#[cfg(not(feature = "dbus-support"))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(feature = "dbus-support"))]
static mut GLOBAL_CALLBACKS: Option<Arc<RwLock<Vec<ShutdownCallback>>>> = None;
#[cfg(not(feature = "dbus-support"))]
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Starts monitoring for Linux shutdown events
pub fn start_monitoring(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dbus-support")]
    {
        std::thread::spawn(move || {
            if let Err(e) = monitor_systemd_signals(callbacks) {
                eprintln!("Failed to monitor systemd signals: {}", e);
            }
        });
    }

    #[cfg(not(feature = "dbus-support"))]
    {
        // Fallback to signal handlers when dbus is not available
        unsafe {
            GLOBAL_CALLBACKS = Some(callbacks);
            register_signal_handlers()?;
        }
        println!("Linux shutdown monitoring active (using signal handlers)");
    }

    Ok(())
}

#[cfg(feature = "dbus-support")]
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

#[cfg(feature = "dbus-support")]
fn is_shutdown_signal(msg: &Message) -> bool {
    msg.interface()
        .map(|i| i.as_ref() == "org.freedesktop.login1.Manager")
        .unwrap_or(false)
        && msg
            .member()
            .map(|m| m.as_ref() == "PrepareForShutdown")
            .unwrap_or(false)
}

#[cfg(not(feature = "dbus-support"))]
unsafe fn register_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
    use std::mem;

    // Register SIGTERM handler
    let mut sa_term: libc::sigaction = mem::zeroed();
    sa_term.sa_sigaction = handle_shutdown_signal as usize;
    sa_term.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    libc::sigemptyset(&mut sa_term.sa_mask);

    if libc::sigaction(libc::SIGTERM, &sa_term, std::ptr::null_mut()) != 0 {
        return Err("Failed to register SIGTERM handler".into());
    }

    // Register SIGINT handler
    let mut sa_int: libc::sigaction = mem::zeroed();
    sa_int.sa_sigaction = handle_shutdown_signal as usize;
    sa_int.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    libc::sigemptyset(&mut sa_int.sa_mask);

    if libc::sigaction(libc::SIGINT, &sa_int, std::ptr::null_mut()) != 0 {
        return Err("Failed to register SIGINT handler".into());
    }

    Ok(())
}

#[cfg(not(feature = "dbus-support"))]
extern "C" fn handle_shutdown_signal(
    _sig: libc::c_int,
    _: *mut libc::siginfo_t,
    _: *mut libc::c_void,
) {
    if SIGNAL_RECEIVED.swap(true, Ordering::SeqCst) {
        return;
    }

    unsafe {
        let callbacks_ptr = std::ptr::addr_of!(GLOBAL_CALLBACKS);
        if let Some(callbacks) = (*callbacks_ptr).as_ref() {
            if let Some(callbacks_lock) = callbacks.try_read() {
                for callback in callbacks_lock.iter() {
                    callback();
                }
            }
            libc::sync();
            libc::usleep(100_000);
        }
    }

    unsafe {
        libc::_exit(0);
    }
}
