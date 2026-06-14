use std::fmt;

#[derive(Debug)]
pub enum IssueJumperError {
    GitNotFound,
    RepoPathInvalid(String),
    NotGitRepo(String),
    DetachedHead,
    RemoteParseFailed(String),
    NoMatchingRule(String),
    UnknownRule(String),
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

impl IssueJumperError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NoMatchingRule(_)
            | Self::UnknownRule(_)
            | Self::PlatformUnresolved
            | Self::UrlBuildFailed(_) => 1,
            Self::InvalidConfig(_) => 2,
            Self::GitNotFound
            | Self::RepoPathInvalid(_)
            | Self::NotGitRepo(_)
            | Self::DetachedHead => 3,
            Self::BrowserOpenFailed(_) => 4,
            Self::ZedConfigPathNotFound
            | Self::ZedConfigInvalidJson(_)
            | Self::ZedKeyConflict(_) => 5,
            Self::RemoteParseFailed(_) | Self::Io(_) | Self::Usage(_) => 1,
        }
    }
}

impl fmt::Display for IssueJumperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitNotFound => write!(f, "Git was not found in PATH."),
            Self::RepoPathInvalid(path) => write!(f, "Repository path is invalid: {path}"),
            Self::NotGitRepo(path) => {
                write!(f, "Current Zed worktree is not a Git repository: {path}")
            }
            Self::DetachedHead => write!(
                f,
                "Repository is in detached HEAD state; no branch name is available."
            ),
            Self::RemoteParseFailed(remote) => {
                write!(f, "Failed to parse Git remote URL: {remote}")
            }
            Self::NoMatchingRule(branch) => write!(f, "No issue ID matched branch \"{branch}\"."),
            Self::UnknownRule(rule) => write!(f, "Unknown issue rule \"{rule}\"."),
            Self::InvalidConfig(message) => write!(f, "Invalid Issue Jumper config: {message}"),
            Self::PlatformUnresolved => {
                write!(
                    f,
                    "Cannot determine issue platform. Configure fallback_platform or pass --platform."
                )
            }
            Self::UrlBuildFailed(message) => write!(f, "Failed to build issue URL: {message}"),
            Self::BrowserOpenFailed(message) => write!(f, "Failed to open browser. {message}"),
            Self::ZedConfigPathNotFound => {
                write!(f, "Could not locate the Zed configuration directory.")
            }
            Self::ZedConfigInvalidJson(path) => write!(f, "Zed config JSON is invalid: {path}"),
            Self::ZedKeyConflict(key) => {
                write!(f, "Key \"{key}\" is already bound in Zed keymap.json.")
            }
            Self::Io(message) => write!(f, "{message}"),
            Self::Usage(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for IssueJumperError {}

impl From<std::io::Error> for IssueJumperError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, IssueJumperError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_exit_codes() {
        let cases = [
            (IssueJumperError::NoMatchingRule("main".to_string()), 1),
            (IssueJumperError::PlatformUnresolved, 1),
            (IssueJumperError::UrlBuildFailed("bad".to_string()), 1),
            (IssueJumperError::RemoteParseFailed("bad".to_string()), 1),
            (IssueJumperError::Io("io".to_string()), 1),
            (IssueJumperError::Usage("usage".to_string()), 1),
            (IssueJumperError::UnknownRule("missing".to_string()), 1),
            (IssueJumperError::InvalidConfig("bad".to_string()), 2),
            (IssueJumperError::GitNotFound, 3),
            (IssueJumperError::RepoPathInvalid("/x".to_string()), 3),
            (IssueJumperError::NotGitRepo("/x".to_string()), 3),
            (IssueJumperError::DetachedHead, 3),
            (IssueJumperError::BrowserOpenFailed("bad".to_string()), 4),
            (IssueJumperError::ZedConfigPathNotFound, 5),
            (IssueJumperError::ZedConfigInvalidJson("bad".to_string()), 5),
            (IssueJumperError::ZedKeyConflict("alt-i".to_string()), 5),
        ];

        for (err, code) in cases {
            assert_eq!(err.exit_code(), code);
        }
    }

    #[test]
    fn formats_all_error_messages() {
        let errors = [
            IssueJumperError::GitNotFound,
            IssueJumperError::RepoPathInvalid("/x".to_string()),
            IssueJumperError::NotGitRepo("/x".to_string()),
            IssueJumperError::DetachedHead,
            IssueJumperError::RemoteParseFailed("remote".to_string()),
            IssueJumperError::NoMatchingRule("main".to_string()),
            IssueJumperError::UnknownRule("missing".to_string()),
            IssueJumperError::InvalidConfig("bad".to_string()),
            IssueJumperError::PlatformUnresolved,
            IssueJumperError::UrlBuildFailed("bad".to_string()),
            IssueJumperError::BrowserOpenFailed("bad".to_string()),
            IssueJumperError::ZedConfigPathNotFound,
            IssueJumperError::ZedConfigInvalidJson("keymap.json".to_string()),
            IssueJumperError::ZedKeyConflict("alt-i".to_string()),
            IssueJumperError::Io("io".to_string()),
            IssueJumperError::Usage("usage".to_string()),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn converts_io_error() {
        let err: IssueJumperError = std::io::Error::other("boom").into();
        assert!(matches!(err, IssueJumperError::Io(message) if message == "boom"));
    }
}
