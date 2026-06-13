use super::super::BrowserCommand;

pub(super) fn command(url: &str) -> BrowserCommand {
    BrowserCommand {
        program: "xdg-open",
        args: vec![url.to_string()],
    }
}
