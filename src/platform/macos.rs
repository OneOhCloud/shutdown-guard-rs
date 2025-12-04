//! macOS platform-specific implementation using signal handlers
//!
//! This implementation uses Unix signals (SIGTERM, SIGINT) to detect shutdown.
//! On macOS, we need to handle signals synchronously and ensure immediate file writes.

use crate::ShutdownCallback;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static mut GLOBAL_CALLBACKS: Option<Arc<RwLock<Vec<ShutdownCallback>>>> = None;
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Starts monitoring for macOS shutdown events
pub fn start_monitoring(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Store callbacks globally for signal handler access
    unsafe {
        GLOBAL_CALLBACKS = Some(callbacks);
    }

    // Register signal handlers
    unsafe {
        register_signal_handlers()?;
    }

    println!("macOS shutdown monitoring active (using signal handlers)");
    println!("⚠️  Note: The application should run in the background or use nohup to avoid shutdown dialogs");

    Ok(())
}

unsafe fn register_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
    use std::mem;

    // Register SIGTERM handler (most common shutdown signal)
    let mut sa_term: libc::sigaction = mem::zeroed();
    sa_term.sa_sigaction = handle_shutdown_signal as usize;
    sa_term.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    libc::sigemptyset(&mut sa_term.sa_mask);

    if libc::sigaction(libc::SIGTERM, &sa_term, std::ptr::null_mut()) != 0 {
        return Err("Failed to register SIGTERM handler".into());
    }

    // Register SIGINT handler (Ctrl+C)
    let mut sa_int: libc::sigaction = mem::zeroed();
    sa_int.sa_sigaction = handle_shutdown_signal as usize;
    sa_int.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    libc::sigemptyset(&mut sa_int.sa_mask);

    if libc::sigaction(libc::SIGINT, &sa_int, std::ptr::null_mut()) != 0 {
        return Err("Failed to register SIGINT handler".into());
    }

    // Register SIGHUP handler
    let mut sa_hup: libc::sigaction = mem::zeroed();
    sa_hup.sa_sigaction = handle_shutdown_signal as usize;
    sa_hup.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    libc::sigemptyset(&mut sa_hup.sa_mask);

    if libc::sigaction(libc::SIGHUP, &sa_hup, std::ptr::null_mut()) != 0 {
        return Err("Failed to register SIGHUP handler".into());
    }

    Ok(())
}

extern "C" fn handle_shutdown_signal(
    sig: libc::c_int,
    _: *mut libc::siginfo_t,
    _: *mut libc::c_void,
) {
    // Avoid executing multiple times
    if SIGNAL_RECEIVED.swap(true, Ordering::SeqCst) {
        return;
    }

    // Write signal info immediately (signal-safe)
    let msg = match sig {
        libc::SIGTERM => "Received SIGTERM\n\0",
        libc::SIGINT => "Received SIGINT\n\0",
        libc::SIGHUP => "Received SIGHUP\n\0",
        _ => "Received unknown signal\n\0",
    };

    unsafe {
        libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len() - 1);
    }

    // Execute callbacks - note: this is NOT signal-safe but we need it for functionality
    unsafe {
        let callbacks_ptr = std::ptr::addr_of!(GLOBAL_CALLBACKS);
        if let Some(callbacks) = (*callbacks_ptr).as_ref() {
            // Try to lock, but don't block forever
            if let Some(callbacks_lock) = callbacks.try_read() {
                for callback in callbacks_lock.iter() {
                    callback();
                }
            }

            // Force sync all file descriptors to disk
            libc::sync();

            // Small delay to ensure writes complete
            libc::usleep(100_000); // 100ms
        }
    }

    // Exit immediately
    unsafe {
        libc::_exit(0);
    }
}
