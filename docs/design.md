# Issue Jumper 技术设计文档

## 文档信息

| 项目 | 内容 |
| --- | --- |
| 项目名称 | Issue Jumper |
| 产品版本 | v0.1.0 |
| 文档版本 | 5.0 |
| 最后更新 | 2026-06-13 |
| 状态 | v0.1.0 技术设计 |
| 设计基线 | 通用 CLI / Zed Tasks / Keybindings / Command Palette / Extensions 官方文档 |

## 一、目标与范围

### 1.1 目标体验

v0.1.0 的核心目标是：提供一个通用 CLI，从当前 Git 分支解析并打开对应 Issue 页面；同时提供 Zed 安装适配，在 Zed 中通过一个快捷键直接打开当前 Git 分支对应的 Issue 页面。成功路径不需要打开 Agent、复制 Issue ID 或点击中间链接。

Issue Jumper 遵循“约定大于配置”：GitHub、GitLab、私有 GitLab、常见分支命名默认可用；Redmine、Jira、GitLab work items、企业内部工单系统等非统一 URL 形态通过可选配置覆盖。

### 1.2 运行链路

Command Palette 链路：

```
Zed Command Palette
  -> task: spawn
  -> Issue Jumper: Open Current Issue
  -> Zed task 执行 issue-jumper open --repo $ZED_WORKTREE_ROOT
  -> CLI 读取 Git 分支和 remote
  -> CLI 构建 Issue URL
  -> CLI 调用系统默认浏览器打开 URL
```

快捷键链路：

```
Zed keymap 快捷键
  -> task::Spawn
  -> Zed task 执行 issue-jumper open --repo $ZED_WORKTREE_ROOT
  -> CLI 读取 Git 分支和 remote
  -> CLI 构建 Issue URL
  -> CLI 调用系统默认浏览器打开 URL
```

Zed 集成由 `tasks.json` 和 `keymap.json` 完成。CLI 负责 Git 信息读取、Issue ID 匹配、URL 构建，并调用系统默认浏览器。当前公开 Zed extension API 的 slash command 入口出现在 Agent UI，不是截图中的 Command Palette，因此 v0.1.0 不采用 `/issue` 路径。后续其他编辑器或启动器集成应作为新的 `install-*` 适配层加入，不复制核心解析逻辑。

### 1.3 v0.1.0 功能范围

| 能力 | v0.1.0 设计 |
| --- | --- |
| Command Palette 入口 | 通过 Zed 内置 `task: spawn` 选择 `Issue Jumper: Open Current Issue` |
| 快捷键触发 | 用户 keymap 绑定 `task::Spawn` |
| 一键打开浏览器 | task 调用 `issue-jumper open`，CLI 调用系统 opener |
| 当前项目定位 | task 传入 `$ZED_WORKTREE_ROOT` |
| Git 信息读取 | CLI 在 repo 路径下执行 `git` |
| 平台识别 | 从 remote URL 识别 GitHub、GitLab、Bitbucket、Gitee |
| 配置扩展 | 读取 `.zed/issue-jumper.json` 或 `.issue-jumper.json` |
| 失败反馈 | CLI 输出错误，Zed task terminal 可查看 |

### 1.4 验收标准

| 验收项 | 必须满足 |
| --- | --- |
| 快捷键可用 | 在 Zed Workspace 中按配置快捷键即可触发跳转 |
| 无中间交互 | 成功路径不需要确认、复制、点击或输入命令 |
| 浏览器打开 | 默认浏览器直接打开 Issue URL |
| 当前仓库正确 | 使用当前 Zed worktree 对应的 Git 仓库 |
| 当前分支正确 | 从当前 Git 分支提取 Issue ID |
| 跳转目标正确 | URL 对应当前仓库 remote 和提取出的 Issue ID |
| 失败可诊断 | 失败时能看到明确原因，如无匹配规则、非 Git 仓库、无 remote |

### 1.5 非目标

- v0.1.0 不发布 Zed marketplace extension。
- v0.1.0 不依赖未公开的 Zed Action API，也不宣称能注册独立 `issue: open` Command Palette action。
- 快捷键路径不要求用户打开 Agent、Assistant 或点击 Markdown 链接。
- v0.1.0 不调用 Issue 平台 API，不需要用户 token。
- v0.1.0 不自动创建 Issue，只打开已有 Issue URL。

