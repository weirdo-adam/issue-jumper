#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    GitHub,
    GitLab,
    Bitbucket,
    Gitee,
    Redmine,
    Custom(String),
}

impl Platform {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "github" => Self::GitHub,
            "gitlab" => Self::GitLab,
            "bitbucket" => Self::Bitbucket,
            "gitee" => Self::Gitee,
            "redmine" => Self::Redmine,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::Bitbucket => "bitbucket",
            Self::Gitee => "gitee",
            Self::Redmine => "redmine",
            Self::Custom(name) => name.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_and_custom_platforms() {
        assert_eq!(Platform::parse("github"), Platform::GitHub);
        assert_eq!(Platform::parse("GitLab"), Platform::GitLab);
        assert_eq!(Platform::parse("bitbucket"), Platform::Bitbucket);
        assert_eq!(Platform::parse("gitee"), Platform::Gitee);
        assert_eq!(Platform::parse("redmine"), Platform::Redmine);
        assert_eq!(
            Platform::parse("jira"),
            Platform::Custom("jira".to_string())
        );
    }

    #[test]
    fn returns_platform_names() {
        let cases = [
            (Platform::GitHub, "github"),
            (Platform::GitLab, "gitlab"),
            (Platform::Bitbucket, "bitbucket"),
            (Platform::Gitee, "gitee"),
            (Platform::Redmine, "redmine"),
            (Platform::Custom("jira".to_string()), "jira"),
        ];

        for (platform, name) in cases {
            assert_eq!(platform.name(), name);
        }
    }
}
