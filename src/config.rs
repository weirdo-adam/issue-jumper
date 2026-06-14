use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
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

pub(crate) fn lint_config(repo: &Path, explicit_paths: &[PathBuf]) -> ConfigLintReport {
    if explicit_paths.is_empty() {
        lint_discovered_config(repo, global_config_path())
    } else {
        lint_explicit_config_paths(explicit_paths)
    }
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

fn lint_discovered_config(repo: &Path, global_path: Option<PathBuf>) -> ConfigLintReport {
    let mut report = ConfigLintReport::default();
    let paths = discovered_config_paths(repo, global_path.clone());
    let mut parsed_configs = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }
        report.checked_paths.push(path.clone());

        match read_config(&path) {
            Ok(config) => parsed_configs.push((path, config)),
            Err(err) => {
                report.errors.push(ConfigLintError {
                    path: Some(path),
                    message: err.to_string(),
                });
            }
        }
    }

    let discovered_custom_platforms = parsed_configs
        .iter()
        .flat_map(|(_, config)| custom_platform_names(config))
        .collect();

    for (path, config) in &parsed_configs {
        lint_config_values(
            Some(path),
            config,
            &discovered_custom_platforms,
            &mut report,
        );
    }

    if report.is_ok()
        && !report.checked_paths.is_empty()
        && let Ok(config) = load_config_from(repo, global_path)
    {
        lint_config_values(None, &config, &custom_platform_names(&config), &mut report);
    }

    report
}

fn lint_explicit_config_paths(paths: &[PathBuf]) -> ConfigLintReport {
    let mut report = ConfigLintReport::default();

    for path in paths {
        lint_single_config_path(path, true, &mut report);
    }

    report
}

fn lint_single_config_path(path: &Path, require_exists: bool, report: &mut ConfigLintReport) {
    if !path.exists() {
        if require_exists {
            report.errors.push(ConfigLintError {
                path: Some(path.to_path_buf()),
                message: "file does not exist".to_string(),
            });
        }
        return;
    }

    report.checked_paths.push(path.to_path_buf());

    match read_config(path) {
        Ok(config) => {
            let custom_platforms = custom_platform_names(&config);
            lint_config_values(Some(path), &config, &custom_platforms, report);
        }
        Err(err) => {
            report.errors.push(ConfigLintError {
                path: Some(path.to_path_buf()),
                message: err.to_string(),
            });
        }
    }
}

fn lint_config_values(
    path: Option<&Path>,
    config: &UserConfig,
    custom_platforms: &HashSet<String>,
    report: &mut ConfigLintReport,
) {
    lint_platform_reference(
        path,
        "fallback_platform",
        config.fallback_platform.as_deref(),
        custom_platforms,
        report,
    );
    lint_optional_http_url(
        path,
        "redmine_base_url",
        config.redmine_base_url.as_deref(),
        report,
    );
    lint_string_list(
        path,
        "disabled_default_rules",
        &config.disabled_default_rules,
        report,
    );
    lint_string_list(path, "disabled_rules", &config.disabled_rules, report);
    lint_issue_rules(path, &config.issue_rules, custom_platforms, report);
    lint_custom_platforms(path, &config.custom_platforms, report);
}

fn lint_issue_rules(
    path: Option<&Path>,
    rules: &[RawIssueRule],
    custom_platforms: &HashSet<String>,
    report: &mut ConfigLintReport,
) {
    let mut names = HashSet::new();

    for rule in rules {
        if rule.name.trim().is_empty() {
            push_lint_error(
                path,
                "issue_rules entries must have a non-empty name",
                report,
            );
        } else if !names.insert(rule.name.as_str()) {
            push_lint_error(
                path,
                format!("issue rule `{}` is defined more than once", rule.name),
                report,
            );
        }

        if rule.pattern.trim().is_empty() {
            push_lint_error(
                path,
                format!("issue rule `{}` must have a non-empty pattern", rule.name),
                report,
            );
        } else {
            match Regex::new(&rule.pattern) {
                Ok(pattern) => {
                    if !pattern.capture_names().any(|name| name == Some("id")) {
                        push_lint_error(
                            path,
                            format!(
                                "issue rule `{}` must include named capture group `id`",
                                rule.name
                            ),
                            report,
                        );
                    }
                }
                Err(err) => push_lint_error(
                    path,
                    format!("issue rule `{}` has invalid regex: {err}", rule.name),
                    report,
                ),
            }
        }

        lint_platform_reference(
            path,
            format!("issue rule `{}` platform", rule.name),
            rule.platform.as_deref(),
            custom_platforms,
            report,
        );
    }
}

