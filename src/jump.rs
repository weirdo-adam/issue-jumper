use std::path::PathBuf;

use crate::config::{UserConfig, load_config};
use crate::error::{IssueJumperError, Result};
use crate::git::remote::{RemoteInfo, parse_remote};
use crate::git::{GitReader, RemoteUrl, resolve_repo};
use crate::issue::extract_issue;
use crate::platform::Platform;
use crate::url::{build_issue_url, build_repository_url, resolve_platform};

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
    pub platform: Platform,
    pub target: JumpTarget,
    pub url: String,
    pub remote: Option<RemoteInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpTarget {
    Issue {
        issue_id: String,
        matched_rule: String,
    },
    Repository,
}

pub fn resolve_jump(options: JumpOptions) -> Result<JumpResult> {
    let repo = resolve_repo(options.repo)?;
    let config = load_config(&repo)?;
    let git = GitReader::new(repo.clone())?;
    let branch = git.current_branch()?;
    let allow_repository_fallback = options.rule_name.is_none();
    let issue = match extract_issue(&branch, &config, options.rule_name.as_deref()) {
        Ok(issue) => issue,
        Err(IssueJumperError::NoMatchingRule(branch)) if allow_repository_fallback => {
            return resolve_repository_jump(repo, branch, &git, &config);
        }
        Err(err) => return Err(err),
    };
    let remote = resolve_remote(&git, &config)?;
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
        platform,
        target: JumpTarget::Issue {
            issue_id: issue.issue_id,
            matched_rule: issue.rule_name,
        },
        url,
        remote,
    })
}

fn resolve_repository_jump(
    repo: PathBuf,
    branch: String,
    git: &GitReader,
    config: &UserConfig,
) -> Result<JumpResult> {
    let no_match = IssueJumperError::NoMatchingRule(branch.clone());
    let Some(remote) = optional_remote_for_no_match(git, config) else {
        return Err(no_match);
    };
    let platform = remote.platform.clone();
    if !matches!(platform, Platform::GitHub | Platform::GitLab) {
        return Err(no_match);
    }
    let url = build_repository_url(&platform, Some(&remote))?;

    Ok(JumpResult {
        repo,
        branch,
        platform,
        target: JumpTarget::Repository,
        url,
        remote: Some(remote),
    })
}

fn optional_remote_for_no_match(git: &GitReader, config: &UserConfig) -> Option<RemoteInfo> {
    resolve_remote(git, config).ok().flatten()
}

fn resolve_remote(git: &GitReader, config: &UserConfig) -> Result<Option<RemoteInfo>> {
    git.remote_url()?
        .map(|remote| parse_remote_url(remote, config))
        .transpose()
}

fn parse_remote_url(remote: RemoteUrl, config: &UserConfig) -> Result<RemoteInfo> {
    parse_remote(&remote.name, &remote.url, config)
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

        assert_eq!(result.platform, Platform::Redmine);
        assert_eq!(result.url, "https://redmine.company.com/issues/77");
        assert_eq!(
            result.target,
            JumpTarget::Issue {
                issue_id: "77".to_string(),
                matched_rule: "custom".to_string(),
            }
        );
    }

    #[test]
    fn falls_back_to_github_repository_when_no_issue_matches() {
        let repo = temp_repo("github-repo-fallback");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "main"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );

        let result = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap();

        assert_eq!(result.platform, Platform::GitHub);
        assert_eq!(result.target, JumpTarget::Repository);
        assert_eq!(result.url, "https://github.com/owner/repo");
    }

    #[test]
    fn does_not_fallback_to_repository_when_rule_override_does_not_match() {
        let repo = temp_repo("github-rule-no-fallback");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "main"]);
        git(
            &repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );
        fs::write(
            repo.join(".issue-jumper.json"),
            r#"{
              "issue_rules": [
                {
                  "name": "ticket",
                  "pattern": "TICKET-(?P<id>\\d+)"
                }
              ]
            }"#,
        )
        .unwrap();

        let err = resolve_jump(JumpOptions {
            repo: Some(repo),
            rule_name: Some("ticket".to_string()),
            ..JumpOptions::default()
        })
        .unwrap_err();

        assert!(matches!(
            err,
            crate::error::IssueJumperError::NoMatchingRule(branch) if branch == "main"
        ));
    }

    #[test]
    fn falls_back_to_gitlab_repository_when_no_issue_matches() {
        let repo = temp_repo("gitlab-repo-fallback");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "main"]);
        git(
            &repo,
            [
                "remote",
                "add",
                "origin",
                "git@gitlab.com:group/subgroup/app.git",
            ],
        );

        let result = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap();

        assert_eq!(result.platform, Platform::GitLab);
        assert_eq!(result.target, JumpTarget::Repository);
        assert_eq!(result.url, "https://gitlab.com/group/subgroup/app");
    }

    #[test]
    fn keeps_no_matching_rule_for_non_github_gitlab_remotes() {
        let repo = temp_repo("no-repo-fallback");
        git(&repo, ["init"]);
        git(&repo, ["checkout", "-b", "main"]);
        git(
            &repo,
            [
                "remote",
                "add",
                "origin",
                "git@bitbucket.org:owner/repo.git",
            ],
        );

        let err = resolve_jump(JumpOptions {
            repo: Some(repo),
            ..JumpOptions::default()
        })
        .unwrap_err();

        assert!(matches!(
            err,
            crate::error::IssueJumperError::NoMatchingRule(branch) if branch == "main"
        ));
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
