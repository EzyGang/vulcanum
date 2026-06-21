use crate::services::poller::prompts::ENVIRONMENT_INSTRUCTION;

#[test]
fn environment_instruction_requires_repo_format_validation_and_tests() {
    assert!(ENVIRONMENT_INSTRUCTION.contains("formatter"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("validation"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("test commands"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("every repository you changed"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("changed repository directory"));
    assert!(ENVIRONMENT_INSTRUCTION.contains("AGENTS.md"));
}
