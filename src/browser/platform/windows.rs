use super::super::BrowserCommand;

pub(super) fn command(url: &str) -> BrowserCommand {
    BrowserCommand {
        program: "cmd",
        args: vec![
            "/C".to_string(),
            "start".to_string(),
            String::new(),
            url.to_string(),
        ],
    }
}
