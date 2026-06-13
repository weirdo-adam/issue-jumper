use crate::error::Result;
use crate::jump::{JumpOptions, JumpTarget, resolve_jump};
use crate::zed::task_label;

use super::parse_jump_args;

pub fn run(args: &[String]) -> Result<()> {
    let parsed = parse_jump_args(args)?;
    let result = resolve_jump(JumpOptions {
        repo: parsed.repo,
        rule_name: parsed.rule,
        platform_override: parsed.platform,
    })?;
    println!("Issue Jumper doctor");
    println!("Repo: {}", result.repo.display());
    println!("Branch: {}", result.branch);
    println!("Platform: {}", result.platform.name());
    match &result.target {
        JumpTarget::Issue {
            issue_id,
            matched_rule,
        } => {
            println!("Target: issue");
            println!("Issue ID: {issue_id}");
            println!("Rule: {matched_rule}");
        }
        JumpTarget::Repository => {
            println!("Target: repository");
        }
    }
    if let Some(remote) = &result.remote {
        println!("Remote: {} ({})", remote.remote_name, remote.original_url);
    } else {
        println!("Remote: none");
    }
    println!("URL: {}", result.url);
    println!("Zed task label: {}", task_label());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prints_diagnostics_for_repo() {
        let repo = temp_repo("doctor");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-123"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );

        run(&["--repo".to_string(), repo.display().to_string()]).unwrap();
    }

    #[test]
    fn prints_diagnostics_without_remote() {
        let repo = temp_repo("doctor-no-remote");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/redmine-123"]);
        fs::write(
            repo.join(".issue-jumper.json"),
            r#"{
              "fallback_platform": "redmine",
              "redmine_base_url": "https://redmine.company.com",
              "issue_rules": [
                {
                  "name": "redmine",
                  "pattern": "redmine-(?P<id>\\d+)",
                  "platform": "redmine"
                }
              ]
            }"#,
        )
        .unwrap();

        run(&["--repo".to_string(), repo.display().to_string()]).unwrap();
    }

    fn temp_repo(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-cli-{label}-{nonce}"))
    }

    fn git<const N: usize>(repo: &Path, args: [&str; N]) {
        fs::create_dir_all(repo).unwrap();
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
