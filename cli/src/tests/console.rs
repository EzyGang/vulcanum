use std::borrow::Cow;

use crate::console::{escape_terminal, redact_url, render_table};

#[test]
fn terminal_controls_are_rendered_as_visible_literals() {
    let clean = "plain text";
    assert!(matches!(escape_terminal(clean), Cow::Borrowed(value) if value == clean));

    assert_eq!(
        escape_terminal("line\ncolumn\treturn\ransi\u{001b}nul\0"),
        r"line\ncolumn\treturn\ransi\u{001B}nul\u{0000}"
    );
}

#[test]
fn urls_are_redacted_without_echoing_invalid_input() {
    assert_eq!(
        redact_url("https://user:password@example.com/path?q=secret#fragment"),
        "https://example.com/path"
    );
    assert_eq!(
        redact_url("user:password@example.com/path?q=secret#fragment"),
        "example.com/path"
    );
    assert_eq!(redact_url("not a hosted URL"), "[invalid URL]");
}

#[test]
fn table_uses_terminal_width_and_has_no_trailing_padding() {
    let table = render_table(
        &["N", "VALUE"],
        vec![
            vec!["a".to_owned(), "1".to_owned()],
            vec!["界".to_owned(), "22".to_owned()],
        ],
    );

    assert_eq!(table, "N   VALUE\na   1\n界  22");
    assert!(table.lines().all(|line| !line.ends_with(' ')));
}
