use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn installs_task_and_keymap_into_empty_config_dir() {
    let dir = temp_dir("empty-config");
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["label"], TASK_LABEL);
    assert_eq!(tasks[0]["args"][2], "$ZED_WORKTREE_ROOT");
    assert_eq!(keymaps.len(), 1);
    assert_eq!(keymaps[0]["context"], "Editor || Workspace");
    assert_eq!(keymaps[0]["use_key_equivalents"], true);
    assert_eq!(keymaps[0]["bindings"]["alt-i"][0], "task::Spawn");
}

#[test]
fn refuses_to_overwrite_foreign_key_binding_without_force() {
    let dir = temp_dir("key-conflict");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        r#"[{"context":"Workspace","bindings":{"alt-i":"editor::OpenUrl"}}]"#,
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();
    assert!(matches!(err, IssueJumperError::ZedKeyConflict(key) if key == "alt-i"));
    assert!(!dir.join("tasks.json").exists());
}

#[test]
fn updates_existing_issue_jumper_binding() {
    let dir = temp_dir("update-binding");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        format!(
            r#"[{{"context":"Workspace","bindings":{{"alt-i":["task::Spawn",{{"task_name":"{}"}}]}}}}]"#,
            TASK_LABEL
        ),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("cmd-i");

    install_zed_into_dir(&dir, task, keymap, "cmd-i", false).unwrap();

    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(keymaps.len(), 1);
    assert_eq!(keymaps[0]["context"], "Editor || Workspace");
    assert_eq!(keymaps[0]["use_key_equivalents"], true);
    assert_eq!(keymaps[0]["bindings"]["cmd-i"][1]["task_name"], TASK_LABEL);
}

#[test]
fn updates_existing_workspace_issue_jumper_binding_to_global_context() {
    let dir = temp_dir("update-binding-context");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        format!(
            r#"[{{"context":"Workspace","bindings":{{"alt-i":["task::Spawn",{{"task_name":"{}"}}]}}}}]"#,
            TASK_LABEL
        ),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(keymaps.len(), 1);
    assert_eq!(keymaps[0]["context"], "Editor || Workspace");
    assert_eq!(keymaps[0]["use_key_equivalents"], true);
    assert_eq!(keymaps[0]["bindings"]["alt-i"][1]["task_name"], TASK_LABEL);
}

#[test]
fn preserves_neighbor_bindings_when_moving_issue_jumper_keymap_context() {
    let dir = temp_dir("preserve-neighbor-binding");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        format!(
            r#"[{{"context":"Workspace","bindings":{{"alt-i":["task::Spawn",{{"task_name":"{}"}}],"cmd-k":"workspace::ToggleLeftDock"}}}}]"#,
            TASK_LABEL
        ),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(keymaps.len(), 2);
    assert_eq!(keymaps[0]["context"], "Workspace");
    assert!(keymaps[0]["bindings"].get("alt-i").is_none());
    assert_eq!(keymaps[0]["bindings"]["cmd-k"], "workspace::ToggleLeftDock");
    assert_eq!(keymaps[1]["context"], "Editor || Workspace");
    assert_eq!(keymaps[1]["use_key_equivalents"], true);
    assert_eq!(keymaps[1]["bindings"]["alt-i"][1]["task_name"], TASK_LABEL);
}

#[test]
fn removes_old_issue_jumper_binding_when_installing_new_key() {
    let dir = temp_dir("remove-old-binding");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        format!(
            r#"[{{"context":"Workspace","bindings":{{"alt-i":["task::Spawn",{{"task_name":"{}"}}],"cmd-k":"workspace::ToggleLeftDock"}}}}]"#,
            TASK_LABEL
        ),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("ctrl-i");

    install_zed_into_dir(&dir, task, keymap, "ctrl-i", false).unwrap();

    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(keymaps.len(), 2);
    assert!(keymaps[0]["bindings"].get("alt-i").is_none());
    assert_eq!(keymaps[0]["bindings"]["cmd-k"], "workspace::ToggleLeftDock");
    assert_eq!(keymaps[1]["bindings"]["ctrl-i"][1]["task_name"], TASK_LABEL);
}

