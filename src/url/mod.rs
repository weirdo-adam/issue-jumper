use crate::config::UserConfig;
use crate::error::{IssueJumperError, Result};
use crate::git::remote::RemoteInfo;
use crate::platform::Platform;

pub fn build_issue_url(
    issue_id: &str,
    platform: &Platform,
    remote: Option<&RemoteInfo>,
    config: &UserConfig,
) -> Result<String> {
    let url = match platform {
        Platform::GitHub => {
            let remote = required_remote(remote, "GitHub")?;
            format!(
                "https://github.com/{}/{}/issues/{}",
                required(remote.owner.as_deref(), "GitHub owner")?,
                required(remote.repo.as_deref(), "GitHub repo")?,
                encode_segment(issue_id)
            )
        }
        Platform::GitLab => {
            let remote = required_remote(remote, "GitLab")?;
            format!(
                "https://{}/{}/-/issues/{}",
                remote.host,
                encode_path(required(remote.project.as_deref(), "GitLab project")?),
                encode_segment(issue_id)
            )
        }
        Platform::Bitbucket => {
            let remote = required_remote(remote, "Bitbucket")?;
            format!(
                "https://bitbucket.org/{}/{}/issues/{}",
                required(remote.owner.as_deref(), "Bitbucket owner")?,
                required(remote.repo.as_deref(), "Bitbucket repo")?,
                encode_segment(issue_id)
            )
        }
        Platform::Gitee => {
            let remote = required_remote(remote, "Gitee")?;
            format!(
                "https://gitee.com/{}/{}/issues/{}",
                required(remote.owner.as_deref(), "Gitee owner")?,
                required(remote.repo.as_deref(), "Gitee repo")?,
                encode_segment(issue_id)
            )
        }
        Platform::Redmine => {
            let base = required(
                config.redmine_base_url.as_deref(),
                "Redmine requires redmine_base_url",
            )?;
            format!(
                "{}/issues/{}",
                base.trim_end_matches('/'),
                encode_segment(issue_id)
            )
        }
        Platform::Custom(name) => {
            let platform = config
                .custom_platforms
                .iter()
                .find(|platform| platform.name.trim().eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    IssueJumperError::UrlBuildFailed(format!(
                        "custom platform `{name}` requires a custom_platforms entry"
                    ))
                })?;
            apply_template(&platform.url_template, issue_id, remote, config)?
        }
    };

    validate_http_url(&url)?;
    Ok(url)
}

pub fn build_repository_url(platform: &Platform, remote: Option<&RemoteInfo>) -> Result<String> {
    let url = match platform {
        Platform::GitHub => {
            let remote = required_remote(remote, "GitHub")?;
            format!(
                "https://github.com/{}/{}",
                encode_segment(required(remote.owner.as_deref(), "GitHub owner")?),
                encode_segment(required(remote.repo.as_deref(), "GitHub repo")?)
            )
        }
        Platform::GitLab => {
            let remote = required_remote(remote, "GitLab")?;
            format!(
                "https://{}/{}",
                remote.host,
                encode_path(required(remote.project.as_deref(), "GitLab project")?)
            )
        }
        _ => {
            return Err(IssueJumperError::UrlBuildFailed(format!(
                "Cannot build repository URL for {}",
                platform.name()
            )));
        }
    };

    validate_http_url(&url)?;
    Ok(url)
}

pub fn resolve_platform(
    platform_override: Option<Platform>,
    rule_platform: Option<Platform>,
    remote: Option<&RemoteInfo>,
    config: &UserConfig,
) -> Result<Platform> {
    if let Some(platform) = platform_override {
        return Ok(platform);
    }
    if let Some(platform) = rule_platform {
        return Ok(platform);
    }
    if let Some(remote) = remote {
        return Ok(remote.platform.clone());
    }
    if let Some(fallback_platform) = config.fallback_platform.as_deref() {
        return Ok(Platform::parse(fallback_platform));
    }
    Err(IssueJumperError::PlatformUnresolved)
}

