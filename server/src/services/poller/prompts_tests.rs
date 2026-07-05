use crate::services::poller::prompts::{ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION};

#[test]
fn environment_instruction_requires_repo_format_validation_and_tests() {
    assert!(ENVIRONMENT_INSTRUCTION.contains("formatter"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("validation"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("test commands"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("every repository you changed"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("changed repository directory"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("AGENTS.md"));
}

#[test]
fn github_instruction_requires_task_body_and_blocker_context_before_code_changes() {
    assert!(GITHUB_INSTRUCTION.contains("Before changing code"));
    assert!(GITHUB_INSTRUCTION.contains("inspect the task body for blocker context"));
    assert!(GITHUB_INSTRUCTION.contains("linked pull requests"));
}

#[test]
fn github_instruction_limits_existing_pr_checkout_to_same_ticket() {
    assert!(GITHUB_INSTRUCTION.contains("Check out and build on an existing PR only when",));
    assert!(GITHUB_INSTRUCTION.contains("explicitly for this same ticket/current task"));
}

#[test]
fn github_instruction_forbids_using_blocker_pr_branch_for_current_ticket_work() {
    assert!(GITHUB_INSTRUCTION.contains("If the blocker is another ticket's PR"));
    assert!(GITHUB_INSTRUCTION.contains("do not check out or modify that blocker PR branch",));
    assert!(GITHUB_INSTRUCTION
        .contains("For a separate current ticket, create a new branch for your changes",));
}

#[test]
fn github_instruction_requires_default_branch_pr_base_unless_context_overrides() {
    assert!(GITHUB_INSTRUCTION.contains("Target the repository default branch"));
    assert!(GITHUB_INSTRUCTION.contains("usually main or master"));
    assert!(GITHUB_INSTRUCTION.contains("unless the task body says otherwise"));
}

#[test]
fn github_instruction_allows_blocker_branch_only_as_pr_base() {
    assert!(GITHUB_INSTRUCTION.contains("blocker resolution requires the PR base"));
    assert!(GITHUB_INSTRUCTION.contains("branch where the blocking ticket is implemented"));
}

#[test]
fn github_instruction_forbids_committing_directly_to_default_branch() {
    assert!(GITHUB_INSTRUCTION.contains("never commit directly"));
    assert!(GITHUB_INSTRUCTION.contains("main, master, or another default branch"));
}
