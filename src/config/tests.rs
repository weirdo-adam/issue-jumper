use super::lint::lint_discovered_config;
use super::*;
use std::fs;
use std::path::PathBuf;
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
fn project_config_can_clear_inherited_config() {
    let dir = temp_dir("clear-inherited");
    let global = temp_dir("global-clear-config").join("config.json");
    fs::create_dir_all(&dir).unwrap();
    fs::create_dir_all(global.parent().unwrap()).unwrap();
    fs::write(
        &global,
        r#"{
          "fallback_platform": "redmine",
          "redmine_base_url": "https://redmine.global.example.com",
          "issue_rules": [
            {
              "name": "global-redmine",
              "pattern": "global-(?P<id>\\d+)",
              "platform": "redmine"
            }
          ],
          "custom_platforms": [
            {
              "name": "jira",
              "url_template": "https://jira.example.com/browse/{id}"
            }
          ]
        }"#,
    )
    .unwrap();
    fs::write(
        dir.join(".issue-jumper.json"),
        r#"{
          "clear_inherited_config": true,
          "fallback_platform": "github"
        }"#,
    )
    .unwrap();

    let config = load_config_from(&dir, Some(global)).unwrap();

    assert_eq!(config.fallback_platform.as_deref(), Some("github"));
    assert!(config.redmine_base_url.is_none());
    assert!(config.issue_rules.is_empty());
    assert!(config.custom_platforms.is_empty());
    assert!(!config.clear_inherited_config);
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
fn lint_rejects_duplicate_custom_platform_names_ignoring_case() {
    let dir = temp_dir("lint-platform-case");
    let path = dir.join("config.json");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        &path,
        r#"{
          "custom_platforms": [
            {
              "name": "Jira",
              "url_template": "https://jira.example.com/browse/{id}"
            },
            {
              "name": "jira",
              "url_template": "https://jira-alt.example.com/browse/{id}"
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
            .any(|error| error.message.contains("defined more than once"))
    );
}

#[test]
fn lint_rejects_unmatched_url_template_braces() {
    let dir = temp_dir("lint-template-braces");
    let path = dir.join("config.json");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        &path,
        r#"{
          "custom_platforms": [
            {
              "name": "jira",
              "url_template": "https://jira.example.com/browse/{id}}"
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
            .any(|error| error.message.contains("unmatched `}`"))
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
