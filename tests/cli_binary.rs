use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn binary_prints_help_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Issue Jumper"));
}

#[test]
fn binary_prints_version_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        concat!("issue-jumper ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn binary_reports_unknown_command_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("missing")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown command"));
}

#[test]
fn binary_prints_zed_snippets_without_writing_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .args(["install-zed", "--print"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tasks.json"));
    assert!(stdout.contains("keymap.json"));
    assert!(stdout.contains("issue-jumper"));
}

#[test]
fn binary_installs_zed_config_into_temp_home() {
    let home = temp_dir("zed-home");
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("install-zed")
        .env("HOME", &home)
        .env("APPDATA", home.join("AppData").join("Roaming"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let zed_config_dir = platform_zed_config_dir(&home);
    let tasks = std::fs::read_to_string(zed_config_dir.join("tasks.json")).unwrap();
    let keymap = std::fs::read_to_string(zed_config_dir.join("keymap.json")).unwrap();

    assert!(tasks.contains("Issue Jumper: Open Current Issue"));
    assert!(tasks.contains("issue-jumper"));
    assert!(keymap.contains("alt alt"));
}

#[test]
fn binary_reports_missing_zed_config_dir_without_home() {
    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("install-zed")
        .env_remove("HOME")
        .env_remove("APPDATA")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Zed configuration directory"));
}

#[test]
fn binary_reports_zed_config_creation_failure() {
    let home = temp_dir("zed-home-file");
    std::fs::write(&home, "not a directory").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_issue-jumper"))
        .arg("install-zed")
        .env("HOME", &home)
        .env("APPDATA", home.join("AppData").join("Roaming"))
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty());
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("issue-jumper-binary-{label}-{nonce}"))
}

fn platform_zed_config_dir(home: &std::path::Path) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        home.join("AppData").join("Roaming").join("Zed")
    }

    #[cfg(not(target_os = "windows"))]
    {
        home.join(".config").join("zed")
    }
}
