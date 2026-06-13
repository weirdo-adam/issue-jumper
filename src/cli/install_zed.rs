use crate::error::{IssueJumperError, Result};
use crate::zed::{InstallOptions, install_zed};

pub fn run(args: &[String]) -> Result<()> {
    let mut key = "alt-j".to_string();
    let mut force = false;
    let mut print = false;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--key" => {
                key = iter
                    .next()
                    .ok_or_else(|| IssueJumperError::Usage("--key requires a value".to_string()))?
                    .clone();
            }
            "--force" => force = true,
            "--print" => print = true,
            other => {
                return Err(IssueJumperError::Usage(format!(
                    "Unknown install-zed argument `{other}`"
                )));
            }
        }
    }

    install_zed(InstallOptions { key, force, print })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_zed_config_snippets() {
        run(&["--print".to_string()]).unwrap();
    }

    #[test]
    fn accepts_custom_key_when_printing() {
        run(&[
            "--key".to_string(),
            "cmd-i".to_string(),
            "--force".to_string(),
            "--print".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn rejects_missing_key_value() {
        let err = run(&["--key".to_string()]).unwrap_err();
        assert!(matches!(err, IssueJumperError::Usage(_)));
    }

    #[test]
    fn rejects_unknown_argument() {
        let err = run(&["--bad".to_string()]).unwrap_err();
        assert!(matches!(err, IssueJumperError::Usage(_)));
    }
}
