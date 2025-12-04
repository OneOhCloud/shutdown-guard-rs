//! Windows platform-specific implementation using Console Control Handler

use crate::ShutdownCallback;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::BOOL;
use windows::Win32::System::Console::*;

static mut GLOBAL_CALLBACKS: Option<Arc<RwLock<Vec<ShutdownCallback>>>> = None;
static HANDLER_CALLED: AtomicBool = AtomicBool::new(false);

/// Starts monitoring for Windows shutdown events
pub fn start_monitoring(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    unsafe {
        GLOBAL_CALLBACKS = Some(callbacks);

        SetConsoleCtrlHandler(Some(console_ctrl_handler), true)
            .map_err(|e| format!("Failed to set console control handler: {}", e))?;

        println!("Windows shutdown monitoring active (using Console Control Handler)");
    }

    Ok(())
}

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    // Only handle shutdown events, not Ctrl+C
    match ctrl_type {
        CTRL_SHUTDOWN_EVENT | CTRL_LOGOFF_EVENT | CTRL_CLOSE_EVENT => {
            // Avoid executing multiple times
            if HANDLER_CALLED.swap(true, Ordering::SeqCst) {
                return BOOL(1);
            }

            // Execute all registered callbacks
            let callbacks_ptr = std::ptr::addr_of!(GLOBAL_CALLBACKS);
            if let Some(callbacks) = (*callbacks_ptr).as_ref() {
                if let Some(callbacks_lock) = callbacks.try_read() {
                    for callback in callbacks_lock.iter() {
                        callback();
                    }
                }
            }

            BOOL(1)
        }
        _ => BOOL(0),
    }
}
