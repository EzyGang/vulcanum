pub const ENVIRONMENT_INSTRUCTION: &str = "\n\n\
You are running in a Debian-based container. Common build tools are pre-installed: \
gcc, make, cmake, python3, nodejs, npm, pnpm (via corepack), and mise. \
If a tool is missing, install it with `apt-get update && apt-get install -y <package>` or `mise install <tool>@<version>`. \
If the project requires a database or service (e.g., PostgreSQL, Redis), \
install and start it inside the container. \
Before finishing, run the formatter, validation, and test commands that apply to every repository you changed. \
Run commands from the changed repository directory, not the wrapper workspace. \
Check package.json, Cargo.toml, pyproject.toml, Makefile, and AGENTS.md files for the exact commands. \
If a required command cannot run because of missing external infrastructure, include the command and blocker in your finish summary.";

pub const GITHUB_INSTRUCTION: &str = "\n\nGitHub workflow:\n\
    1. Before changing code, inspect the task body for blocker context and linked pull requests. Check out and build on an existing PR only when that PR is explicitly for this same ticket/current task. If the blocker is another ticket's PR, do not check out or modify that blocker PR branch.\n\
    2. For a separate current ticket, create a new branch for your changes from the appropriate base; never commit directly to main, master, or another default branch.\n\
    3. Commit changes to the active work branch.\n\
    4. Push the branch and create a pull request using `gh pr create`. Target the repository default branch, usually main or master, unless the task body says otherwise or blocker resolution requires the PR base to be the branch where the blocking ticket is implemented.\n\
    5. Call the `finish_run` tool with PR URLs in the `pr_urls` field";
