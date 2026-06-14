use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::error::{IssueJumperError, Result};

pub mod remote;

#[derive(Debug, Clone)]
pub struct GitReader {
    repo: PathBuf,
    git_binary: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RemoteUrl {
    pub name: String,
    pub url: String,
}

impl GitReader {
    pub fn new(repo: PathBuf) -> Result<Self> {
        Self::with_git_binary(repo, PathBuf::from("git"))
    }

    fn with_git_binary(repo: PathBuf, git_binary: PathBuf) -> Result<Self> {
        if !repo.exists() || !repo.is_dir() {
            return Err(IssueJumperError::RepoPathInvalid(
                repo.display().to_string(),
            ));
        }
        Ok(Self { repo, git_binary })
    }

    pub fn current_branch(&self) -> Result<String> {
        let primary = self.run_git(["branch", "--show-current"])?;
        if primary.status.success() {
            let branch = output_stdout(&primary);
            if !branch.is_empty() {
                return Ok(branch);
            }
        } else if looks_like_not_git_repo(&primary) {
            return Err(IssueJumperError::NotGitRepo(
                self.repo.display().to_string(),
            ));
        }

        let fallback = self.run_git(["rev-parse", "--abbrev-ref", "HEAD"])?;
        if !fallback.status.success() {
            if looks_like_not_git_repo(&fallback) {
                return Err(IssueJumperError::NotGitRepo(
                    self.repo.display().to_string(),
                ));
            }
            return Err(IssueJumperError::DetachedHead);
        }

        let branch = output_stdout(&fallback);
        if branch.is_empty() || branch == "HEAD" {
            return Err(IssueJumperError::DetachedHead);
        }

        Ok(branch)
    }

    pub fn remote_url(&self) -> Result<Option<RemoteUrl>> {
        for remote_name in ["origin", "upstream"] {
            let output = self.run_git(["remote", "get-url", remote_name])?;
            if output.status.success() {
                let url = output_stdout(&output);
                if !url.is_empty() {
                    return Ok(Some(RemoteUrl {
                        name: remote_name.to_string(),
                        url,
                    }));
                }
            }
        }

        Ok(None)
    }

    fn run_git<const N: usize>(&self, args: [&str; N]) -> Result<Output> {
        let mut command = Command::new(&self.git_binary);
        command.arg("-C").arg(&self.repo);
        for arg in args {
            command.arg(arg);
        }
        command.output().map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                IssueJumperError::GitNotFound
            } else {
                IssueJumperError::Io(err.to_string())
            }
        })
    }
}

pub fn resolve_repo(repo: Option<PathBuf>) -> Result<PathBuf> {
    let repo = match repo {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    normalize_path(&repo)
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|_| IssueJumperError::RepoPathInvalid(path.display().to_string()))
}

fn output_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn output_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_ascii_lowercase()
}

fn looks_like_not_git_repo(output: &Output) -> bool {
    let stderr = output_stderr(output);
    stderr_looks_like_not_git_repo(&stderr)
}

