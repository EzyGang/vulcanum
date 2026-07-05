use indicatif::{ProgressBar, ProgressStyle};

use std::time::Duration;

pub fn step<T>(name: &str, f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
    progress(&format!("Installing {name}"), name, f)
}

pub fn progress<T>(
    message: &str,
    done_label: &str,
    f: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let style = match ProgressStyle::with_template("{spinner} {msg}") {
        Ok(s) => s.tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        Err(_) => anyhow::bail!("failed to create spinner style"),
    };

    let pb = ProgressBar::new_spinner();
    pb.set_style(style);
    pb.set_message(format!("{message}..."));
    pb.enable_steady_tick(Duration::from_millis(80));

    match f() {
        Ok(value) => {
            pb.finish_and_clear();
            eprintln!("  [OK] {done_label}");
            Ok(value)
        }
        Err(e) => {
            pb.finish_and_clear();
            eprintln!("  [FAIL] {done_label}");
            Err(e)
        }
    }
}

pub fn info(msg: &str) {
    eprintln!("  {msg}");
}

pub fn warn(msg: &str) {
    eprintln!("  [WARNING] {msg}");
}