### 1.6 命名与标识

项目名、用户可见 task 名围绕 `Issue Jumper`；仓库名、命令名和 Cargo package 使用不绑定编辑器的 `issue-jumper`。Zed 相关名称只出现在安装子命令、task label 和配置路径中。

| 对象 | 名称 | 说明 |
| --- | --- | --- |
| 产品展示名 | `Issue Jumper` | 用户看到的主名称，直接表达“跳转到 Issue 页面” |
| 项目/仓库名 | `issue-jumper` | 不绑定单一编辑器，方便后续扩展其他入口 |
| 可执行命令 | `issue-jumper` | 安装器写入 Zed task 的 executable |
| Cargo package | `issue-jumper` | 与命令名保持一致 |
| Rust crate import | `issue_jumper` | Rust 自动将 package hyphen 转为 underscore |
| Zed task label | `Issue Jumper: Open Current Issue` | 清楚描述快捷键触发的动作 |
| 配置文件 | `.issue-jumper.json` / `.zed/issue-jumper.json` | 核心 CLI 配置和 Zed 项目覆盖路径都保持短且可读 |
| README 副标题 | `Jump from the current Git branch to its issue page` | 补充说明核心 CLI 能力 |

## 二、运行环境

### 2.1 Zed 配置

Zed 集成依赖 tasks 和 keybindings：

- task 可以定义在全局 `~/.config/zed/tasks.json` 或项目 `.zed/tasks.json`。
- task 可使用 `ZED_WORKTREE_ROOT` 获取当前 worktree 根目录。
- task 支持 `command`、`args`、`cwd`、`reveal`、`hide`、`show_summary`、`show_command` 等字段。
- keymap 可以通过 `task::Spawn` 直接绑定某个 task。
- Command Palette 中可执行 Zed 内置 `task: spawn`，再选择已安装的 `Issue Jumper: Open Current Issue` task。

当前公开 Zed extension API 没有提供可把自定义 action 直接注册进 Command Palette 的 manifest/API 字段。`slash_commands` 属于 Agent UI 入口，不满足本项目目标。

### 2.2 CLI 执行环境

CLI 可以直接在终端或脚本中运行，也可以在 Zed task 进程中运行。`install-zed` 写入当前 CLI 的绝对路径，避免 Zed task 环境与交互式 shell 的 PATH 不一致。task 通过 `args` 传递 `$ZED_WORKTREE_ROOT`，CLI 使用该路径读取 Git 信息。

## 三、用户流程

### 3.1 安装流程

```
1. 安装 issue-jumper CLI
2. 执行 issue-jumper install-zed
3. 安装器写入或更新 Zed tasks.json
4. 安装器写入或更新 Zed keymap.json
5. 用户在 Zed 中按 alt-j
6. 默认浏览器打开当前分支对应的 Issue
```

### 3.2 使用流程

快捷键使用流程：

```
用户在 Zed 任意项目窗口
  -> 按下配置的快捷键
  -> Zed 执行 "Issue Jumper: Open Current Issue" task
  -> task 将 $ZED_WORKTREE_ROOT 传给 CLI
  -> CLI 读取 Git 分支和 remote
  -> CLI 解析 Issue ID 和平台
  -> CLI 打开系统默认浏览器
```

Command Palette 使用流程：

```
用户在 Zed 任意项目窗口
  -> 打开 Command Palette
  -> 执行 task: spawn
  -> 选择 Issue Jumper: Open Current Issue
  -> Zed 执行同一个 task
  -> CLI 打开系统默认浏览器
```

### 3.3 失败体验

失败时不弹自定义 Zed UI，而是让 task 失败并输出清晰错误：

```text
Issue Jumper failed: no issue ID matched branch "main".
```

task 配置默认 `reveal = "never"`、`hide = "on_success"`，成功时不打断编辑；失败时终端保留输出，便于定位问题。

## 四、Zed 集成设计

### 4.1 Command Palette 边界

截图里的 `Execute a command...` 是 Zed Command Palette。v0.1.0 不能注册独立的 `issue: open` action，因为当前公开 Zed extension API 没有 action/command palette 注册接口。

本项目在 Command Palette 中的可用路径是：

```
task: spawn
  -> Issue Jumper: Open Current Issue
```