#[test]
fn install_zed_prints_templates_without_config_dir() {
    install_zed_with(
        InstallOptions {
            key: "alt-i".to_string(),
            force: false,
            print: true,
        },
        PathBuf::from("/usr/local/bin/issue-jumper"),
        None,
    )
    .unwrap();
}

#[test]
fn install_zed_writes_to_injected_config_dir() {
    let dir = temp_dir("injected-config");

    install_zed_with(
        InstallOptions {
            key: "cmd-i".to_string(),
            force: false,
            print: false,
        },
        PathBuf::from("/usr/local/bin/issue-jumper"),
        Some(dir.clone()),
    )
    .unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(tasks[0]["command"], "/usr/local/bin/issue-jumper");
    assert_eq!(keymaps[0]["bindings"]["cmd-i"][1]["task_name"], TASK_LABEL);
}

#[test]
fn install_zed_from_writes_to_injected_config_dir() {
    fn current_executable() -> Result<PathBuf> {
        Ok(PathBuf::from("/usr/local/bin/issue-jumper"))
    }

    let dir = temp_dir("injected-config-from");

    install_zed_from(
        InstallOptions {
            key: "cmd-i".to_string(),
            force: false,
            print: false,
        },
        current_executable,
        Some(dir.clone()),
    )
    .unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    assert_eq!(tasks[0]["command"], "/usr/local/bin/issue-jumper");
}

#[test]
fn install_zed_requires_config_dir_when_not_printing() {
    let err = install_zed_with(
        InstallOptions {
            key: "alt-i".to_string(),
            force: false,
            print: false,
        },
        PathBuf::from("/usr/local/bin/issue-jumper"),
        None,
    )
    .unwrap_err();

    assert!(matches!(err, IssueJumperError::ZedConfigPathNotFound));
}

#[test]
fn install_zed_reports_current_executable_error() {
    fn missing_executable() -> Result<PathBuf> {
        Err(IssueJumperError::Io("missing executable".to_string()))
    }

    let err = install_zed_from(
        InstallOptions {
            key: "alt-i".to_string(),
            force: false,
            print: false,
        },
        missing_executable,
        None,
    )
    .unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(message) if message == "missing executable"));
}

