use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{IssueJumperError, Result};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserConfig {
    pub fallback_platform: Option<String>,
    pub redmine_base_url: Option<String>,
    #[serde(default)]
    pub disabled_default_rules: Vec<String>,
    #[serde(default)]
    pub issue_rules: Vec<RawIssueRule>,
    #[serde(default)]
    pub custom_platforms: Vec<CustomPlatform>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawIssueRule {
    pub name: String,
    pub pattern: String,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomPlatform {
    pub name: String,
    #[serde(default)]
    pub host_patterns: Vec<String>,
    pub url_template: String,
}

pub fn load_config(repo: &Path) -> Result<UserConfig> {
    for path in config_paths(repo) {
        if path.exists() {
            let text = fs::read_to_string(&path)?;
            return serde_json::from_str(&text).map_err(|err| {
                IssueJumperError::InvalidConfig(format!("{}: {err}", path.display()))
            });
        }
    }

    Ok(UserConfig::default())
}

fn config_paths(repo: &Path) -> [PathBuf; 2] {
    [
        repo.join(".zed").join("issue-jumper.json"),
        repo.join(".issue-jumper.json"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_default_when_config_is_missing() {
        let dir = temp_dir("missing");
        fs::create_dir_all(&dir).unwrap();

        let config = load_config(&dir).unwrap();

        assert!(config.fallback_platform.is_none());
        assert!(config.issue_rules.is_empty());
    }

    #[test]
    fn prefers_zed_config_over_root_config() {
        let dir = temp_dir("priority");
        fs::create_dir_all(dir.join(".zed")).unwrap();
        fs::write(
            dir.join(".issue-jumper.json"),
            r#"{"fallback_platform":"github"}"#,
        )
        .unwrap();
        fs::write(
            dir.join(".zed").join("issue-jumper.json"),
            r#"{"fallback_platform":"redmine","redmine_base_url":"https://redmine.example.com"}"#,
        )
        .unwrap();

        let config = load_config(&dir).unwrap();

        assert_eq!(config.fallback_platform.as_deref(), Some("redmine"));
        assert_eq!(
            config.redmine_base_url.as_deref(),
            Some("https://redmine.example.com")
        );
    }

    #[test]
    fn rejects_unknown_config_fields() {
        let dir = temp_dir("unknown-field");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(".issue-jumper.json"),
            r#"{"default_platform":"redmine"}"#,
        )
        .unwrap();

        let err = load_config(&dir).unwrap_err();

        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains("unknown field") && message.contains("default_platform"))
        );
    }

    #[test]
    fn rejects_invalid_json() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".issue-jumper.json"), "{").unwrap();

        let err = load_config(&dir).unwrap_err();

        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains(".issue-jumper.json"))
        );
    }

    #[test]
    fn returns_expected_config_paths() {
        let dir = PathBuf::from("/repo");
        let paths = config_paths(&dir);

        assert_eq!(paths[0], PathBuf::from("/repo/.zed/issue-jumper.json"));
        assert_eq!(paths[1], PathBuf::from("/repo/.issue-jumper.json"));
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-config-{label}-{nonce}"))
    }
}
