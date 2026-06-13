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
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub issue_rules: Vec<RawIssueRule>,
    #[serde(default)]
    pub custom_platforms: Vec<CustomPlatform>,
}

impl UserConfig {
    fn merge_override(mut self, override_config: UserConfig) -> Self {
        if override_config.fallback_platform.is_some() {
            self.fallback_platform = override_config.fallback_platform;
        }
        if override_config.redmine_base_url.is_some() {
            self.redmine_base_url = override_config.redmine_base_url;
        }
        if !override_config.disabled_default_rules.is_empty() {
            self.disabled_default_rules
                .extend(override_config.disabled_default_rules);
        }
        if !override_config.disabled_rules.is_empty() {
            self.disabled_rules.extend(override_config.disabled_rules);
        }
        if !override_config.issue_rules.is_empty() {
            let override_names: Vec<&str> = override_config
                .issue_rules
                .iter()
                .map(|rule| rule.name.as_str())
                .collect();
            self.issue_rules
                .retain(|rule| !override_names.contains(&rule.name.as_str()));
            let mut issue_rules = override_config.issue_rules;
            issue_rules.extend(self.issue_rules);
            self.issue_rules = issue_rules;
        }
        if !override_config.custom_platforms.is_empty() {
            let override_names: Vec<&str> = override_config
                .custom_platforms
                .iter()
                .map(|platform| platform.name.as_str())
                .collect();
            self.custom_platforms
                .retain(|platform| !override_names.contains(&platform.name.as_str()));
            let mut custom_platforms = override_config.custom_platforms;
            custom_platforms.extend(self.custom_platforms);
            self.custom_platforms = custom_platforms;
        }
        self
    }
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
    load_config_from(repo, global_config_path())
}

fn load_config_from(repo: &Path, global_path: Option<PathBuf>) -> Result<UserConfig> {
    let mut config = if let Some(path) = global_path.filter(|path| path.exists()) {
        read_config(&path)?
    } else {
        UserConfig::default()
    };

    for path in project_config_paths(repo) {
        if path.exists() {
            config = config.merge_override(read_config(&path)?);
            break;
        }
    }

    Ok(config)
}

fn read_config(path: &Path) -> Result<UserConfig> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| IssueJumperError::InvalidConfig(format!("{}: {err}", path.display())))
}

fn project_config_paths(repo: &Path) -> [PathBuf; 2] {
    [
        repo.join(".zed").join("issue-jumper.json"),
        repo.join(".issue-jumper.json"),
    ]
}

fn global_config_path() -> Option<PathBuf> {
    #[cfg(windows)]
    if let Some(path) = std::env::var_os("APPDATA") {
        return Some(PathBuf::from(path).join("issue-jumper").join("config.json"));
    }

    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path).join("issue-jumper").join("config.json"));
    }

    std::env::var_os("HOME").map(|path| {
        PathBuf::from(path)
            .join(".config")
            .join("issue-jumper")
            .join("config.json")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_default_when_config_is_missing() {
        let dir = temp_dir("missing");
        fs::create_dir_all(&dir).unwrap();

        let config = load_config_from(&dir, None).unwrap();

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

        let config = load_config_from(&dir, None).unwrap();

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

        let err = load_config_from(&dir, None).unwrap_err();

        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains("unknown field") && message.contains("default_platform"))
        );
    }

    #[test]
    fn rejects_invalid_json() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".issue-jumper.json"), "{").unwrap();

        let err = load_config_from(&dir, None).unwrap_err();

        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains(".issue-jumper.json"))
        );
    }

    #[test]
    fn returns_expected_config_paths() {
        let dir = PathBuf::from("/repo");
        let paths = project_config_paths(&dir);

        assert_eq!(paths[0], PathBuf::from("/repo/.zed/issue-jumper.json"));
        assert_eq!(paths[1], PathBuf::from("/repo/.issue-jumper.json"));
    }

    #[test]
    fn merges_global_config_with_project_override() {
        let dir = temp_dir("merge");
        let global = temp_dir("global-config").join("config.json");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(global.parent().unwrap()).unwrap();
        fs::write(
            &global,
            r#"{
              "redmine_base_url": "https://redmine.global.example.com",
              "issue_rules": [
                {
                  "name": "shared-redmine",
                  "pattern": "global-(?P<id>\\d+)",
                  "platform": "redmine"
                },
                {
                  "name": "disabled-global-redmine",
                  "pattern": "disabled-(?P<id>\\d+)",
                  "platform": "redmine"
                }
              ]
            }"#,
        )
        .unwrap();
        fs::write(
            dir.join(".issue-jumper.json"),
            r#"{
              "redmine_base_url": "https://redmine.project.example.com",
              "disabled_rules": ["disabled-global-redmine"],
              "issue_rules": [
                {
                  "name": "shared-redmine",
                  "pattern": "project-(?P<id>\\d+)",
                  "platform": "redmine"
                }
              ]
            }"#,
        )
        .unwrap();

        let config = load_config_from(&dir, Some(global)).unwrap();

        assert_eq!(
            config.redmine_base_url.as_deref(),
            Some("https://redmine.project.example.com")
        );
        assert_eq!(config.issue_rules.len(), 2);
        assert_eq!(config.issue_rules[0].name, "shared-redmine");
        assert_eq!(config.issue_rules[0].pattern, "project-(?P<id>\\d+)");
        assert_eq!(config.issue_rules[1].name, "disabled-global-redmine");
        assert_eq!(
            config.disabled_rules,
            vec!["disabled-global-redmine".to_string()]
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-config-{label}-{nonce}"))
    }
}
