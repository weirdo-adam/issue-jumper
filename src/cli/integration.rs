use std::path::PathBuf;

use serde_json::json;

use crate::error::{IssueJumperError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntegrationTarget {
    All,
    VsCode,
    Cursor,
    Generic,
}

pub fn run(args: &[String]) -> Result<()> {
    let mut target = IntegrationTarget::All;
    let mut command = current_command_path();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "print" => {}
            "--target" => {
                target = parse_target(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--target requires a value".to_string())
                })?)?;
            }
            "--command" => {
                command = PathBuf::from(iter.next().ok_or_else(|| {
                    IssueJumperError::Usage("--command requires a value".to_string())
                })?);
            }
            other => {
                return Err(IssueJumperError::Usage(format!(
                    "Unknown integration argument `{other}`"
                )));
            }
        }
    }

    print_examples(target, command.to_string_lossy().as_ref())?;
    Ok(())
}

fn parse_target(value: &str) -> Result<IntegrationTarget> {
    match value.to_ascii_lowercase().as_str() {
        "all" => Ok(IntegrationTarget::All),
        "vscode" | "vs-code" | "code" => Ok(IntegrationTarget::VsCode),
        "cursor" => Ok(IntegrationTarget::Cursor),
        "generic" | "shell" => Ok(IntegrationTarget::Generic),
        other => Err(IssueJumperError::Usage(format!(
            "Unknown integration target `{other}`"
        ))),
    }
}

fn print_examples(target: IntegrationTarget, command: &str) -> Result<()> {
    match target {
        IntegrationTarget::All => {
            print_vscode("VS Code", command)?;
            println!();
            print_vscode("Cursor", command)?;
            println!();
            print_generic(command);
        }
        IntegrationTarget::VsCode => print_vscode("VS Code", command)?,
        IntegrationTarget::Cursor => print_vscode("Cursor", command)?,
        IntegrationTarget::Generic => print_generic(command),
    }
    Ok(())
}

fn print_vscode(label: &str, command: &str) -> Result<()> {
    let task = json!({
        "version": "2.0.0",
        "tasks": [
            {
                "label": "Issue Jumper: Open Current Issue",
                "type": "shell",
                "command": command,
                "args": ["open", "--repo", "${workspaceFolder}"],
                "problemMatcher": [],
                "presentation": {
                    "reveal": "never",
                    "panel": "dedicated",
                    "clear": true
                }
            }
        ]
    });
    let keybinding = json!([
        {
            "key": "ctrl+alt+j",
            "command": "workbench.action.tasks.runTask",
            "args": "Issue Jumper: Open Current Issue"
        }
    ]);

    println!("{label} integration snippets");
    println!(".vscode/tasks.json:");
    println!("{}", pretty_json(&task)?);
    println!();
    println!("keybindings.json:");
    println!("{}", pretty_json(&keybinding)?);
    Ok(())
}

fn print_generic(command: &str) {
    let command = shell_quote(command);

    println!("Generic editor integration examples");
    println!("Open current repository issue or repository page:");
    println!("{command} open --repo /absolute/path/to/repo");
    println!();
    println!("Print the resolved URL for scripts:");
    println!("{command} url --repo /absolute/path/to/repo --print-url");
}

fn pretty_json(value: &serde_json::Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(|err| IssueJumperError::Io(err.to_string()))
}

fn current_command_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("issue-jumper"))
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_targets() {
        assert_eq!(parse_target("vscode").unwrap(), IntegrationTarget::VsCode);
        assert_eq!(parse_target("cursor").unwrap(), IntegrationTarget::Cursor);
        assert_eq!(parse_target("generic").unwrap(), IntegrationTarget::Generic);
        assert_eq!(parse_target("all").unwrap(), IntegrationTarget::All);
    }

    #[test]
    fn rejects_unknown_target() {
        let err = parse_target("missing").unwrap_err();
        assert!(matches!(err, IssueJumperError::Usage(_)));
    }

    #[test]
    fn prints_vscode_example() {
        run(&[
            "print".to_string(),
            "--target".to_string(),
            "vscode".to_string(),
            "--command".to_string(),
            "/opt/homebrew/bin/issue-jumper".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn quotes_shell_commands_with_spaces() {
        assert_eq!(
            shell_quote("/Applications/Issue Jumper"),
            "'/Applications/Issue Jumper'"
        );
    }
}
