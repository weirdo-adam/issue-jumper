use std::path::PathBuf;

use crate::config::load_config;
use crate::error::Result;
use crate::git::remote::{RemoteInfo, parse_remote};
use crate::git::{GitReader, resolve_repo};
use crate::issue::extract_issue;
use crate::platform::Platform;
use crate::url::{build_issue_url, resolve_platform};

#[derive(Debug, Clone, Default)]
pub struct JumpOptions {
    pub repo: Option<PathBuf>,
    pub rule_name: Option<String>,
    pub platform_override: Option<Platform>,
}

#[derive(Debug, Clone)]
pub struct JumpResult {
    pub repo: PathBuf,
    pub branch: String,
    pub issue_id: String,
    pub platform: Platform,
    pub matched_rule: String,
    pub url: String,
    pub remote: Option<RemoteInfo>,
}

pub fn resolve_jump(options: JumpOptions) -> Result<JumpResult> {
    let repo = resolve_repo(options.repo)?;
    let config = load_config(&repo)?;
    let git = GitReader::new(repo.clone())?;
    let branch = git.current_branch()?;
    let issue = extract_issue(&branch, &config, options.rule_name.as_deref())?;
    let remote = git
        .remote_url()?
        .map(|remote| parse_remote(&remote.name, &remote.url, &config))
        .transpose()?;
    let platform = resolve_platform(
        options.platform_override,
        issue.platform_hint.clone(),
        remote.as_ref(),
        &config,
    )?;
    let url = build_issue_url(&issue.issue_id, &platform, remote.as_ref(), &config)?;

    Ok(JumpResult {
        repo,
        branch,
        issue_id: issue.issue_id,
        platform,
        matched_rule: issue.rule_name,
        url,
        remote,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolves_redmine_url_without_remote() {
        let repo = temp_repo("redmine-no-remote");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/redmine-101"]);
        fs::create_dir_all(repo.join(".zed")).unwrap();
        fs::write(
            repo.join(".zed").join("issue-jumper.json"),
            r#"{
              "fallback_platform": "redmine",
              "redmine_base_url": "https://redmine.company.com",
              "issue_rules": [
                {
                  "name": "redmine-prefix",
                  "pattern": "(?i)redmine[-_](?P<id>\\d+)",
                  "platform": "redmine"
                }
              ]
            }"#,
        )
        .unwrap();

        let result = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap();

        assert_eq!(result.url, "https://redmine.company.com/issues/101");
        assert!(result.remote.is_none());
    }

    #[test]
    fn resolves_custom_platform_from_remote_host() {
        let repo = temp_repo("custom-platform");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/ABC-456"]);
        git(
            &repo,
            [
                "remote",
                "add",
                "origin",
                "https://jira.company.com/team/app.git",
            ],
        );
        fs::write(
            repo.join(".issue-jumper.json"),
            r#"{
              "custom_platforms": [
                {
                  "name": "jira",
                  "host_patterns": ["jira.company.com"],
                  "url_template": "https://jira.company.com/browse/{id}"
                }
              ]
            }"#,
        )
        .unwrap();

        let result = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap();

        assert_eq!(result.platform.name(), "jira");
        assert_eq!(result.url, "https://jira.company.com/browse/ABC-456");
    }

    #[test]
    fn propagates_repo_resolution_error() {
        let err = resolve_jump(JumpOptions {
            repo: Some(PathBuf::from("/definitely/missing/repo")),
            ..JumpOptions::default()
        })
        .unwrap_err();

        assert!(matches!(
            err,
            crate::error::IssueJumperError::RepoPathInvalid(_)
        ));
    }

    #[test]
    fn supports_rule_and_platform_overrides() {
        let repo = temp_repo("override");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/custom-77"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );
        fs::write(
            repo.join(".issue-jumper.json"),
            r#"{
              "redmine_base_url": "https://redmine.company.com",
              "issue_rules": [
                {
                  "name": "custom",
                  "pattern": "custom-(?P<id>\\d+)"
                }
              ]
            }"#,
        )
        .unwrap();

        let result = resolve_jump(JumpOptions {
            repo: Some(repo),
            rule_name: Some("custom".to_string()),
            platform_override: Some(Platform::Redmine),
        })
        .unwrap();

        assert_eq!(result.issue_id, "77");
        assert_eq!(result.platform, Platform::Redmine);
        assert_eq!(result.url, "https://redmine.company.com/issues/77");
    }

    #[test]
    fn fails_when_platform_cannot_be_resolved() {
        let repo = temp_repo("no-platform");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "feature/GH-123"]);

        let err = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap_err();

        assert!(matches!(
            err,
            crate::error::IssueJumperError::PlatformUnresolved
        ));
    }

    fn temp_repo(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-{label}-{nonce}"))
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
