#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod imp;

#[cfg(not(target_os = "windows"))]
#[path = "platform/unix.rs"]
mod imp;

pub(super) fn config_dir() -> Option<std::path::PathBuf> {
    imp::config_dir()
}
