use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{IssueJumperError, Result};

mod lint;
#[cfg(test)]
mod tests;

pub(crate) use lint::lint_config;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserConfig {
    pub fallback_platform: Option<String>,
    pub redmine_base_url: Option<String>,
    #[serde(default)]
    pub clear_inherited_config: bool,
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
        if override_config.clear_inherited_config {
            self = UserConfig::default();
        }
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
            let override_names: HashSet<String> = override_config
                .custom_platforms
                .iter()
                .map(|platform| normalized_platform_name(&platform.name))
                .collect();
            self.custom_platforms.retain(|platform| {
                !override_names.contains(&normalized_platform_name(&platform.name))
            });
            let mut custom_platforms = override_config.custom_platforms;
            custom_platforms.extend(self.custom_platforms);
            self.custom_platforms = custom_platforms;
        }
        self.clear_inherited_config = false;
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

#[derive(Debug, Default)]
pub struct ConfigLintReport {
    pub checked_paths: Vec<PathBuf>,
    pub errors: Vec<ConfigLintError>,
}

impl ConfigLintReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigLintError {
    pub path: Option<PathBuf>,
    pub message: String,
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

fn normalized_platform_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
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