fn apply_template(
    template: &str,
    issue_id: &str,
    remote: Option<&RemoteInfo>,
    config: &UserConfig,
) -> Result<String> {
    let mut result = template.to_string();
    result = result.replace("{id}", &encode_segment(issue_id));
    if let Some(remote) = remote {
        result = result.replace("{host}", &remote.host);
        if let Some(owner) = &remote.owner {
            result = result.replace("{owner}", &encode_segment(owner));
        }
        if let Some(repo) = &remote.repo {
            result = result.replace("{repo}", &encode_segment(repo));
        }
        if let Some(project) = &remote.project {
            result = result.replace("{project}", &encode_path(project));
        }
    }
    if let Some(base) = &config.redmine_base_url {
        result = result.replace("{redmine_base_url}", base.trim_end_matches('/'));
    }

    if result.contains('{') || result.contains('}') {
        return Err(IssueJumperError::UrlBuildFailed(format!(
            "unresolved placeholder in template `{template}`"
        )));
    }

    Ok(result)
}

fn required_remote<'a>(remote: Option<&'a RemoteInfo>, platform: &str) -> Result<&'a RemoteInfo> {
    remote.ok_or_else(|| {
        IssueJumperError::UrlBuildFailed(format!("Cannot build {platform} URL without Git remote"))
    })
}

fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str> {
    value.ok_or_else(|| IssueJumperError::UrlBuildFailed(format!("missing {name}")))
}

fn validate_http_url(url: &str) -> Result<()> {
    if url.starts_with("https://") || url.starts_with("http://") {
        Ok(())
    } else {
        Err(IssueJumperError::UrlBuildFailed(format!(
            "URL must start with http:// or https://: {url}"
        )))
    }
}

