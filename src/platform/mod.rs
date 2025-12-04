//! Platform-specific implementations

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub use macos::start_monitoring;

#[cfg(target_os = "windows")]
pub use windows::start_monitoring;

#[cfg(target_os = "linux")]
pub use linux::start_monitoring;