这条路径使用 Zed 官方 task 机制，不经过 Agent slash command。

### 4.2 Task 配置

全局 task 路径：

- macOS/Linux：`~/.config/zed/tasks.json`
- Windows：`%APPDATA%\Zed\tasks.json`

Task 配置示例：

```json
[
  {
    "label": "Issue Jumper: Open Current Issue",
    "command": "/absolute/path/to/issue-jumper",
    "args": ["open", "--repo", "$ZED_WORKTREE_ROOT"],
    "cwd": "$ZED_WORKTREE_ROOT",
    "use_new_terminal": false,
    "allow_concurrent_runs": false,
    "reveal": "never",
    "hide": "on_success",
    "show_summary": false,
    "show_command": false,
    "save": "none"
  }
]
```

设计说明：

- `install-zed` 默认写入当前 CLI 的绝对路径；手动配置时，如果确认 PATH 在 Zed task 环境中可用，也可以写 `issue-jumper`。
- 使用 `args` 传递 `$ZED_WORKTREE_ROOT`，避免 shell 字符串拼接和路径空格问题。
- `cwd` 设置为 `$ZED_WORKTREE_ROOT`，CLI 即使没有 `--repo` 也可回退当前目录。
- `reveal = "never"` 保持编辑器焦点。
- `hide = "on_success"` 成功后隐藏任务终端。
- 失败时终端不隐藏，保留错误输出。
- Zed task 在登录 shell 中运行，可能读取用户 shell profile；绝对路径可以降低不同 shell/PATH 配置导致的启动失败。

### 4.3 Keymap 配置

全局 keymap 路径：

- macOS/Linux：`~/.config/zed/keymap.json`
- Windows：`%APPDATA%\Zed\keymap.json`

Keymap 配置示例：

```json
[
  {
    "context": "Workspace",
    "bindings": {
      "alt-j": [
        "task::Spawn",
        { "task_name": "Issue Jumper: Open Current Issue" }
      ]
    }
  }
]
```

### 4.4 安装器

CLI 提供安装辅助：

```bash
issue-jumper install-zed
```

安装器职责：

1. 定位 Zed 配置目录。
2. 创建缺失的 `tasks.json`、`keymap.json`。
3. 解析 JSON 数组，保留用户已有配置。
4. 按 `label` 更新或追加 Issue Jumper task。
5. 按 task name 更新或追加 keymap binding。
6. 修改前生成 `.bak` 备份。
7. 输出安装结果和冲突提示。

冲突处理：

| 场景 | 行为 |
| --- | --- |
| 快捷键未占用 | 直接写入 |
| 快捷键已绑定其他 action | 不覆盖，提示用户使用 `--force` 或指定新 key |
| task label 已存在且 command 相同 | 更新参数 |
| task label 已存在但 command 不同 | 不覆盖，提示手动处理 |
| JSON 格式非法 | 不写入，报告文件路径和解析错误 |

公开一键安装脚本和本地源码安装脚本默认调用 `install-zed --force`，用于支持重复安装时覆盖并刷新所选快捷键；直接运行 `issue-jumper install-zed` 仍保留不覆盖外部快捷键的安全默认值。

### 4.5 手动配置兜底

如果安装器无法安全合并配置，`install-zed --print` 输出 task/keymap 片段，由用户手动复制到 Zed 配置。

## 五、CLI 设计

### 5.1 命令结构

```text
issue-jumper open [--repo <path>] [--platform <name>] [--rule <name>]
issue-jumper url  [--repo <path>] [--platform <name>] [--rule <name>] [--print-url]
issue-jumper install-zed [--key <key>] [--force] [--print]
issue-jumper doctor [--repo <path>]
```

| 命令 | 用途 |
| --- | --- |
| `open` | 构建 URL 并打开默认浏览器，Zed 快捷键调用此命令 |
| `url` | 只输出 URL，便于测试和脚本使用 |
| `install-zed` | 写入 Zed task/keymap 集成 |
| `doctor` | 检查 git、repo、branch、remote、配置和生成 URL |

### 5.2 `open` 行为

```
1. 解析 repo 路径；优先 --repo，否则使用当前工作目录
2. 读取项目配置
3. 执行 git branch --show-current
4. 必要时降级 git rev-parse --abbrev-ref HEAD
5. 从分支名提取 Issue ID
6. 读取 git remote get-url origin；失败后尝试 upstream
7. 解析 remote host/path
8. 解析目标平台
9. 构建 Issue URL
10. 调用系统 opener 打开 URL
11. stdout 输出 URL，exit code = 0
```

