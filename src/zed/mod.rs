use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{IssueJumperError, Result};

mod platform;

const TASK_LABEL: &str = "Issue Jumper: Open Current Issue";
const CURRENT_BINARY_NAMES: &[&str] = &["issue-jumper", "issue-jumper.exe"];
const LEGACY_BINARY_NAMES: &[&str] = &["zed-issue-jumper", "zed-issue-jumper.exe"];

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub key: String,
    pub force: bool,
    pub print: bool,
}

pub fn install_zed(options: InstallOptions) -> Result<()> {
    install_zed_from(options, current_executable, zed_config_dir())
}

fn install_zed_from(
    options: InstallOptions,
    current_executable: fn() -> Result<PathBuf>,
    config_dir: Option<PathBuf>,
) -> Result<()> {
    install_zed_with(options, current_executable()?, config_dir)
}

fn install_zed_with(
    options: InstallOptions,
    executable: PathBuf,
    config_dir: Option<PathBuf>,
) -> Result<()> {
    let task = task_template(&executable);
    let keymap = keymap_template(&options.key);

    if options.print {
        println!("tasks.json:");
        println!("{}", pretty_json_array(task));
        println!();
        println!("keymap.json:");
        println!("{}", pretty_json_array(keymap));
        return Ok(());
    }

    let config_dir = config_dir.ok_or(IssueJumperError::ZedConfigPathNotFound)?;
    install_zed_into_dir(&config_dir, task, keymap, &options.key, options.force)?;

    println!("Installed Issue Jumper Zed integration.");
    println!("Task: {TASK_LABEL}");
    println!("Key: {}", options.key);
    Ok(())
}

pub(crate) fn install_zed_into_dir(
    config_dir: &Path,
    task: Value,
    keymap: Value,
    key: &str,
    force: bool,
) -> Result<()> {
    fs::create_dir_all(config_dir)?;

    let tasks_path = config_dir.join("tasks.json");
    let keymap_path = config_dir.join("keymap.json");
    let mut tasks = read_json_array(&tasks_path)?;
    let mut keymaps = read_json_array(&keymap_path)?;

    merge_task(&tasks_path, &mut tasks, task)?;
    merge_keymap(&mut keymaps, keymap, key, force)?;

    write_json_array(&tasks_path, tasks)?;
    write_json_array(&keymap_path, keymaps)?;

    Ok(())
}

pub fn task_label() -> &'static str {
    TASK_LABEL
}

fn current_executable() -> Result<PathBuf> {
    std::env::current_exe().map_err(|err| IssueJumperError::Io(err.to_string()))
}

fn task_template(executable: &Path) -> Value {
    json!({
        "label": TASK_LABEL,
        "command": executable.display().to_string(),
        "args": ["open", "--repo", "$ZED_WORKTREE_ROOT"],
        "cwd": "$ZED_WORKTREE_ROOT",
        "use_new_terminal": false,
        "allow_concurrent_runs": false,
        "reveal": "never",
        "hide": "on_success",
        "show_summary": false,
        "show_command": false,
        "save": "none"
    })
}

fn keymap_template(key: &str) -> Value {
    json!({
        "context": "Workspace",
        "bindings": {
            key: ["task::Spawn", { "task_name": TASK_LABEL }]
        }
    })
}

fn pretty_json_array(value: Value) -> String {
    serde_json::to_string_pretty(&json!([value])).expect("serializing JSON Value should not fail")
}

fn merge_task(path: &Path, tasks: &mut Vec<Value>, task: Value) -> Result<()> {
    let index = tasks
        .iter()
        .position(|value| value.get("label").and_then(Value::as_str) == Some(TASK_LABEL));

    match index {
        Some(index) => {
            if let Some(command) = tasks[index].get("command").and_then(Value::as_str) {
                if !is_issue_jumper_command(command) {
                    return Err(IssueJumperError::ZedConfigInvalidJson(format!(
                        "{} contains task label `{TASK_LABEL}` with a different command",
                        path.display()
                    )));
                }
            }
            tasks[index] = task;
        }
        None => tasks.push(task),
    }
    Ok(())
}

fn is_issue_jumper_command(command: &str) -> bool {
    CURRENT_BINARY_NAMES
        .iter()
        .chain(LEGACY_BINARY_NAMES.iter())
        .any(|name| command.ends_with(name) || command.contains(name))
}

fn merge_keymap(keymaps: &mut Vec<Value>, keymap: Value, key: &str, force: bool) -> Result<()> {
    let mut existing_binding = None;

    for (index, group) in keymaps.iter().enumerate() {
        if let Some(binding) = group
            .get("bindings")
            .and_then(Value::as_object)
            .and_then(|bindings| bindings.get(key))
        {
            existing_binding = Some((index, binding.clone()));
            break;
        }
    }

    if let Some((index, binding)) = existing_binding {
        if !is_issue_jumper_binding(&binding) && !force {
            return Err(IssueJumperError::ZedKeyConflict(key.to_string()));
        }

        if let Some(bindings) = keymaps[index]
            .get_mut("bindings")
            .and_then(Value::as_object_mut)
        {
            bindings.insert(key.to_string(), keymap["bindings"][key].clone());
        }
    } else {
        keymaps.push(keymap);
    }
    Ok(())
}

fn is_issue_jumper_binding(value: &Value) -> bool {
    value
        .as_array()
        .and_then(|items| items.get(1))
        .and_then(|args| args.get("task_name"))
        .and_then(Value::as_str)
        == Some(TASK_LABEL)
}

fn read_json_array(path: &Path) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(path)?;
    json5::from_str(&text)
        .map_err(|_| IssueJumperError::ZedConfigInvalidJson(path.display().to_string()))
}

fn write_json_array(path: &Path, value: Vec<Value>) -> Result<()> {
    if path.exists() {
        let backup = path.with_extension("json.bak");
        fs::copy(path, backup)?;
    }
    let text =
        serde_json::to_string_pretty(&value).expect("serializing JSON Value should not fail");
    fs::write(path, format!("{text}\n"))?;
    Ok(())
}

fn zed_config_dir() -> Option<PathBuf> {
    platform::config_dir()
}

impl From<serde_json::Error> for IssueJumperError {
    fn from(value: serde_json::Error) -> Self {
        IssueJumperError::Io(value.to_string())
    }
}

#[cfg(test)]
mod tests;
