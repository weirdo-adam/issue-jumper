use crate::error::Result;
use crate::jump::{JumpOptions, JumpResult, resolve_jump};

use super::parse_jump_args;

pub fn run(args: &[String]) -> Result<()> {
    let parsed = parse_jump_args(args)?;
    let result = resolve_jump(JumpOptions {
        repo: parsed.repo,
        rule_name: parsed.rule,
        platform_override: parsed.platform,
    })?;
    if parsed.print_url {
        println!("{}", result.url);
    } else {
        print_jump_result(&result);
    }
    Ok(())
}

fn print_jump_result(result: &JumpResult) {
    println!("{}", result.url);
    println!("branch: {}", result.branch);
    println!("issue_id: {}", result.issue_id);
    println!("platform: {}", result.platform.name());
    println!("rule: {}", result.matched_rule);
    if let Some(remote) = &result.remote {
        println!("remote: {} {}", remote.remote_name, remote.original_url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prints_url_only_when_requested() {
        let repo = temp_repo("url-print");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-123"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );

        run(&[
            "--repo".to_string(),
            repo.display().to_string(),
            "--print-url".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn prints_full_jump_result_by_default() {
        let repo = temp_repo("url-full");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-456"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );

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
