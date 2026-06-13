use std::path::PathBuf;

use crate::error::{IssueJumperError, Result};
use crate::platform::Platform;

mod doctor;
mod install_zed;
mod open;
mod url;

#[derive(Debug, Default)]
pub(super) struct ParsedJumpArgs {
    pub repo: Option<PathBuf>,
    pub platform: Option<Platform>,
    pub rule: Option<String>,
    pub print_url: bool,
}

pub fn run(args: Vec<String>) -> Result<()> {
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };

    match command {
        "open" => open::run(&args[1..]),
        "url" => url::run(&args[1..]),
        "install-zed" => install_zed::run(&args[1..]),
        "doctor" => doctor::run(&args[1..]),
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(IssueJumperError::Usage(format!(
            "Unknown command `{other}`"
        ))),
    }
}

pub(super) fn parse_jump_args(args: &[String]) -> Result<ParsedJumpArgs> {
    let mut parsed = ParsedJumpArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo" => {
                parsed.repo = Some(PathBuf::from(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--repo requires a value".to_string())
                })?));
            }
            "--platform" => {
                parsed.platform = Some(Platform::parse(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--platform requires a value".to_string())
                })?));
            }
            "--rule" => {
                parsed.rule = Some(
                    iter.next()
                        .ok_or_else(|| {
                            IssueJumperError::Usage("--rule requires a value".to_string())
                        })?
                        .clone(),
                );
            }
            "--print-url" => parsed.print_url = true,
            other => {
                return Err(IssueJumperError::Usage(format!(
                    "Unknown argument `{other}`"
                )));
            }
        }
    }

    Ok(parsed)
}

fn print_help() {
    println!("Issue Jumper");
    println!();
    println!("Usage:");
    println!("  issue-jumper open [--repo <path>] [--platform <name>] [--rule <name>]");
    println!(
        "  issue-jumper url [--repo <path>] [--platform <name>] [--rule <name>] [--print-url]"
    );
    println!("  issue-jumper install-zed [--key <key>] [--force] [--print]");
    println!("  issue-jumper doctor [--repo <path>]");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_help_without_command() {
        run(Vec::new()).unwrap();
    }

    #[test]
    fn prints_help_for_help_commands() {
        for command in ["help", "-h", "--help"] {
            run(vec![command.to_string()]).unwrap();
        }
    }

    #[test]
    fn rejects_unknown_command() {
        let err = run(vec!["missing".to_string()]).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::Usage(message) if message.contains("Unknown command"))
        );
    }

    #[test]
    fn parses_jump_arguments() {
        let parsed = parse_jump_args(&[
            "--repo".to_string(),
            "/tmp/repo".to_string(),
            "--platform".to_string(),
            "github".to_string(),
            "--rule".to_string(),
            "gh".to_string(),
            "--print-url".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.repo.unwrap(), PathBuf::from("/tmp/repo"));
        assert_eq!(parsed.platform.unwrap(), Platform::GitHub);
        assert_eq!(parsed.rule.as_deref(), Some("gh"));
        assert!(parsed.print_url);
    }

    #[test]
    fn rejects_missing_jump_argument_values() {
        for arg in ["--repo", "--platform", "--rule"] {
            let err = parse_jump_args(&[arg.to_string()]).unwrap_err();
            assert!(matches!(err, IssueJumperError::Usage(_)));
        }
    }
}
