//! # Shutdown Guard
//!
//! A cross-platform library for registering cleanup callbacks that execute before system shutdown.
//!
//! ## Example
//!
//! ```rust
//! use shutdown_guard::{ShutdownGuard, ShutdownCallback};
//!
//! fn main() {
//!     let guard = ShutdownGuard::new();
//!     
//!     // Register a cleanup callback
//!     guard.register(Box::new(|| {
//!         println!("Cleaning up before shutdown...");
//!         // Perform cleanup operations
//!     }));
//!     
//!     // Start monitoring for shutdown events
//!     guard.start().expect("Failed to start shutdown monitoring");
//!     
//!     // Your application code here...
//! }
//! ```

use parking_lot::RwLock;
use std::sync::Arc;

/// A callback function that will be executed before system shutdown
pub type ShutdownCallback = Box<dyn Fn() + Send + Sync + 'static>;

/// Platform-specific shutdown monitoring implementation
#[cfg(target_os = "macos")]
mod platform;

#[cfg(target_os = "windows")]
mod platform;

#[cfg(target_os = "linux")]
mod platform;

/// Main structure for managing shutdown callbacks
pub struct ShutdownGuard {
    callbacks: Arc<RwLock<Vec<ShutdownCallback>>>,
}

impl ShutdownGuard {
    /// Creates a new ShutdownGuard instance
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registers a callback to be executed before shutdown
    ///
    /// # Arguments
    ///
    /// * `callback` - A boxed closure that will be called before shutdown
    ///
    /// # Example
    ///
    /// ```
    /// use shutdown_guard::ShutdownGuard;
    ///
    /// let guard = ShutdownGuard::new();
    /// guard.register(Box::new(|| {
    ///     println!("Cleanup in progress...");
    /// }));
    /// ```
    pub fn register(&self, callback: ShutdownCallback) {
        self.callbacks.write().push(callback);
    }

    /// Starts monitoring for shutdown events
    ///
    /// This method begins listening for system shutdown signals and will
    /// execute all registered callbacks when a shutdown is detected.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if monitoring started successfully, or an error if
    /// the platform-specific implementation failed to initialize.
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let callbacks = Arc::clone(&self.callbacks);
        platform::start_monitoring(callbacks)
    }

    /// Executes all registered callbacks
    ///
    /// This method is typically called automatically when a shutdown is detected,
    /// but can also be called manually if needed.
    pub fn execute_callbacks(&self) {
        let callbacks = self.callbacks.read();
        for callback in callbacks.iter() {
            callback();
        }
    }

    /// Returns the number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.read().len()
    }

    /// Clears all registered callbacks
    pub fn clear(&self) {
        self.callbacks.write().clear();
    }
}

impl Default for ShutdownGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_register_callback() {
        let guard = ShutdownGuard::new();
        assert_eq!(guard.callback_count(), 0);

        guard.register(Box::new(|| println!("Test callback")));
        assert_eq!(guard.callback_count(), 1);
    }

    #[test]
    fn test_execute_callbacks() {
        let guard = ShutdownGuard::new();
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = Arc::clone(&executed);

        guard.register(Box::new(move || {
            executed_clone.store(true, Ordering::SeqCst);
        }));

        guard.execute_callbacks();
        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_clear_callbacks() {
        let guard = ShutdownGuard::new();
        guard.register(Box::new(|| println!("Test")));
        assert_eq!(guard.callback_count(), 1);

        guard.clear();
        assert_eq!(guard.callback_count(), 0);
    }
}
