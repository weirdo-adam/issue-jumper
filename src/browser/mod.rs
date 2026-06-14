use std::process::Command;

use crate::error::{IssueJumperError, Result};

mod platform;

pub fn open_url(url: &str) -> Result<()> {
    let mut runner = run_browser_command;
    open_url_with(url, &mut runner)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserCommand {
    program: &'static str,
    args: Vec<String>,
}

fn open_url_with(
    url: &str,
    run: &mut dyn for<'a> FnMut(&'a BrowserCommand) -> std::io::Result<std::process::ExitStatus>,
) -> Result<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(IssueJumperError::BrowserOpenFailed(format!(
            "Refusing to open non-http URL: {url}"
        )));
    }

    let command = BrowserCommand::new(url);
    match run(&command) {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(IssueJumperError::BrowserOpenFailed(format!(
            "URL: {url}; opener exit status: {status}"
        ))),
        Err(err) => Err(IssueJumperError::BrowserOpenFailed(format!(
            "URL: {url}; {err}"
        ))),
    }
}

impl BrowserCommand {
    fn new(url: &str) -> Self {
        platform::command(url)
    }
}

fn run_browser_command(command: &BrowserCommand) -> std::io::Result<std::process::ExitStatus> {
    Command::new(command.program).args(&command.args).status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(windows)]
    fn exit_status(code: u32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }

    fn successful_opener(_: &BrowserCommand) -> io::Result<std::process::ExitStatus> {
        Ok(exit_status(0))
    }

    fn failing_opener(_: &BrowserCommand) -> io::Result<std::process::ExitStatus> {
        Ok(exit_status(1))
    }

    fn missing_opener(_: &BrowserCommand) -> io::Result<std::process::ExitStatus> {
        Err(io::Error::new(io::ErrorKind::NotFound, "missing opener"))
    }

    #[test]
    fn rejects_non_http_urls() {
        let err = open_url("file:///tmp/x").unwrap_err();
        assert!(
            matches!(err, IssueJumperError::BrowserOpenFailed(message) if message.contains("non-http"))
        );
    }

    #[test]
    fn builds_platform_command_and_handles_success() {
        let mut seen = None;
        let mut runner = |command: &BrowserCommand| {
            seen = Some(command.clone());
            Ok(exit_status(0))
        };
        open_url_with("https://example.com/issue/1", &mut runner).unwrap();

        let command = seen.unwrap();
        assert_eq!(command.program, EXPECTED_PROGRAM);
        assert!(
            command
                .args
                .iter()
                .any(|arg| arg == "https://example.com/issue/1")
        );
    }

    #[test]
    fn accepts_http_urls() {
        let mut runner = successful_opener;
        open_url_with("http://example.com/issue/1", &mut runner).unwrap();
    }

    #[test]
    fn reports_non_zero_opener_status() {
        let mut runner = failing_opener;
        let err = open_url_with("https://example.com", &mut runner).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::BrowserOpenFailed(message) if message.contains("exit status"))
        );
    }

    #[test]
    fn reports_opener_io_error() {
        let mut runner = missing_opener;
        let err = open_url_with("https://example.com", &mut runner).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::BrowserOpenFailed(message) if message.contains("missing opener"))
        );
    }

    #[test]
    fn runs_browser_command() {
        #[cfg(target_os = "windows")]
        let command = BrowserCommand {
            program: "cmd",
            args: vec![
                "/C".to_string(),
                "exit".to_string(),
                "/B".to_string(),
                "0".to_string(),
            ],
        };

        #[cfg(not(target_os = "windows"))]
        let command = BrowserCommand {
            program: "true",
            args: Vec::new(),
        };

        assert!(run_browser_command(&command).unwrap().success());
    }

    #[cfg(target_os = "macos")]
    const EXPECTED_PROGRAM: &str = "open";

    #[cfg(target_os = "windows")]
    const EXPECTED_PROGRAM: &str = "cmd";

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    const EXPECTED_PROGRAM: &str = "xdg-open";
}
