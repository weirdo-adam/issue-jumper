#[cfg(target_os = "macos")]
#[path = "platform/macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod imp;

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#[path = "platform/unix.rs"]
mod imp;

pub(super) fn command(url: &str) -> super::BrowserCommand {
    imp::command(url)
}
