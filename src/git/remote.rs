use crate::config::UserConfig;
use crate::error::{IssueJumperError, Result};
use crate::platform::Platform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteInfo {
    pub remote_name: String,
    pub original_url: String,
    pub host: String,
    pub path: String,
    pub platform: Platform,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub project: Option<String>,
}

pub fn parse_remote(
    remote_name: &str,
    remote_url: &str,
    config: &UserConfig,
) -> Result<RemoteInfo> {
    let (host, path) = split_remote_url(remote_url)
        .ok_or_else(|| IssueJumperError::RemoteParseFailed(remote_url.to_string()))?;
    let cleaned_path = clean_path(&path);
    if host.is_empty() || cleaned_path.is_empty() {
        return Err(IssueJumperError::RemoteParseFailed(remote_url.to_string()));
    }

    let platform = detect_platform(&host, config);
    let segments: Vec<&str> = cleaned_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let owner = segments.first().map(|value| (*value).to_string());
    let repo = segments.last().map(|value| (*value).to_string());

    Ok(RemoteInfo {
        remote_name: remote_name.to_string(),
        original_url: remote_url.to_string(),
        host,
        path: cleaned_path.clone(),
        platform,
        owner,
        repo,
        project: Some(cleaned_path),
    })
}

fn split_remote_url(value: &str) -> Option<(String, String)> {
    if let Some((left, right)) = value.split_once("://") {
        let without_scheme = right;
        let slash = without_scheme.find('/')?;
        let authority = &without_scheme[..slash];
        let path = &without_scheme[slash + 1..];
        let host_port = authority.rsplit('@').next().unwrap_or(authority);
        let host = host_port.split(':').next().unwrap_or(host_port);
        if left == "http" || left == "https" || left == "ssh" || left == "git" {
            return Some((host.to_string(), path.to_string()));
        }
    }

    if let Some((left, path)) = value.split_once(':')
        && left.contains('@')
        && !path.starts_with("//")
    {
        let host = left.rsplit('@').next()?;
        return Some((host.to_string(), path.to_string()));
    }

    None
}

fn clean_path(path: &str) -> String {
    path.trim_start_matches('/')
        .trim_end_matches(".git")
        .trim_end_matches('/')
        .to_string()
}

fn detect_platform(host: &str, config: &UserConfig) -> Platform {
    let host_lower = host.to_ascii_lowercase();
    for platform in &config.custom_platforms {
        if platform
            .host_patterns
            .iter()
            .any(|pattern| host_lower == pattern.to_ascii_lowercase())
        {
            return Platform::Custom(platform.name.clone());
        }
    }

    match host_lower.as_str() {
        "github.com" => Platform::GitHub,
        "gitlab.com" => Platform::GitLab,
        "bitbucket.org" => Platform::Bitbucket,
        "gitee.com" => Platform::Gitee,
        _ if looks_like_private_gitlab_host(&host_lower) => Platform::GitLab,
        _ => Platform::Custom(host.to_string()),
    }
}

fn looks_like_private_gitlab_host(host: &str) -> bool {
    host.starts_with("gitlab.") || host.contains(".gitlab.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_remote_urls() {
        let config = UserConfig::default();
        let remote = parse_remote("origin", "git@github.com:owner/repo.git", &config).unwrap();
        assert_eq!(remote.host, "github.com");
        assert_eq!(remote.path, "owner/repo");
        assert_eq!(remote.owner.as_deref(), Some("owner"));
        assert_eq!(remote.repo.as_deref(), Some("repo"));

        let remote =
            parse_remote("origin", "ssh://git@gitlab.com:2222/team/app.git", &config).unwrap();
        assert_eq!(remote.host, "gitlab.com");
        assert_eq!(remote.project.as_deref(), Some("team/app"));
    }

    #[test]
    fn parses_https_git_protocol_bitbucket_and_gitee_urls() {
        let config = UserConfig::default();

        let remote = parse_remote("origin", "https://bitbucket.org/team/app.git", &config).unwrap();
        assert_eq!(remote.platform, Platform::Bitbucket);
        assert_eq!(remote.path, "team/app");

        let remote = parse_remote("origin", "git://gitee.com/team/app.git", &config).unwrap();
        assert_eq!(remote.platform, Platform::Gitee);
        assert_eq!(remote.path, "team/app");
    }

    #[test]
    fn parses_custom_and_unknown_hosts() {
        let config = UserConfig {
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "jira".to_string(),
                host_patterns: vec!["jira.company.com".to_string()],
                url_template: "https://jira.company.com/browse/{id}".to_string(),
            }],
            ..UserConfig::default()
        };

        let remote = parse_remote("origin", "https://jira.company.com/team/app/", &config).unwrap();
        assert_eq!(remote.platform, Platform::Custom("jira".to_string()));
        assert_eq!(remote.path, "team/app");

        let remote =
            parse_remote("origin", "https://git.example.com/team/app.git", &config).unwrap();
        assert_eq!(
            remote.platform,
            Platform::Custom("git.example.com".to_string())
        );
    }

    #[test]
    fn detects_private_gitlab_hosts_by_convention() {
        let config = UserConfig::default();

        let remote = parse_remote(
            "origin",
            "https://gitlab.example.com/devops/app.git",
            &config,
        )
        .unwrap();
        assert_eq!(remote.platform, Platform::GitLab);

        let remote = parse_remote(
            "origin",
            "ssh://git@code.gitlab.company.com/team/app.git",
            &config,
        )
        .unwrap();
        assert_eq!(remote.platform, Platform::GitLab);
    }

    #[test]
    fn custom_platform_overrides_private_gitlab_convention() {
        let config = UserConfig {
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "gitlab-work-items".to_string(),
                host_patterns: vec!["gitlab.example.com".to_string()],
                url_template: "https://{host}/{project}/-/work_items/{id}".to_string(),
            }],
            ..UserConfig::default()
        };

        let remote = parse_remote(
            "origin",
            "https://gitlab.example.com/devops/app.git",
            &config,
        )
        .unwrap();

        assert_eq!(
            remote.platform,
            Platform::Custom("gitlab-work-items".to_string())
        );
    }

    #[test]
    fn rejects_unparseable_remote_urls() {
        let config = UserConfig::default();
        let err = parse_remote("origin", "not-a-remote", &config).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::RemoteParseFailed(value) if value == "not-a-remote")
        );
    }

    #[test]
    fn rejects_supported_shape_with_invalid_details() {
        let config = UserConfig::default();

        for remote_url in [
            "ftp://github.com/owner/repo.git",
            "https://github.com",
            "https://github.com/",
            "git@github.com:",
        ] {
            let err = parse_remote("origin", remote_url, &config).unwrap_err();
            assert!(
                matches!(err, IssueJumperError::RemoteParseFailed(value) if value == remote_url)
            );
        }
    }
}