fn encode_path(value: &str) -> String {
    value
        .split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::remote::parse_remote;

    #[test]
    fn builds_github_url() {
        let config = UserConfig::default();
        let remote = parse_remote("origin", "git@github.com:owner/repo.git", &config).unwrap();
        let url = build_issue_url("123", &Platform::GitHub, Some(&remote), &config).unwrap();
        assert_eq!(url, "https://github.com/owner/repo/issues/123");

        let repo_url = build_repository_url(&Platform::GitHub, Some(&remote)).unwrap();
        assert_eq!(repo_url, "https://github.com/owner/repo");
    }

    #[test]
    fn builds_gitlab_url_with_nested_project() {
        let config = UserConfig::default();
        let remote =
            parse_remote("origin", "git@gitlab.com:group/subgroup/app.git", &config).unwrap();
        let url = build_issue_url("456", &Platform::GitLab, Some(&remote), &config).unwrap();
        assert_eq!(url, "https://gitlab.com/group/subgroup/app/-/issues/456");

        let repo_url = build_repository_url(&Platform::GitLab, Some(&remote)).unwrap();
        assert_eq!(repo_url, "https://gitlab.com/group/subgroup/app");
    }

    #[test]
    fn builds_bitbucket_gitee_and_redmine_urls() {
        let config = UserConfig {
            redmine_base_url: Some("https://redmine.company.com/".to_string()),
            ..UserConfig::default()
        };

        let bitbucket = parse_remote("origin", "git@bitbucket.org:team/app.git", &config).unwrap();
        assert_eq!(
            build_issue_url("7", &Platform::Bitbucket, Some(&bitbucket), &config).unwrap(),
            "https://bitbucket.org/team/app/issues/7"
        );

        let gitee = parse_remote("origin", "git@gitee.com:team/app.git", &config).unwrap();
        assert_eq!(
            build_issue_url("8", &Platform::Gitee, Some(&gitee), &config).unwrap(),
            "https://gitee.com/team/app/issues/8"
        );

        assert_eq!(
            build_issue_url("9", &Platform::Redmine, None, &config).unwrap(),
            "https://redmine.company.com/issues/9"
        );
    }

    #[test]
    fn builds_custom_url_with_template_placeholders_and_encoding() {
        let config = UserConfig {
            redmine_base_url: Some("https://redmine.company.com/".to_string()),
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "custom".to_string(),
                host_patterns: vec!["tracker.example.com".to_string()],
                url_template: "{redmine_base_url}/projects/{project}/owners/{owner}/repos/{repo}/issues/{id}?host={host}".to_string(),
            }],
            ..UserConfig::default()
        };
        let remote = parse_remote(
            "origin",
            "https://tracker.example.com/team/app repo.git",
            &config,
        )
        .unwrap();

        let url = build_issue_url(
            "ABC 1",
            &Platform::Custom("custom".to_string()),
            Some(&remote),
            &config,
        )
        .unwrap();

        assert_eq!(
            url,
            "https://redmine.company.com/projects/team/app%20repo/owners/team/repos/app%20repo/issues/ABC%201?host=tracker.example.com"
        );
    }

    #[test]
    fn resolves_custom_platform_names_case_insensitively() {
        let config = UserConfig {
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "Jira".to_string(),
                host_patterns: Vec::new(),
                url_template: "https://jira.example.com/browse/{id}".to_string(),
            }],
            ..UserConfig::default()
        };

        let url = build_issue_url(
            "ABC-123",
            &Platform::Custom("jira".to_string()),
            None,
            &config,
        )
        .unwrap();

        assert_eq!(url, "https://jira.example.com/browse/ABC-123");
    }

    #[test]
    fn resolves_platform_precedence() {
        let config = UserConfig {
            fallback_platform: Some("redmine".to_string()),
            ..UserConfig::default()
        };
        let remote = parse_remote("origin", "git@github.com:owner/repo.git", &config).unwrap();

        assert_eq!(
            resolve_platform(
                Some(Platform::GitLab),
                Some(Platform::GitHub),
                Some(&remote),
                &config
            )
            .unwrap(),
            Platform::GitLab
        );
        assert_eq!(
            resolve_platform(None, Some(Platform::GitHub), Some(&remote), &config).unwrap(),
            Platform::GitHub
        );
        assert_eq!(
            resolve_platform(None, None, Some(&remote), &config).unwrap(),
            Platform::GitHub
        );
        assert_eq!(
            resolve_platform(None, None, None, &config).unwrap(),
            Platform::Redmine
        );
    }

    #[test]
    fn reports_platform_and_url_build_errors() {
        let config = UserConfig::default();
        let github_error = build_issue_url("1", &Platform::GitHub, None, &config).unwrap_err();
        assert!(
            matches!(github_error, IssueJumperError::UrlBuildFailed(message) if message.contains("without Git remote"))
        );

        let repository_error = build_repository_url(&Platform::Bitbucket, None).unwrap_err();
        assert!(
            matches!(repository_error, IssueJumperError::UrlBuildFailed(message) if message.contains("Bitbucket") || message.contains("bitbucket"))
        );

        let redmine_error = build_issue_url("1", &Platform::Redmine, None, &config).unwrap_err();
        assert!(
            matches!(redmine_error, IssueJumperError::UrlBuildFailed(message) if message.contains("redmine_base_url"))
        );

        let custom_error =
            build_issue_url("1", &Platform::Custom("missing".to_string()), None, &config)
                .unwrap_err();
        assert!(
            matches!(custom_error, IssueJumperError::UrlBuildFailed(message) if message.contains("custom_platforms"))
        );

        let unresolved = resolve_platform(None, None, None, &config).unwrap_err();
        assert!(matches!(unresolved, IssueJumperError::PlatformUnresolved));
    }

    #[test]
    fn rejects_unresolved_template_placeholders_and_non_http_urls() {
        let config = UserConfig {
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "bad-placeholder".to_string(),
                host_patterns: Vec::new(),
                url_template: "https://example.com/{missing}/{id}".to_string(),
            }],
            ..UserConfig::default()
        };

        let err = build_issue_url(
            "1",
            &Platform::Custom("bad-placeholder".to_string()),
            None,
            &config,
        )
        .unwrap_err();
        assert!(
            matches!(err, IssueJumperError::UrlBuildFailed(message) if message.contains("unresolved placeholder"))
        );

        let config = UserConfig {
            custom_platforms: vec![crate::config::CustomPlatform {
                name: "bad-scheme".to_string(),
                host_patterns: Vec::new(),
                url_template: "file:///tmp/{id}".to_string(),
            }],
            ..UserConfig::default()
        };
        let err = build_issue_url(
            "1",
            &Platform::Custom("bad-scheme".to_string()),
            None,
            &config,
        )
        .unwrap_err();
        assert!(
            matches!(err, IssueJumperError::UrlBuildFailed(message) if message.contains("http:// or https://"))
        );
    }
}
