use std::borrow::Cow;
use std::time::Duration;

use unicode_width::UnicodeWidthStr;

use indicatif::{ProgressBar, ProgressStyle};

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

#[must_use]
pub fn escape_terminal(value: &str) -> Cow<'_, str> {
    if !value.chars().any(char::is_control) {
        return Cow::Borrowed(value);
    }

    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control.is_control() => {
                escaped.push_str(&format!("\\u{{{:04X}}}", control as u32));
            }
            character => escaped.push(character),
        }
    }
    Cow::Owned(escaped)
}

#[must_use]
pub fn redact_url(value: &str) -> String {
    let (mut url, scheme_less) = match url::Url::parse(value) {
        Ok(url) if url.has_host() => (url, false),
        _ => match url::Url::parse(&format!("https://{value}")) {
            Ok(url) if url.has_host() => (url, true),
            _ => return "[invalid URL]".to_owned(),
        },
    };

    if url.set_username("").is_err() || url.set_password(None).is_err() {
        return "[invalid URL]".to_owned();
    }
    url.set_query(None);
    url.set_fragment(None);

    let redacted = url.to_string();
    match scheme_less {
        true => redacted
            .strip_prefix("https://")
            .unwrap_or(redacted.as_str())
            .to_owned(),
        false => redacted,
    }
}

#[must_use]
pub fn render_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let escaped_headers: Vec<String> = headers
        .iter()
        .map(|header| escape_terminal(header).into_owned())
        .collect();
    let escaped_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell| escape_terminal(&cell).into_owned())
                .collect()
        })
        .collect();
    let mut widths: Vec<usize> = escaped_headers
        .iter()
        .map(|header| UnicodeWidthStr::width(header.as_str()))
        .collect();
    for row in &escaped_rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(UnicodeWidthStr::width(cell.as_str()));
            }
        }
    }

    let mut lines = Vec::with_capacity(escaped_rows.len() + 1);
    lines.push(render_table_row(&escaped_headers, &widths));
    lines.extend(
        escaped_rows
            .iter()
            .map(|row| render_table_row(row, &widths)),
    );
    lines.join("\n")
}

fn render_table_row(cells: &[String], widths: &[usize]) -> String {
    let mut line = String::new();
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            line.push_str("  ");
        }
        line.push_str(cell);
        if index + 1 < cells.len() {
            let width = UnicodeWidthStr::width(cell.as_str());
            line.push_str(&" ".repeat(widths[index].saturating_sub(width)));
        }
    }
    line
}