fn lint_custom_platforms(
    path: Option<&Path>,
    platforms: &[CustomPlatform],
    report: &mut ConfigLintReport,
) {
    let mut names = HashSet::new();

    for platform in platforms {
        if platform.name.trim().is_empty() {
            push_lint_error(
                path,
                "custom_platforms entries must have a non-empty name",
                report,
            );
        } else if !names.insert(platform.name.as_str()) {
            push_lint_error(
                path,
                format!(
                    "custom platform `{}` is defined more than once",
                    platform.name
                ),
                report,
            );
        }

        lint_string_list(
            path,
            format!("custom platform `{}` host_patterns", platform.name),
            &platform.host_patterns,
            report,
        );
        lint_url_template(path, platform, report);
    }
}

fn lint_string_list(
    path: Option<&Path>,
    field: impl AsRef<str>,
    values: &[String],
    report: &mut ConfigLintReport,
) {
    let field = field.as_ref();
    for value in values {
        if value.trim().is_empty() {
            push_lint_error(
                path,
                format!("{field} must not contain empty values"),
                report,
            );
        }
    }
}

fn lint_platform_reference(
    path: Option<&Path>,
    field: impl AsRef<str>,
    value: Option<&str>,
    custom_platforms: &HashSet<String>,
    report: &mut ConfigLintReport,
) {
    let Some(value) = value else {
        return;
    };
    let normalized = value.trim().to_ascii_lowercase();

    if normalized.is_empty() {
        push_lint_error(
            path,
            format!("{} must not be empty", field.as_ref()),
            report,
        );
    } else if !is_builtin_platform(&normalized) && !custom_platforms.contains(&normalized) {
        push_lint_error(
            path,
            format!("{} references unknown platform `{value}`", field.as_ref()),
            report,
        );
    }
}

fn lint_optional_http_url(
    path: Option<&Path>,
    field: &str,
    value: Option<&str>,
    report: &mut ConfigLintReport,
) {
    let Some(value) = value else {
        return;
    };
    if !value.starts_with("http://") && !value.starts_with("https://") {
        push_lint_error(
            path,
            format!("{field} must start with http:// or https://"),
            report,
        );
    }
}

fn lint_url_template(
    path: Option<&Path>,
    platform: &CustomPlatform,
    report: &mut ConfigLintReport,
) {
    if platform.url_template.trim().is_empty() {
        push_lint_error(
            path,
            format!(
                "custom platform `{}` must have a non-empty url_template",
                platform.name
            ),
            report,
        );
        return;
    }

    if !platform.url_template.contains("{id}") {
        push_lint_error(
            path,
            format!(
                "custom platform `{}` url_template must include `{{id}}`",
                platform.name
            ),
            report,
        );
    }

    if !platform.url_template.starts_with("http://")
        && !platform.url_template.starts_with("https://")
        && !platform.url_template.starts_with("{redmine_base_url}")
    {
        push_lint_error(
            path,
            format!(
                "custom platform `{}` url_template must start with http://, https://, or {{redmine_base_url}}",
                platform.name
            ),
            report,
        );
    }

    for placeholder in template_placeholders(&platform.url_template) {
        if !matches!(
            placeholder.as_str(),
            "id" | "host" | "owner" | "repo" | "project" | "redmine_base_url"
        ) {
            push_lint_error(
                path,
                format!(
                    "custom platform `{}` url_template uses unsupported placeholder `{{{placeholder}}}`",
                    platform.name
                ),
                report,
            );
        }
    }
}

