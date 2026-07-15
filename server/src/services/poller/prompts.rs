pub const ENVIRONMENT_INSTRUCTION: &str = "\n\n\
You are running in a Debian-based container. Common build tools are pre-installed: \
gcc, make, cmake, python3, nodejs, npm, pnpm (via corepack), and mise. \
If a tool is missing, install it with `apt-get update && apt-get install -y <package>` or `mise install <tool>@<version>`. \
If validation requires a database or service, reproduce it inside the container whenever feasible; \
install and start local dependencies such as PostgreSQL or Redis rather than treating them as external blockers. \
Before finishing, run the formatter, validation, and test commands that apply to every repository you changed. \
Run commands from the changed repository directory, not the wrapper workspace. \
Check package.json, Cargo.toml, pyproject.toml, Makefile, and AGENTS.md files for the exact commands. \
Report external infrastructure as a blocker only when it cannot be reproduced in or reached from the container after reasonable setup. \
In that case, include the command, the unavailable dependency or credential, and the failed setup or access attempt in your finish summary.";

pub const GITHUB_INSTRUCTION: &str = "\n\nGitHub workflow:\n\
    1. Before changing code, inspect the task body for blocker context and linked pull requests. Check out and build on an existing PR only when that PR is explicitly for this same ticket/current task. If the blocker is another ticket's PR, do not check out or modify that blocker PR branch.\n\
    2. For a separate current ticket, create a new branch for your changes from the appropriate base; never commit directly to main, master, or another default branch.\n\
    3. Commit changes to the active work branch.\n\
    4. Push the branch and create a pull request using `gh pr create`. Target the repository default branch, usually main or master, unless the task body says otherwise or blocker resolution requires the PR base to be the branch where the blocking ticket is implemented.\n\
    5. Call the `finish_run` tool with PR URLs in the `pr_urls` field";

pub const REVIEW_GITHUB_INSTRUCTION: &str = "\n\nGitHub review workflow:\n\
    1. Confirm the focus pull request and record its current head commit before reviewing.\n\
    2. Use `gh pr view`, `gh pr diff`, and the repository's validation commands to inspect the change and its checks without modifying the worktree.\n\
    3. Do not create branches, edit files, commit, push, or create another pull request during the review.\n\
    4. Post exactly one comment-only GitHub pull request review for the current head commit. Do not approve or request changes. If a suitable review already exists for that commit, do not duplicate it; if the head changed, review the new commit and post a new review.\n\
    5. Call `finish_run` with the posted or existing review details";