### 5.3 系统浏览器打开

按平台调用系统命令：

| 平台 | 命令 |
| --- | --- |
| macOS | `open <url>` |
| Linux | `xdg-open <url>` |
| Windows | `cmd /C start "" <url>` |

实现要求：

- 不通过 shell 拼接 URL。
- URL 必须先校验 scheme 为 `http` 或 `https`。
- opener 失败时返回非 0 exit code，并输出 URL 供用户手动打开。

## 六、系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                            Zed                              │
│                                                             │
│  keymap.json                                                │
│    alt-j -> task::Spawn("Issue Jumper: Open Current Issue") │
│                                                             │
│  tasks.json                                                 │
│    command: issue-jumper                                    │
│    args: open --repo $ZED_WORKTREE_ROOT                     │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    issue-jumper CLI                         │
│                                                             │
│  CommandParser -> JumpService -> BrowserOpener              │
│                       │                                     │
│       ┌───────────────┼───────────────┐                     │
│       ▼               ▼               ▼                     │
│   GitReader      IdExtractor     UrlBuilder                 │
│       │               │               │                     │
│       ▼               ▼               ▼                     │
│  git command     branch rules     issue URL                 │
└─────────────────────────────────────────────────────────────┘
```

### 6.1 模块职责

| 模块 | 文件位置 | 职责 |
| --- | --- | --- |
| `main.rs` | `src/main.rs` | 二进制入口，只调用库 crate 的 CLI |
| `lib.rs` | `src/lib.rs` | crate 模块出口 |
| `cli/open.rs` | `src/cli/open.rs` | `open` 命令 |
| `cli/url.rs` | `src/cli/url.rs` | `url` 命令 |
| `cli/install_zed.rs` | `src/cli/install_zed.rs` | Zed task/keymap 安装命令 |
| `cli/doctor.rs` | `src/cli/doctor.rs` | 环境诊断命令 |
| `jump.rs` | `src/jump.rs` | 主流程编排 |
| `git` | `src/git/mod.rs` | Git 命令封装 |
| `git::remote` | `src/git/remote.rs` | remote URL 解析 |
| `issue` | `src/issue/mod.rs` | Issue ID 提取 |
| `url` | `src/url/mod.rs` | URL 构建 |
| `config.rs` | `src/config.rs` | 项目配置读取 |
| `browser` | `src/browser/mod.rs` | 浏览器打开 |
| `zed` | `src/zed/mod.rs` | Zed 配置目录和 JSON 合并适配层 |
| `error.rs` | `src/error.rs` | 错误类型 |

## 七、数据模型

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    GitHub,
    GitLab,
    Bitbucket,
    Gitee,
    Redmine,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct IssueRule {
    pub name: String,
    pub pattern: regex::Regex,
    pub platform_hint: Option<Platform>,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub remote_name: String,
    pub original_url: String,
    pub host: String,
    pub path: String,
    pub platform: Platform,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JumpRequest {
    pub repo: std::path::PathBuf,
    pub rule_name: Option<String>,
    pub platform_override: Option<Platform>,
}

#[derive(Debug, Clone)]
pub struct JumpResult {
    pub repo: std::path::PathBuf,
    pub branch: String,
    pub issue_id: String,
    pub platform: Platform,
    pub matched_rule: String,
    pub url: String,
    pub remote: Option<RemoteInfo>,
}
```

## 八、配置设计

### 8.1 项目配置路径

按优先级读取：

1. `<repo>/.zed/issue-jumper.json`
2. `<repo>/.issue-jumper.json`

### 8.2 配置示例

```json
{
  "fallback_platform": "redmine",
  "redmine_base_url": "https://redmine.company.com",
  "disabled_default_rules": ["trailing-number"],
  "issue_rules": [
    {
      "name": "redmine-prefix",
      "pattern": "(?i)redmine[-_](?P<id>\\d+)",
      "platform": "redmine"
    }
  ],
  "custom_platforms": [
    {
      "name": "jira",
      "host_patterns": ["jira.company.com"],
      "url_template": "https://jira.company.com/browse/{id}"
    }
  ]
}
```

