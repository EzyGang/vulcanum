pub const GITHUB_INSTRUCTION: &str = "\n\nWhen the task is complete:\n\
    1. Create a new branch for your changes (never commit directly to main)\n\
    2. Commit your changes to that branch\n\
    3. Push the branch and create a pull request using `gh pr create` with a descriptive title and body\n\
    4. Call the `finish_run` tool with the PR URL in the `pr_url` field";