#[test]
fn install_zed_propagates_directory_creation_error() {
    let path = temp_dir("config-file");
    fs::write(&path, "not a directory").unwrap();

    let err = install_zed_with(
        InstallOptions {
            key: "alt-i".to_string(),
            force: false,
            print: false,
        },
        PathBuf::from("/usr/local/bin/issue-jumper"),
        Some(path),
    )
    .unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn install_zed_propagates_tasks_read_error() {
    let dir = temp_dir("tasks-read-error");
    fs::create_dir_all(dir.join("tasks.json")).unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn install_zed_propagates_keymap_read_error() {
    let dir = temp_dir("keymap-read-error");
    fs::create_dir_all(dir.join("keymap.json")).unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn updates_existing_issue_jumper_task() {
    let dir = temp_dir("update-task");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        format!(r#"[{{"label":"{TASK_LABEL}","command":"/old/issue-jumper","args":[]}}]"#),
    )
    .unwrap();
    let task = task_template(Path::new("/new/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["command"], "/new/issue-jumper");
    assert!(dir.join("tasks.json.bak").exists());
}

#[test]
fn updates_legacy_zed_issue_jumper_task() {
    let dir = temp_dir("update-legacy-task");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        format!(r#"[{{"label":"{TASK_LABEL}","command":"/old/zed-issue-jumper","args":[]}}]"#),
    )
    .unwrap();
    let task = task_template(Path::new("/new/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["command"], "/new/issue-jumper");
}

#[test]
fn updates_existing_task_label_without_command() {
    let dir = temp_dir("task-without-command");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        format!(r#"[{{"label":"{TASK_LABEL}"}}]"#),
    )
    .unwrap();
    let task = task_template(Path::new("/new/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();
    assert_eq!(tasks[0]["command"], "/new/issue-jumper");
}

#[test]
fn refuses_existing_task_label_with_foreign_command() {
    let dir = temp_dir("foreign-task");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        format!(r#"[{{"label":"{TASK_LABEL}","command":"/bin/echo"}}]"#),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();
    assert!(
        matches!(err, IssueJumperError::ZedConfigInvalidJson(message) if message.contains("different command"))
    );
}

#[test]
fn refuses_existing_task_label_with_issue_jumper_substring_command() {
    let dir = temp_dir("foreign-substring-task");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        format!(r#"[{{"label":"{TASK_LABEL}","command":"/bin/not-issue-jumper-wrapper"}}]"#),
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();

    assert!(
        matches!(err, IssueJumperError::ZedConfigInvalidJson(message) if message.contains("different command"))
    );
}

#[test]
fn force_overwrites_foreign_key_binding() {
    let dir = temp_dir("force-key");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("keymap.json"),
        r#"[{"context":"Workspace","bindings":{"alt-i":"editor::OpenUrl"}}]"#,
    )
    .unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    install_zed_into_dir(&dir, task, keymap, "alt-i", true).unwrap();

    let keymaps = read_json_array(&dir.join("keymap.json")).unwrap();
    assert_eq!(keymaps[0]["context"], "Editor || Workspace");
    assert_eq!(keymaps[0]["use_key_equivalents"], true);
    assert_eq!(keymaps[0]["bindings"]["alt-i"][0], "task::Spawn");
    assert!(dir.join("keymap.json.bak").exists());
}

#[test]
fn rejects_invalid_json_array() {
    let dir = temp_dir("invalid-json");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("tasks.json"), "{").unwrap();
    let task = task_template(Path::new("/usr/local/bin/issue-jumper"));
    let keymap = keymap_template("alt-i");

    let err = install_zed_into_dir(&dir, task, keymap, "alt-i", false).unwrap_err();
    assert!(
        matches!(err, IssueJumperError::ZedConfigInvalidJson(path) if path.ends_with("tasks.json"))
    );
}

#[test]
fn reads_zed_jsonc_arrays() {
    let dir = temp_dir("jsonc-arrays");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("tasks.json"),
        r#"// Static tasks configuration.
[
  {
    "label": "npm test",
    "command": "npm test",
  },
]"#,
    )
    .unwrap();

    let tasks = read_json_array(&dir.join("tasks.json")).unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["label"], "npm test");
}

#[test]
fn write_json_array_reports_backup_copy_error() {
    let dir = temp_dir("backup-copy-error");
    fs::create_dir_all(&dir).unwrap();

    let err = write_json_array(&dir, Vec::new()).unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn write_json_array_reports_write_error() {
    let path = temp_dir("write-error").join("missing").join("tasks.json");

    let err = write_json_array(&path, Vec::new()).unwrap_err();

    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn write_json_array_replaces_existing_file_without_temp_leftover() {
    let dir = temp_dir("atomic-write");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tasks.json");
    fs::write(&path, r#"[{"label":"old"}]"#).unwrap();

    write_json_array(&path, vec![json!({"label": "new"})]).unwrap();

    let tasks = read_json_array(&path).unwrap();
    assert_eq!(tasks[0]["label"], "new");
    assert!(dir.join("tasks.json.bak").exists());
    assert!(!temp_json_path(&path).exists());
}

#[test]
fn task_label_returns_constant() {
    assert_eq!(task_label(), TASK_LABEL);
}

#[test]
fn zed_config_dir_uses_platform_home() {
    let path = zed_config_dir().unwrap();
    #[cfg(target_os = "windows")]
    assert!(path.ends_with("Zed"));
    #[cfg(not(target_os = "windows"))]
    assert!(path.ends_with(".config/zed"));
}

#[test]
fn serde_json_error_maps_to_io() {
    let err: IssueJumperError = serde_json::from_str::<Value>("{").unwrap_err().into();
    assert!(matches!(err, IssueJumperError::Io(_)));
}

#[test]
fn pretty_json_array_outputs_array() {
    let text = pretty_json_array(json!({"label": TASK_LABEL}));
    assert!(text.starts_with('['));
    assert!(text.contains(TASK_LABEL));
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("issue-jumper-{label}-{nonce}"))
}