fn stderr_looks_like_not_git_repo(stderr: &str) -> bool {
    stderr.contains("not a git repository") || stderr.contains("not in a git directory")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn rejects_invalid_repo_path() {
        let err = GitReader::new(PathBuf::from("/definitely/missing/repo")).unwrap_err();
        assert!(matches!(err, IssueJumperError::RepoPathInvalid(_)));

        let err = resolve_repo(Some(PathBuf::from("/definitely/missing/repo"))).unwrap_err();
        assert!(matches!(err, IssueJumperError::RepoPathInvalid(_)));
    }

    #[test]
    fn resolves_current_dir_when_repo_is_none() {
        let resolved = resolve_repo(None).unwrap();
        assert!(resolved.is_absolute());
    }

    #[test]
    fn reports_not_git_repo() {
        let repo = temp_dir("not-git");
        fs::create_dir_all(&repo).unwrap();

        let err = GitReader::new(repo).unwrap().current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::NotGitRepo(_)));
    }

    #[test]
    fn reports_missing_git_binary() {
        let repo = temp_dir("missing-git");
        fs::create_dir_all(&repo).unwrap();
        let reader =
            GitReader::with_git_binary(repo, PathBuf::from("/definitely/missing/issue-jumper-git"))
                .unwrap();

        let err = reader.current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::GitNotFound));
    }

    #[test]
    fn reports_non_not_found_git_io_error() {
        let repo = temp_dir("git-io");
        fs::create_dir_all(&repo).unwrap();
        let reader = GitReader::with_git_binary(repo.clone(), repo).unwrap();

        let err = reader.current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::Io(_)));
    }

    #[test]
    fn reads_origin_upstream_and_missing_remote() {
        let origin_repo = temp_dir("origin");
        git(&origin_repo, ["init"]);
        git(&origin_repo, ["checkout", "-b", "feature/GH-1"]);
        git(
            &origin_repo,
            ["remote", "add", "origin", "git@github.com:owner/repo.git"],
        );
        let reader = GitReader::new(origin_repo).unwrap();
        let remote = reader.remote_url().unwrap().unwrap();
        assert_eq!(remote.name, "origin");
        assert_eq!(remote.url, "git@github.com:owner/repo.git");

        let upstream_repo = temp_dir("upstream");
        git(&upstream_repo, ["init"]);
        git(&upstream_repo, ["checkout", "-b", "feature/GH-1"]);
        git(
            &upstream_repo,
            ["remote", "add", "upstream", "git@gitlab.com:owner/repo.git"],
        );
        let reader = GitReader::new(upstream_repo).unwrap();
        let remote = reader.remote_url().unwrap().unwrap();
        assert_eq!(remote.name, "upstream");
        assert_eq!(remote.url, "git@gitlab.com:owner/repo.git");

        let no_remote_repo = temp_dir("no-remote");
        git(&no_remote_repo, ["init"]);
        git(&no_remote_repo, ["checkout", "-b", "feature/GH-1"]);
        assert!(
            GitReader::new(no_remote_repo)
                .unwrap()
                .remote_url()
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn uses_rev_parse_fallback_when_primary_branch_is_empty() {
        let repo = temp_dir("fallback");
        fs::create_dir_all(&repo).unwrap();
        let git = fake_git(
            "fallback",
            r#"#!/bin/sh
if [ "$3" = "branch" ]; then
  exit 0
fi
if [ "$3" = "rev-parse" ]; then
  echo feature/fallback
  exit 0
fi
exit 1
"#,
            r#"@echo off
if "%3"=="branch" exit /B 0
if "%3"=="rev-parse" (
  echo feature/fallback
  exit /B 0
)
exit /B 1
"#,
        );
        let reader = GitReader::with_git_binary(repo, git).unwrap();

        assert_eq!(reader.current_branch().unwrap(), "feature/fallback");
    }

    #[test]
    fn reports_detached_head_when_fallback_command_fails() {
        let repo = temp_dir("fallback-fails");
        fs::create_dir_all(&repo).unwrap();
        let git = fake_git(
            "fallback-fails",
            r#"#!/bin/sh
if [ "$3" = "branch" ]; then
  exit 0
fi
exit 1
"#,
            r#"@echo off
if "%3"=="branch" exit /B 0
exit /B 1
"#,
        );
        let reader = GitReader::with_git_binary(repo, git).unwrap();

        let err = reader.current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::DetachedHead));
    }

    #[test]
    fn uses_fallback_after_non_not_git_primary_failure() {
        let repo = temp_dir("primary-fails");
        fs::create_dir_all(&repo).unwrap();
        let git = fake_git(
            "primary-fails",
            r#"#!/bin/sh
if [ "$3" = "branch" ]; then
  echo "temporary branch failure" >&2
  exit 1
fi
if [ "$3" = "rev-parse" ]; then
  echo feature/recovered
  exit 0
fi
exit 1
"#,
            r#"@echo off
if "%3"=="branch" (
  echo temporary branch failure 1>&2
  exit /B 1
)
if "%3"=="rev-parse" (
  echo feature/recovered
  exit /B 0
)
exit /B 1
"#,
        );
        let reader = GitReader::with_git_binary(repo, git).unwrap();

        assert_eq!(reader.current_branch().unwrap(), "feature/recovered");
    }

    #[test]
    fn treats_fallback_not_git_error_as_not_git_repo() {
        let repo = temp_dir("fallback-not-git");
        fs::create_dir_all(&repo).unwrap();
        let git = fake_git(
            "fallback-not-git",
            r#"#!/bin/sh
if [ "$3" = "branch" ]; then
  exit 0
fi
echo "fatal: not in a git directory" >&2
exit 1
"#,
            r#"@echo off
if "%3"=="branch" exit /B 0
echo fatal: not in a git directory 1>&2
exit /B 1
"#,
        );
        let reader = GitReader::with_git_binary(repo, git).unwrap();

        let err = reader.current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::NotGitRepo(_)));
    }

    #[test]
    fn ignores_empty_remote_url() {
        let repo = temp_dir("empty-remote");
        fs::create_dir_all(&repo).unwrap();
        let git = fake_git(
            "empty-remote",
            r#"#!/bin/sh
if [ "$3" = "remote" ]; then
  exit 0
fi
exit 1
"#,
            r#"@echo off
if "%3"=="remote" exit /B 0
exit /B 1
"#,
        );
        let reader = GitReader::with_git_binary(repo, git).unwrap();

        assert!(reader.remote_url().unwrap().is_none());
    }

    #[test]
    fn detects_not_git_error_variants() {
        assert!(stderr_looks_like_not_git_repo(
            "fatal: not a git repository"
        ));
        assert!(stderr_looks_like_not_git_repo(
            "fatal: not in a git directory"
        ));
        assert!(!stderr_looks_like_not_git_repo("other error"));
    }

    #[test]
    fn reports_detached_head() {
        let repo = temp_dir("detached");
        git(&repo, ["init"]);
        git(&repo, ["config", "user.email", "test@example.com"]);
        git(&repo, ["config", "user.name", "Test User"]);
        fs::write(repo.join("README.md"), "test").unwrap();
        git(&repo, ["add", "README.md"]);
        git(&repo, ["commit", "-m", "initial"]);
        let commit = git_stdout(&repo, ["rev-parse", "HEAD"]);
        git(&repo, ["checkout", commit.trim()]);

        let err = GitReader::new(repo).unwrap().current_branch().unwrap_err();

        assert!(matches!(err, IssueJumperError::DetachedHead));
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("issue-jumper-git-{label}-{nonce}-{counter}"))
    }

    fn git<const N: usize>(repo: &Path, args: [&str; N]) {
        fs::create_dir_all(repo).unwrap();
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn git_stdout<const N: usize>(repo: &Path, args: [&str; N]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    }

    fn fake_git(label: &str, _unix_script: &str, _windows_script: &str) -> PathBuf {
        #[cfg(windows)]
        let path = temp_dir(label).join("git.bat");
        #[cfg(not(windows))]
        let path = temp_dir(label).join("git");
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let temp_path = path.with_file_name(format!(
            "{}.tmp",
            path.file_name().unwrap().to_string_lossy()
        ));
        #[cfg(windows)]
        fs::write(&temp_path, _windows_script).unwrap();
        #[cfg(not(windows))]
        fs::write(&temp_path, _unix_script).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&temp_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&temp_path, permissions).unwrap();
        }

        fs::rename(&temp_path, &path).unwrap();
        path
    }
}
