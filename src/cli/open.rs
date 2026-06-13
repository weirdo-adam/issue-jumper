use crate::browser;
use crate::error::Result;
use crate::jump::{JumpOptions, resolve_jump};

use super::parse_jump_args;

pub fn run(args: &[String]) -> Result<()> {
    run_with_opener(args, browser::open_url)
}

fn run_with_opener<F>(args: &[String], opener: F) -> Result<()>
where
    F: FnOnce(&str) -> Result<()>,
{
    let parsed = parse_jump_args(args)?;
    let result = resolve_jump(JumpOptions {
        repo: parsed.repo,
        rule_name: parsed.rule,
        platform_override: parsed.platform,
    })?;
    opener(&result.url)?;
    println!("{}", result.url);
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
    fn opens_resolved_url_with_injected_opener() {
        let repo = temp_repo("open-success");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-123"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );
        let args = vec!["--repo".to_string(), repo.display().to_string()];
        let mut opened = None;

        run_with_opener(&args, |url| {
            opened = Some(url.to_string());
            Ok(())
        })
        .unwrap();

        assert_eq!(
            opened.as_deref(),
            Some("https://github.com/owner/repo/issues/123")
        );
    }

    #[test]
    fn propagates_opener_error() {
        let repo = temp_repo("open-error");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-123"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );
        let args = vec!["--repo".to_string(), repo.display().to_string()];

        let err = run_with_opener(&args, |_| {
            Err(crate::error::IssueJumperError::BrowserOpenFailed(
                "blocked".to_string(),
            ))
        })
        .unwrap_err();

        assert!(matches!(
            err,
            crate::error::IssueJumperError::BrowserOpenFailed(message) if message == "blocked"
        ));
    }

    #[test]
    fn rejects_unknown_open_argument() {
        let err = run(&["--bad".to_string()]).unwrap_err();
        assert!(matches!(err, crate::error::IssueJumperError::Usage(_)));
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
