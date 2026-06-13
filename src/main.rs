fn main() {
    if let Err(err) = issue_jumper::cli::run(std::env::args().skip(1).collect()) {
        eprintln!("Issue Jumper failed: {err}");
        std::process::exit(err.exit_code());
    }
}
