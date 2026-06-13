use std::path::PathBuf;

pub(super) fn config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".config").join("zed"))
}
