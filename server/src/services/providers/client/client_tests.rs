use crate::services::providers::client::column_name_to_slug;

#[test]
fn column_name_to_slug_uses_lowercase_hyphenated_name() {
    assert_eq!(column_name_to_slug("To Do"), "to-do");
    assert_eq!(column_name_to_slug("  In   Progress  "), "in-progress");
}