### 8.3 Schema 要点

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `fallback_platform` | string | 否 | 规则和 remote 都无法确定平台时使用的平台 |
| `redmine_base_url` | string | Redmine 场景必填 | Redmine 不从 Git remote 自动推导 |
| `disabled_default_rules` | string[] | 否 | 禁用内置分支匹配规则 |
| `issue_rules[].name` | string | 是 | 规则唯一名 |
| `issue_rules[].pattern` | string | 是 | 必须包含 `(?P<id>...)` |
| `issue_rules[].platform` | string | 否 | 匹配后强制指定平台 |
| `custom_platforms[].name` | string | 是 | 自定义平台名 |
| `custom_platforms[].host_patterns` | string[] | 否 | remote host 匹配 |
| `custom_platforms[].url_template` | string | 是 | 支持 `{id}`、`{host}`、`{owner}`、`{repo}`、`{project}` |

### 8.4 高级定制跳转

v0.1.0 支持高级定制 Issue 跳转，不支持泛化的任意目标跳转。

v0.1.0 可配置能力：

| 能力 | 示例 |
| --- | --- |
| 自定义分支规则 | `feature/REDMINE-123`、`bugfix/JIRA-ABC-456` |
| 自定义平台 URL | Redmine、Jira、私有 GitLab、企业工单系统 |
| 指定默认平台 | remote 缺失或不可识别时使用 Redmine/Jira |
| 禁用默认规则 | 避免尾部数字误匹配 |
| 指定规则优先级 | 企业规则优先于内置规则 |

v0.1.0 不包含以下通用跳转能力：

| 能力 | 说明 |
| --- | --- |
| 根据分支打开 PR 页面 | 不包含在 v0.1.0 |
| 根据分支打开 CI pipeline 页面 | 不包含在 v0.1.0 |
| 根据当前文件或选中文本打开文档、监控面板 | 不包含在 v0.1.0 |
| 多目标选择菜单 | 需要额外交互，不包含在 v0.1.0 |

## 九、Git 读取设计

### 9.1 命令列表

| 目的 | 命令 |
| --- | --- |
| 当前分支 | `git -C <repo> branch --show-current` |
| 当前分支降级 | `git -C <repo> rev-parse --abbrev-ref HEAD` |
| remote origin | `git -C <repo> remote get-url origin` |
| remote upstream | `git -C <repo> remote get-url upstream` |

### 9.2 分支读取

```
1. 执行 branch --show-current
2. stdout 非空则返回分支名
3. 执行 rev-parse --abbrev-ref HEAD
4. 输出为空或 HEAD 则 DetachedHead
5. exit code 非 0 且 stderr 指向非 Git 仓库则 NotGitRepo
```

### 9.3 Remote 读取

```
1. 优先读取 origin
2. 失败后读取 upstream
3. 全部失败返回 None
```

remote 缺失不是 GitReader 致命错误；若配置提供了 `fallback_platform` 和足够的 URL 模板字段，仍可继续。

## 十、RemoteParser 设计

### 10.1 支持格式

| 格式 | 示例 | host | path |
| --- | --- | --- | --- |
| SCP-like SSH | `git@github.com:owner/repo.git` | `github.com` | `owner/repo` |
| HTTPS | `https://github.com/owner/repo.git` | `github.com` | `owner/repo` |
| SSH URL | `ssh://git@gitlab.com:2222/team/app.git` | `gitlab.com` | `team/app` |
| Git protocol | `git://github.com/owner/repo.git` | `github.com` | `owner/repo` |

处理规则：

- 去除路径开头 `/`。
- 去除末尾 `.git`。
- 保留 GitLab 多级项目路径，如 `group/subgroup/app`。
- `owner` 为 path 第一段，`repo` 为 path 最后一段。
- `project` 为完整 path。

### 10.2 平台识别

| 条件 | 平台 |
| --- | --- |
| host 等于 `github.com` | GitHub |
| host 等于 `gitlab.com` | GitLab |
| host 形如 `gitlab.*` 或 `*.gitlab.*` | GitLab |
| host 等于 `bitbucket.org` | Bitbucket |
| host 等于 `gitee.com` | Gitee |
| 匹配 `custom_platforms[].host_patterns` | Custom |
| 其他 | Custom(host) |

