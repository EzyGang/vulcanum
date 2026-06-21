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

pub const GITHUB_INSTRUCTION: &str = "\n\nWhen the task is complete:\n\
    1. Create a new branch for your changes (never commit directly to main)\n\
    2. Commit your changes to that branch\n\
    3. Push the branch and create a pull request using `gh pr create` with a descriptive title and body\n\
    4. Call the `finish_run` tool with PR URLs in the `pr_urls` field";
