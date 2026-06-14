use regex::Regex;
use std::collections::HashSet;

use crate::config::{RawIssueRule, UserConfig};
use crate::error::{IssueJumperError, Result};
use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct IssueRule {
    pub name: String,
    pub pattern: Regex,
    pub platform_hint: Option<Platform>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueMatch {
    pub issue_id: String,
    pub rule_name: String,
    pub platform_hint: Option<Platform>,
}

pub fn extract_issue(
    branch: &str,
    config: &UserConfig,
    rule_name: Option<&str>,
) -> Result<IssueMatch> {
    let rules = build_rules(config)?;
    let active_rules: Vec<&IssueRule> = if let Some(name) = rule_name {
        rules.iter().filter(|rule| rule.name == name).collect()
    } else {
        rules.iter().collect()
    };

    if active_rules.is_empty()
        && let Some(name) = rule_name
    {
        return Err(IssueJumperError::UnknownRule(name.to_string()));
    }

    for rule in active_rules {
        if let Some(captures) = rule.pattern.captures(branch)
            && let Some(value) = captures.name("id")
        {
            let issue_id = value.as_str().trim().to_string();
            if !issue_id.is_empty() {
                return Ok(IssueMatch {
                    issue_id,
                    rule_name: rule.name.clone(),
                    platform_hint: rule.platform_hint.clone(),
                });
            }
        }
    }

    Err(IssueJumperError::NoMatchingRule(branch.to_string()))
}

fn build_rules(config: &UserConfig) -> Result<Vec<IssueRule>> {
    let mut rules = Vec::new();
    let disabled_rules: HashSet<&str> = config.disabled_rules.iter().map(String::as_str).collect();

    for raw in &config.issue_rules {
        if !disabled_rules.contains(raw.name.as_str()) {
            rules.push(compile_raw_rule(raw)?);
        }
    }

    let disabled: HashSet<&str> = config
        .disabled_default_rules
        .iter()
        .map(String::as_str)
        .collect();

    for raw in default_rules() {
        if !disabled.contains(raw.name.as_str()) {
            rules.push(compile_raw_rule(&raw)?);
        }
    }

    Ok(rules)
}

fn compile_raw_rule(raw: &RawIssueRule) -> Result<IssueRule> {
    let pattern = Regex::new(&raw.pattern).map_err(|err| {
        IssueJumperError::InvalidConfig(format!("rule `{}` has invalid regex: {err}", raw.name))
    })?;

    if !pattern.capture_names().any(|name| name == Some("id")) {
        return Err(IssueJumperError::InvalidConfig(format!(
            "rule `{}` must include a named capture group `id`",
            raw.name
        )));
    }

    Ok(IssueRule {
        name: raw.name.clone(),
        pattern,
        platform_hint: raw.platform.as_deref().map(Platform::parse),
    })
}

fn default_rules() -> Vec<RawIssueRule> {
    vec![
        raw_rule("github-gh-prefix", r"(?i)\bGH[-_]?(?P<id>\d+)\b"),
        raw_rule("issue-prefix", r"(?i)\bissue[-_]?(?P<id>\d+)\b"),
        raw_rule("hash-number", r"#(?P<id>\d+)\b"),
        raw_rule("leading-number", r"^(?P<id>\d+)[-_]"),
        raw_rule("jira-like-key", r"\b(?P<id>[A-Z][A-Z0-9]+-\d+)\b"),
        raw_rule("trailing-number", r"[-_/](?P<id>\d+)$"),
    ]
}

fn raw_rule(name: &str, pattern: &str) -> RawIssueRule {
    RawIssueRule {
        name: name.to_string(),
        pattern: pattern.to_string(),
        platform: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_default_patterns() {
        let config = UserConfig::default();
        assert_eq!(
            extract_issue("feature/GH-123", &config, None)
                .unwrap()
                .issue_id,
            "123"
        );
        assert_eq!(
            extract_issue("fix/issue-456", &config, None)
                .unwrap()
                .issue_id,
            "456"
        );
        assert_eq!(
            extract_issue("bug/#789", &config, None).unwrap().issue_id,
            "789"
        );
        assert_eq!(
            extract_issue("101-add-login", &config, None)
                .unwrap()
                .issue_id,
            "101"
        );
        assert_eq!(
            extract_issue("feature/ABC-456-login", &config, None)
                .unwrap()
                .issue_id,
            "ABC-456"
        );
        assert_eq!(
            extract_issue("feature/login-789", &config, None)
                .unwrap()
                .issue_id,
            "789"
        );
    }

    #[test]
    fn supports_custom_rules_rule_filtering_and_disabled_defaults() {
        let config = UserConfig {
            disabled_default_rules: vec!["trailing-number".to_string()],
            disabled_rules: vec!["disabled-redmine".to_string()],
            issue_rules: vec![
                RawIssueRule {
                    name: "redmine".to_string(),
                    pattern: r"(?i)redmine[-_](?P<id>\d+)".to_string(),
                    platform: Some("redmine".to_string()),
                },
                RawIssueRule {
                    name: "disabled-redmine".to_string(),
                    pattern: r"(?i)disabled[-_](?P<id>\d+)".to_string(),
                    platform: Some("redmine".to_string()),
                },
            ],
            ..UserConfig::default()
        };

        let matched = extract_issue("feature/redmine-42", &config, Some("redmine")).unwrap();
        assert_eq!(matched.issue_id, "42");
        assert_eq!(matched.rule_name, "redmine");
        assert_eq!(matched.platform_hint, Some(Platform::Redmine));

        let err = extract_issue("feature/login-789", &config, None).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::NoMatchingRule(branch) if branch == "feature/login-789")
        );

        let err = extract_issue("feature/disabled-42", &config, None).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::NoMatchingRule(branch) if branch == "feature/disabled-42")
        );
    }

    #[test]
    fn reports_unknown_rule_filter() {
        let config = UserConfig::default();

        let err = extract_issue("feature/GH-123", &config, Some("missing-rule")).unwrap_err();

        assert!(matches!(err, IssueJumperError::UnknownRule(rule) if rule == "missing-rule"));
    }

    #[test]
    fn rejects_invalid_custom_rule_regex() {
        let config = UserConfig {
            issue_rules: vec![RawIssueRule {
                name: "bad".to_string(),
                pattern: "(".to_string(),
                platform: None,
            }],
            ..UserConfig::default()
        };

        let err = extract_issue("feature/bad", &config, None).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains("invalid regex"))
        );
    }

    #[test]
    fn rejects_custom_rule_without_id_capture() {
        let config = UserConfig {
            issue_rules: vec![RawIssueRule {
                name: "missing-id".to_string(),
                pattern: r"issue-\d+".to_string(),
                platform: None,
            }],
            ..UserConfig::default()
        };

        let err = extract_issue("feature/issue-42", &config, None).unwrap_err();
        assert!(
            matches!(err, IssueJumperError::InvalidConfig(message) if message.contains("named capture group"))
        );
    }

    #[test]
    fn ignores_empty_trimmed_capture() {
        let config = UserConfig {
            disabled_default_rules: vec![
                "github-gh-prefix".to_string(),
                "issue-prefix".to_string(),
                "hash-number".to_string(),
                "leading-number".to_string(),
                "jira-like-key".to_string(),
                "trailing-number".to_string(),
            ],
            issue_rules: vec![RawIssueRule {
                name: "blank".to_string(),
                pattern: r"blank(?P<id>\s+)".to_string(),
                platform: None,
            }],
            ..UserConfig::default()
        };

        let err = extract_issue("blank   ", &config, None).unwrap_err();
        assert!(matches!(err, IssueJumperError::NoMatchingRule(_)));
    }
}
