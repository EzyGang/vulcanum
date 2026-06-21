#[derive(Clone, Copy, Eq, PartialEq)]
enum ReviewSection {
    Critical,
    Warnings,
    Suggestions,
}

#[must_use]
pub fn review_requires_implementation(review_body: &str) -> bool {
    section_has_actionable_content(review_body, ReviewSection::Critical)
        || section_has_actionable_content(review_body, ReviewSection::Warnings)
}

#[must_use]
fn section_has_actionable_content(review_body: &str, target: ReviewSection) -> bool {
    let mut in_target_section = false;

    for line in review_body.lines() {
        match review_heading(line) {
            Some(section) if section == target => {
                in_target_section = true;
                continue;
            }
            Some(_) if in_target_section => return false,
            Some(_) | None => (),
        }

        if in_target_section && is_actionable_review_line(line) {
            return true;
        }
    }

    false
}

#[must_use]
fn review_heading(line: &str) -> Option<ReviewSection> {
    let heading = line
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_end_matches(':')
        .trim();

    if heading.eq_ignore_ascii_case("CRITICAL") {
        return Some(ReviewSection::Critical);
    }
    if heading.eq_ignore_ascii_case("WARNINGS") {
        return Some(ReviewSection::Warnings);
    }
    if heading.eq_ignore_ascii_case("SUGGESTIONS") {
        return Some(ReviewSection::Suggestions);
    }
    None
}

#[must_use]
fn is_actionable_review_line(line: &str) -> bool {
    let text = line
        .trim()
        .trim_start_matches(['-', '*'])
        .trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.')
        .trim()
        .trim_start_matches("[ ]")
        .trim()
        .trim_end_matches('.')
        .trim();

    if text.is_empty() {
        return false;
    }

    !matches!(
        text.to_ascii_lowercase().as_str(),
        "none" | "n/a" | "no issues" | "no critical issues" | "no warnings" | "nothing"
    )
}
