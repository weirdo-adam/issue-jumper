use std::path::PathBuf;

use crate::config::lint_config;
use crate::error::{IssueJumperError, Result};

#[derive(Debug, Default)]
struct ParsedConfigLintArgs {
    repo: Option<PathBuf>,
    paths: Vec<PathBuf>,
}

pub fn run_config_command(args: &[String]) -> Result<()> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(IssueJumperError::Usage(
            "config requires a subcommand: lint".to_string(),
        ));
    };

    match command {
        "lint" => run_lint(&args[1..]),
        other => Err(IssueJumperError::Usage(format!(
            "Unknown config subcommand `{other}`"
        ))),
    }
}

pub fn run_lint(args: &[String]) -> Result<()> {
    let parsed = parse_lint_args(args)?;
    let repo = parsed
        .repo
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let report = lint_config(&repo, &parsed.paths);

    println!("Issue Jumper config lint");

    if report.checked_paths.is_empty() {
        println!("No Issue Jumper config files found.");
    } else {
        for path in &report.checked_paths {
            println!("Checked: {}", path.display());
        }
    }

    if report.is_ok() {
        println!("OK");
        return Ok(());
    }

    for error in &report.errors {
        match &error.path {
            Some(path) => eprintln!("{}: {}", path.display(), error.message),
            None => eprintln!("{}", error.message),
        }
    }

    Err(IssueJumperError::InvalidConfig(format!(
        "config lint failed with {} error(s)",
        report.errors.len()
    )))
}

fn parse_lint_args(args: &[String]) -> Result<ParsedConfigLintArgs> {
    let mut parsed = ParsedConfigLintArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo" => {
                parsed.repo = Some(PathBuf::from(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--repo requires a value".to_string())
                })?));
            }
            "--path" => {
                parsed.paths.push(PathBuf::from(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--path requires a value".to_string())
                })?));
            }
            other => {
                return Err(IssueJumperError::Usage(format!(
                    "Unknown config lint argument `{other}`"
                )));
            }
        }
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn lint_accepts_valid_explicit_config() {
        let dir = temp_dir("valid");
        fs::create_dir_all(&dir).unwrap();
        let config = dir.join("issue-jumper.json");
        fs::write(
            &config,
            r#"{
              "issue_rules": [
                {
                  "name": "ticket",
                  "pattern": "TICKET-(?P<id>\\d+)",
                  "platform": "github"
                }
              ]
            }"#,
        )
        .unwrap();

        run_lint(&["--path".to_string(), config.display().to_string()]).unwrap();
    }

    #[test]
    fn lint_rejects_invalid_explicit_config() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(&dir).unwrap();
        let config = dir.join("issue-jumper.json");
        fs::write(
            &config,
            r#"{
              "issue_rules": [
                {
                  "name": "ticket",
                  "pattern": "TICKET-(\\d+)"
                }
              ]
            }"#,
        )
        .unwrap();

        let err = run_lint(&["--path".to_string(), config.display().to_string()]).unwrap_err();

        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains("lint failed"))
        );
    }

    #[test]
    fn config_command_requires_lint_subcommand() {
        let err = run_config_command(&[]).unwrap_err();
        assert!(matches!(err, IssueJumperError::Usage(_)));
    }

    #[test]
    fn parse_lint_rejects_missing_values() {
        for arg in ["--repo", "--path"] {
            let err = parse_lint_args(&[arg.to_string()]).unwrap_err();
            assert!(matches!(err, IssueJumperError::Usage(_)));
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-config-lint-{label}-{nonce}"))
    }
}
