use std::io::{self, Write};

use std::process::ExitCode;

use nixpkgs_context::run;

fn main() -> ExitCode {
    let report = match run() {
        Ok(report) => report,
        Err(e) => {
            let mut stderr = std::io::stderr().lock();
            drop(writeln!(stderr, "error: {e}"));
            return ExitCode::FAILURE;
        }
    };

    let mut stdout = std::io::stdout().lock();
    if !report.errors.is_empty() {
        let mut stderr = io::stderr().lock();
        for err in &report.errors {
            let _ = writeln!(stderr, "{err}");
        }
    }
    match write!(stdout, "{report}") {
        Err(err) if err.kind() != std::io::ErrorKind::BrokenPipe => ExitCode::FAILURE,
        _ => {
            drop(stdout);
            ExitCode::SUCCESS
        }
    }
}
