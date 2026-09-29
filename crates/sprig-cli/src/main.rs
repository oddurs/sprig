use std::process::ExitCode;

const HELP: &str = "\
sprig: plain-text plans for people and coding agents

Usage: sprig [--help | --version]

No commands exist yet. ROADMAP.md says what comes first.

Exit codes: 0 success, 2 usage error.
";

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        None | Some("-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("sprig {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprint!("sprig: unknown argument {other}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}