`custom_platforms[].host_patterns` 优先于内置平台识别，便于把私有 GitLab 覆盖为 GitLab work item、Jira 或企业内部工单 URL。

Redmine 不通过 Git remote 自动识别。Redmine 必须来自匹配规则、`--platform redmine` 或 `fallback_platform = redmine`。

## 十一、Issue ID 提取

默认规则按优先级执行：

| 名称 | 正则 | 示例 | ID |
| --- | --- | --- | --- |
| `github-gh-prefix` | `(?i)\\bGH[-_]?(?P<id>\\d+)\\b` | `feature/GH-123` | `123` |
| `issue-prefix` | `(?i)\\bissue[-_]?(?P<id>\\d+)\\b` | `fix/issue-456` | `456` |
| `hash-number` | `#(?P<id>\\d+)\\b` | `bug/#789` | `789` |
| `leading-number` | `^(?P<id>\\d+)[-_]` | `101-add-login` | `101` |
| `jira-like-key` | `\\b(?P<id>[A-Z][A-Z0-9]+-\\d+)\\b` | `feature/ABC-456-login` | `ABC-456` |
| `trailing-number` | `[-_/](?P<id>\\d+)$` | `feature/login-789` | `789` |

规则要求：

- 必须包含命名捕获组 `id`。
- 数字 ID 和字符串 ID 都允许。
- 用户自定义规则优先于内置规则。
- `--rule` 指定后只执行该规则。

## 十二、URL 构建

### 12.1 默认模板

| 平台 | 模板 |
| --- | --- |
| GitHub | `https://github.com/{owner}/{repo}/issues/{id}` |
| GitLab | `https://{host}/{project}/-/issues/{id}` |
| Bitbucket | `https://bitbucket.org/{owner}/{repo}/issues/{id}` |
| Gitee | `https://gitee.com/{owner}/{repo}/issues/{id}` |
| Redmine | `{redmine_base_url}/issues/{id}` |
| Custom | 使用 `custom_platforms[].url_template` |

### 12.2 构建规则

```
1. 选择平台模板
2. 检查模板必需字段是否存在
3. 对占位符值进行 URL path segment 编码
4. 对 GitLab project 保留 /，逐段编码
5. 拼接 URL
6. 校验 scheme 为 http 或 https
```

字段缺失时直接失败，不能生成错误 URL。

## 十三、错误处理

```rust
pub enum IssueJumperError {
    GitNotFound,
    RepoPathInvalid(String),
    NotGitRepo(String),
    DetachedHead,
    RemoteParseFailed(String),
    NoMatchingRule(String),
    InvalidConfig(String),
    PlatformUnresolved,
    UrlBuildFailed(String),
    BrowserOpenFailed(String),
    ZedConfigPathNotFound,
    ZedConfigInvalidJson(String),
    ZedKeyConflict(String),
    Io(String),
    Usage(String),
}
```

### 13.1 Exit Code

| Code | 含义 |
| --- | --- |
| 0 | 成功打开或成功输出 URL |
| 1 | 业务错误，如无匹配规则、无 remote |
| 2 | 配置错误 |
| 3 | Git 环境错误 |
| 4 | 浏览器打开失败 |
| 5 | Zed 配置安装失败 |

### 13.2 错误文案

| 错误 | 文案 |
| --- | --- |
| 非 Git 仓库 | `Current Zed worktree is not a Git repository: <path>` |
| detached HEAD | `Repository is in detached HEAD state; no branch name is available.` |
| 无匹配规则 | `No issue ID matched branch "<branch>".` |
| 平台无法决定 | `Cannot determine issue platform. Configure fallback_platform or pass --platform.` |
| 浏览器打开失败 | `Failed to open browser. URL: <url>` |
| 快捷键冲突 | `Key "<key>" is already bound in Zed keymap.json.` |

## 十四、测试策略

### 14.1 单元测试

| 模块 | 测试重点 |
| --- | --- |
| `issue` | 默认规则、自定义规则、优先级、命名捕获组校验 |
| `git::remote` | SSH、HTTPS、SSH URL、Git protocol、多级 GitLab project |
| `url` | 各平台模板、字段缺失、URL 编码、GitLab project 分段编码 |
| `config` | 配置读取优先级、非法 JSON、非法正则、禁用默认规则 |
| `zed` | tasks/keymap JSON 合并、冲突检测、备份逻辑 |
| `browser` | 平台命令选择、URL scheme 校验、失败回退 |

