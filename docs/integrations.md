# Integration Examples

Issue Jumper stays editor-neutral. Editor integrations should call the CLI with
the current workspace path and let the CLI resolve the branch, remote, platform,
and target URL.

Print ready-to-copy snippets:

```sh
issue-jumper integration print --target all
issue-jumper integration print --target vscode
issue-jumper integration print --target cursor
issue-jumper integration print --target generic
```

`print` is optional; `issue-jumper integration --target vscode` prints the same VS Code snippets.

Use `--command <path>` when the editor cannot see the same `PATH` as your shell:

```sh
issue-jumper integration print --target vscode --command /opt/homebrew/bin/issue-jumper
```

## VS Code

Create or update `.vscode/tasks.json` in the workspace:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Issue Jumper: Open Current Issue",
      "type": "shell",
      "command": "issue-jumper",
      "args": ["open", "--repo", "${workspaceFolder}"],
      "problemMatcher": [],
      "presentation": {
        "reveal": "never",
        "panel": "dedicated",
        "clear": true
      }
    }
  ]
}
```

Add a user keybinding:

```json
{
  "key": "ctrl+alt+j",
  "command": "workbench.action.tasks.runTask",
  "args": "Issue Jumper: Open Current Issue"
}
```

## Cursor

Cursor can use the same task and keybinding shape as VS Code. If Cursor does not
inherit your shell `PATH`, print the snippet with an absolute CLI path:

```sh
issue-jumper integration print --target cursor --command /opt/homebrew/bin/issue-jumper
```

## Generic Editors

Any editor or launcher that can bind a shortcut to a command can run:

```sh
issue-jumper open --repo /absolute/path/to/repo
```

For tools that expose a workspace variable, replace `/absolute/path/to/repo` with
that variable. For scripts that need the URL instead of opening a browser:

```sh
issue-jumper url --repo /absolute/path/to/repo --print-url
```
