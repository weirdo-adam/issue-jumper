use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::{
    ConfigLintError, ConfigLintReport, CustomPlatform, RawIssueRule, UserConfig,
    global_config_path, load_config_from, normalized_platform_name, project_config_paths,
    read_config,
};

pub(crate) fn lint_config(repo: &Path, explicit_paths: &[PathBuf]) -> ConfigLintReport {
    if explicit_paths.is_empty() {
        lint_discovered_config(repo, global_config_path())
    } else {
        lint_explicit_config_paths(explicit_paths)
    }
}

pub(super) fn lint_discovered_config(
    repo: &Path,
    global_path: Option<PathBuf>,
) -> ConfigLintReport {
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
        let normalized_name = normalized_platform_name(&platform.name);
        if normalized_name.is_empty() {
            push_lint_error(
                path,
                "custom_platforms entries must have a non-empty name",
                report,
            );
        } else if !names.insert(normalized_name) {
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
    let normalized = normalized_platform_name(value);

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

    match template_placeholders(&platform.url_template) {
        Ok(placeholders) => {
            for placeholder in placeholders {
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
        Err(message) => push_lint_error(
            path,
            format!("custom platform `{}` url_template {message}", platform.name),
            report,
        ),
    }
}

fn template_placeholders(template: &str) -> std::result::Result<Vec<String>, &'static str> {
    let mut placeholders = Vec::new();
    let mut index = 0;

    while let Some(start) = template[index..].find('{') {
        let start = index + start;
        if template[index..start].contains('}') {
            return Err("contains an unmatched `}`");
        }

        let after_start = start + 1;
        let Some(end) = template[after_start..].find('}') else {
            return Err("contains an unmatched `{`");
        };
        let end = after_start + end;
        let placeholder = &template[after_start..end];
        if placeholder.contains('{') {
            return Err("contains nested `{` before `}`");
        }
        placeholders.push(placeholder.to_string());
        index = end + 1;
    }

    if template[index..].contains('}') {
        return Err("contains an unmatched `}`");
    }

    Ok(placeholders)
}

fn custom_platform_names(config: &UserConfig) -> HashSet<String> {
    config
        .custom_platforms
        .iter()
        .map(|platform| normalized_platform_name(&platform.name))
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