### 14.2 集成测试

使用临时 Git 仓库测试：

| 场景 | 预期 |
| --- | --- |
| GitHub remote + `feature/GH-123` | `open` 构建 GitHub issue URL |
| GitLab remote + `456-add-login` | 构建 GitLab issue URL |
| GitLab 多级 project | project 路径保留 `/` 并逐段编码 |
| detached HEAD | 返回 exit code 3 |
| 非 Git 目录 | 返回 exit code 3 |
| 无 remote + Redmine 配置 | 构建 Redmine URL |
| 无匹配分支 | 返回 exit code 1 |

浏览器打开测试默认使用 mock opener；真实 opener 作为手动验收。

### 14.4 本地 mock 数据

本地 mock 数据放在 `local/mock-data/`，该目录通过 `.gitignore` 排除，不提交到 git。当前样例覆盖：

| 平台 | 样例 URL | 用途 |
| --- | --- | --- |
| GitLab | `https://gitlab.example.com/devops/app/-/work_items/30` | 验证私有 GitLab 与 work item 自定义模板 |
| Redmine | `https://redmine.example.com/issues/149228` | 验证 Redmine base URL 与数字 Issue ID |
| GitHub | `https://github.com/weirdo-adam/issue-jumper/issues/1` | 验证公开 GitHub issue URL |

自动化测试不能依赖 `local/` 下的 ignored 文件；它只用于本地手动验证和调试。

### 14.3 Zed 验收测试

| 验收项 | 通过标准 |
| --- | --- |
| `install-zed` | 正确写入或更新 tasks/keymap，并生成备份 |
| 快捷键触发 | 在 Zed Workspace 中触发 `alt-j` keymap 绑定会执行 task |
| worktree 传参 | CLI 收到 `$ZED_WORKTREE_ROOT` 对应的项目路径 |
| 成功体验 | 浏览器打开 Issue URL，Zed 焦点不被明显打断 |
| 失败体验 | task 失败并保留可读错误输出 |

## 十五、文件结构

```
issue-jumper/
├── Cargo.toml
├── rust-toolchain.toml
├── Makefile
├── .editorconfig
├── .github/
│   └── workflows/
│       └── ci.yml
├── README.md
├── README.zh-CN.md
├── AGENTS.md
├── LICENSE
├── docs/
│   ├── design.md
│   └── development.md
├── scripts/
│   ├── coverage.sh
│   ├── install-zed.sh
│   ├── package-release.sh
│   └── publish-release.sh
└── src/
    ├── lib.rs
    ├── main.rs
    ├── cli/
    │   ├── mod.rs
    │   ├── open.rs
    │   ├── url.rs
    │   ├── install_zed.rs
    │   └── doctor.rs
    ├── browser/
    │   └── mod.rs
    ├── git/
    │   ├── mod.rs
    │   └── remote.rs
    ├── issue/
    │   └── mod.rs
    ├── url/
    │   └── mod.rs
    ├── zed/
    │   └── mod.rs
    ├── jump.rs
    ├── config.rs
    ├── platform.rs
    └── error.rs
```

## 十六、运行约束与处理

| 场景 | 影响 | 处理 |
| --- | --- | --- |
| 快捷键冲突 | 用户按键触发其他动作 | `install-zed` 默认不覆盖，提示换 key 或 `--force` |
| CLI 不在 PATH | task 执行失败 | 安装器写入绝对路径，或 doctor 明确提示 |
| Zed task 终端短暂出现 | 打断编辑流程 | `reveal = "never"`、`hide = "on_success"` |
| 浏览器 opener 失败 | 无法打开页面 | 输出 URL 并返回 exit code 4 |
| remote 缺失 | 无法推导平台 | 支持 Redmine/fallback_platform/custom template |
| 分支规则误匹配 | 打开错误 Issue | 输出 URL 到 stdout；支持禁用规则和指定规则 |
| 路径含空格 | task 参数解析错误 | 使用 `args` 数组传递 `$ZED_WORKTREE_ROOT` |

## 十七、参考资料

- [Zed Tasks](https://zed.dev/docs/tasks)
- [Zed Keybindings](https://zed.dev/docs/key-bindings)
