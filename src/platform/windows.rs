//! Windows platform-specific implementation using Windows API

use crate::ShutdownCallback;
use parking_lot::RwLock;
use std::sync::Arc;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Shutdown::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut GLOBAL_CALLBACKS: Option<Arc<RwLock<Vec<ShutdownCallback>>>> = None;

/// Starts monitoring for Windows shutdown events
pub fn start_monitoring(
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        GLOBAL_CALLBACKS = Some(callbacks);

        // Create a hidden window to receive shutdown messages
        std::thread::spawn(|| {
            if let Err(e) = create_message_window() {
                eprintln!("Failed to create message window: {}", e);
            }
        });
    }

    Ok(())
}

unsafe fn create_message_window() -> Result<()> {
    let class_name = w!("ShutdownGuardWindowClass");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: class_name,
        ..Default::default()
    };

    RegisterClassW(&wc);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        w!("ShutdownGuard"),
        WINDOW_STYLE(0),
        0,
        0,
        0,
        0,
        HWND(0),
        None,
        None,
        None,
    );

    if hwnd.0 == 0 {
        return Err(Error::from_win32());
    }

    // Message loop
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_QUERYENDSESSION | WM_ENDSESSION => {
            // Execute all registered callbacks
            if let Some(callbacks) = &GLOBAL_CALLBACKS {
                let callbacks_lock = callbacks.read();
                for callback in callbacks_lock.iter() {
                    callback();
                }
            }
            LRESULT(1) // Allow shutdown to proceed
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
