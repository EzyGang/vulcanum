use crate::services::work_runs::model::WorkRun;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ReviewSection {
    Critical,
    Warnings,
    Suggestions,
}

#[must_use]
pub(crate) fn review_requires_implementation(review_body: &str) -> bool {
    section_has_actionable_content(review_body, ReviewSection::Critical)
        || section_has_actionable_content(review_body, ReviewSection::Warnings)
}

#[must_use]
pub(crate) fn review_fix_prompt(run: &WorkRun, pr_url: &str, review_body: &str) -> String {
    let task_title = run.task_title.as_deref().unwrap_or("");
    format!(
        "Address the CRITICAL and WARNINGS items from the pull request review for the existing PR.\n\n\
Task title:\n{task_title}\n\n\
Task body:\n{}\n\n\
Existing pull request:\n{pr_url}\n\n\
Review body:\n{review_body}\n\n\
Update the existing pull request branch only. Do not create a new pull request. \
When done, call finish_run with status completed and include this existing PR URL in pr_urls: {pr_url}",
        run.task_body,
    )
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