fn template_placeholders(template: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('}') else {
            placeholders.push(after_start.to_string());
            break;
        };
        placeholders.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }

    placeholders
}

fn custom_platform_names(config: &UserConfig) -> HashSet<String> {
    config
        .custom_platforms
        .iter()
        .map(|platform| platform.name.to_ascii_lowercase())
        .collect()
}

fn is_builtin_platform(value: &str) -> bool {
    matches!(
        value,
        "github" | "gitlab" | "bitbucket" | "gitee" | "redmine"
    )
}

fn push_lint_error(path: Option<&Path>, message: impl Into<String>, report: &mut ConfigLintReport) {
    report.errors.push(ConfigLintError {
        path: path.map(Path::to_path_buf),
        message: message.into(),
    });
}

fn discovered_config_paths(repo: &Path, global_path: Option<PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = global_path {
        paths.push(path);
    }
    paths.extend(project_config_paths(repo));
    paths
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

    #[test]
    fn lint_accepts_valid_explicit_config() {
        let dir = temp_dir("lint-valid");
        let path = dir.join("config.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &path,
            r#"{
              "fallback_platform": "jira",
              "issue_rules": [
                {
                  "name": "jira-ticket",
                  "pattern": "(?P<id>[A-Z]+-\\d+)",
                  "platform": "jira"
                }
              ],
              "custom_platforms": [
                {
                  "name": "jira",
                  "host_patterns": ["jira.example.com"],
                  "url_template": "https://jira.example.com/browse/{id}"
                }
              ]
            }"#,
        )
        .unwrap();

        let report = lint_config(&dir, &[path]);

        assert!(report.is_ok(), "{:?}", report.errors);
        assert_eq!(report.checked_paths.len(), 1);
    }

    #[test]
    fn lint_rejects_invalid_rule_regex() {
        let dir = temp_dir("lint-regex");
        let path = dir.join("config.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &path,
            r#"{
              "issue_rules": [
                {
                  "name": "bad-regex",
                  "pattern": "(?P<id>"
                }
              ]
            }"#,
        )
        .unwrap();

        let report = lint_config(&dir, &[path]);

        assert!(!report.is_ok());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.message.contains("invalid regex"))
        );
    }

    #[test]
    fn lint_rejects_unknown_platform_reference() {
        let dir = temp_dir("lint-platform");
        let path = dir.join("config.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &path,
            r#"{
              "fallback_platform": "missing"
            }"#,
        )
        .unwrap();

        let report = lint_config(&dir, &[path]);

        assert!(!report.is_ok());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.message.contains("unknown platform"))
        );
    }

    #[test]
    fn lint_allows_project_rule_to_use_global_custom_platform() {
        let home = temp_dir("lint-home");
        let repo = temp_dir("lint-repo");
        let global = home.join("issue-jumper").join("config.json");
        fs::create_dir_all(global.parent().unwrap()).unwrap();
        fs::create_dir_all(&repo).unwrap();
        fs::write(
            &global,
            r#"{
              "custom_platforms": [
                {
                  "name": "jira",
                  "host_patterns": ["jira.example.com"],
                  "url_template": "https://jira.example.com/browse/{id}"
                }
              ]
            }"#,
        )
        .unwrap();
        fs::write(
            repo.join(".issue-jumper.json"),
            r#"{
              "issue_rules": [
                {
                  "name": "jira-ticket",
                  "pattern": "(?P<id>[A-Z]+-\\d+)",
                  "platform": "jira"
                }
              ]
            }"#,
        )
        .unwrap();

        let report = lint_discovered_config(&repo, Some(global));

        assert!(report.is_ok(), "{:?}", report.errors);
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("issue-jumper-config-{label}-{nonce}"))
    }
}
