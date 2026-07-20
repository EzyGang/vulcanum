---
name: vulcanum-cli
description: Use whenever an agent needs to use the Vulcanum CLI for any reason. Load this skill before running, explaining, troubleshooting, or composing any `vulcanum` command, including login, team selection, workers, work runs, projects, task boards, settings, integrations, and worker lifecycle.
---

# Vulcanum CLI

Use the repository's `vulcanum` binary rather than calling control-plane APIs directly. The generated command help and `docs/cli-reference.md` are authoritative; consult them before composing an unfamiliar command or when the source may have changed.

## Grounding commands

```bash
vulcanum --help
vulcanum <COMMAND> --help
vulcanum <COMMAND> <SUBCOMMAND> --help
```

From this repository, run an uninstalled development build with:

```bash
cargo run -p vulcanum-cli --bin vulcanum -- <ARGS>
```

Do not guess flags or positional argument order. Check help or `docs/cli-reference.md` when uncertain.

## Authentication and team selection

App-facing commands require a saved session:

```bash
vulcanum login [--instance <URL>] [--password-stdin] [--auth-code <CODE>] [--no-browser]
```

Prefer standard input for secrets. Never place passwords, API keys, access tokens, or integration credentials directly in command arguments, logs, or task descriptions.

Commands accepting `--team <UUID>` resolve the effective team in this order:

1. Explicit `--team` override
2. Locally pinned team
3. First available account team

Manage the pin with:

```bash
vulcanum settings team set <UUID>
vulcanum settings team clear
```

Use `--team` for a one-command override; do not change the persistent pin unless the user wants that behavior.

## Command map

### Inspect workers and runs

```bash
vulcanum workers list [--team <UUID>]
vulcanum runs list [--team <UUID>]
```

The plural `workers` namespace inspects registered workers. The singular `worker` namespace controls the worker installed on the current machine.

### Manage projects

```bash
vulcanum projects list [--team <UUID>]
vulcanum projects add [--repo <OWNER/NAME>]... [--team <UUID>]
vulcanum projects add --provider <UUID> --workspace <ID> --project <ID> \
  [--repo <OWNER/NAME>]... [--team <UUID>]
vulcanum projects automation enable <PROJECT_ID> [--team <UUID>]
vulcanum projects automation disable <PROJECT_ID> [--team <UUID>]
vulcanum projects columns set <PROJECT_ID> \
  [--pickup <COLUMN>] [--in-progress <COLUMN>] \
  [--in-review <COLUMN>] [--done <COLUMN>] [--team <UUID>]
vulcanum projects repos list [--team <UUID>]
vulcanum projects repos set <PROJECT_ID> [--repo <OWNER/NAME>]... [--team <UUID>]
vulcanum projects repos set <PROJECT_ID> --clear [--team <UUID>]
```

`PROJECT_ID` is the configured project UUID printed by `vulcanum projects list`, not the provider's external project ID. `projects add` is interactive unless all of `--provider`, `--workspace`, and `--project` are supplied. Repeating `--repo` selects multiple repositories.

### Browse and manage task boards

```bash
vulcanum board view <PROJECT_ID> [--limit <COUNT>] [--team <UUID>]
vulcanum board column <PROJECT_ID> <COLUMN> \
  [--page <NUMBER>] [--page-size <COUNT>] [--team <UUID>]
vulcanum board tasks create <PROJECT_ID> <TITLE> \
  [--body <TEXT> | --body-stdin] [--status <STATUS>] [--priority <PRIORITY>] \
  [--team <UUID>]
vulcanum board tasks get <PROJECT_ID> <TASK> [--team <UUID>]
vulcanum board tasks edit <PROJECT_ID> <TASK> \
  [--title <TITLE>] [--body <TEXT> | --body-stdin] [--team <UUID>]
vulcanum board tasks move <PROJECT_ID> <TASK> <COLUMN> [--team <UUID>]
vulcanum board tasks search <PROJECT_ID> \
  [--query <TEXT>] [--column <COLUMN>] [--label <LABEL>] \
  [--page <NUMBER>] [--page-size <COUNT>] [--team <UUID>]
```

