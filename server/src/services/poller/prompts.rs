pub const ENVIRONMENT_INSTRUCTION: &str = "\n\n\
You are running in a Debian-based container. Common build tools are pre-installed: \
gcc, make, cmake, python3, nodejs, npm, pnpm (via corepack), and mise. \
If a tool is missing, install it with `apt-get update && apt-get install -y <package>` or `mise install <tool>@<version>`. \
If the project requires a database or service (e.g., PostgreSQL, Redis), \
install and start it inside the container. \
After making changes, run the project's test suite and formatters — \
check package.json, Cargo.toml, pyproject.toml, or Makefile for available commands.";

pub const GITHUB_INSTRUCTION: &str = "\n\nWhen the task is complete:\n\
    1. Create a new branch for your changes (never commit directly to main)\n\
    2. Commit your changes to that branch\n\
    3. Push the branch and create a pull request using `gh pr create` with a descriptive title and body\n\
    4. Call the `finish_run` tool with the PR URL in the `pr_url` field";