Use `--body-stdin` for multiline task bodies; it conflicts with `--body`. `TASK` accepts a provider task ID or a case-insensitive task slug such as `VLC-42`. Columns accept a name, slug, or provider ID. For creating tickets with the required body structure, also load the `vulcanum-ticket-template` skill.

### Inspect settings and select models

```bash
vulcanum settings list [--team <UUID>]
vulcanum settings models primary set <PROVIDER_KEY> <MODEL_ID> [--team <UUID>]
vulcanum settings models primary clear [--team <UUID>]
vulcanum settings models small set <PROVIDER_KEY> <MODEL_ID> [--team <UUID>]
vulcanum settings models small clear [--team <UUID>]
```

Use `settings list` to obtain stable task-tracker and model-provider IDs. Mutation commands take those IDs, not display names.

### Manage task trackers

```bash
vulcanum settings task-trackers add \
  --name <NAME> --instance-url <URL> [--credentials-stdin] [--team <UUID>]
vulcanum settings task-trackers update <UUID> \
  [--name <NAME>] [--instance-url <URL>] \
  [--credentials-stdin | --prompt-credentials] [--team <UUID>]
vulcanum settings task-trackers remove <UUID> [--team <UUID>]
```

For automation, pass credentials as a JSON object through standard input. Interactive credential prompts require a terminal.

### Manage model providers and GitHub

```bash
vulcanum settings model-providers add <PROVIDER_KEY> \
  [--name <NAME>] [--auth <api-key|none>] [--credentials-stdin] [--team <UUID>]
vulcanum settings model-providers update <UUID> \
  [--name <NAME>] [--auth <api-key|none>] \
  [--credentials-stdin | --prompt-credentials] [--team <UUID>]
vulcanum settings model-providers remove <UUID> [--team <UUID>]
vulcanum settings model-providers connect-openai \
  [--name <NAME>] [--no-browser] [--team <UUID>]
vulcanum settings github connect [--no-browser] [--team <UUID>]
vulcanum settings github disconnect [--team <UUID>]
```

Use `--no-browser` in headless environments. Device and installation flows print a URL or code for the user to complete externally.

### Install or print Vulcanum agent skills

```bash
vulcanum skills install
vulcanum skills install <cli|ticket-template>
vulcanum skills install <cli|ticket-template> --stdout
```

With no skill name, the command invokes the open agent-skills installer through the first available runner among `pnpm`, `npx`, `bunx`, and `yarn`, points it at `EzyGang/vulcanum`, and selects both Vulcanum skills. A named install selects only that skill. The canonical names `vulcanum-cli` and `vulcanum-ticket-template` are also accepted.

`--stdout` requires one skill and writes its complete `SKILL.md` without invoking a package manager. This supports direct redirection:

```bash
vulcanum skills install ticket-template --stdout > ./SKILL.md
```

### Manage the local worker

```bash
vulcanum worker setup \
  [--instance <URL>] [--code <CODE>] [--force] \
  [--isolation <kata|docker|none>] \
  [--agent-backend <opencode|omp-rpc>]
vulcanum worker daemon
vulcanum worker self-delete
```

`vulcanum wrk` is an alias for `vulcanum worker`. Setup prompts when required non-interactive values are absent. `self-delete` is destructive: confirm that removing registration, service state, and worker-owned runtime data is intended before running it.

## Operating procedure

1. Determine whether the command is app-facing or local-worker lifecycle.
2. Check login state for app-facing work; recover with `vulcanum login` when instructed by the CLI.
3. Resolve the team explicitly when ambiguity could affect another tenant.
4. Obtain stable IDs from list commands instead of guessing them.
5. Use stdin for multiline bodies and secrets.
6. Run the narrowest command that performs the requested operation.
7. Check the exit status and output. Report created or mutated resource IDs and slugs.

Successful commands exit with status `0`. Treat parsing, authentication, authorization, network, and lifecycle failures as real failures; do not report completion until the command succeeds.