# Vulcanum CLI Reference

The `vulcanum` binary manages the local worker and provides authenticated, read-only access to selected control-plane data.

Run `vulcanum --help` or `vulcanum <COMMAND> --help` for generated command help.

## Authentication

App-facing commands require a saved login session:

```bash
vulcanum login [--instance <URL>] [--password-stdin] [--auth-code <CODE>] [--no-browser]
```

| Option | Purpose |
| --- | --- |
| `--instance <URL>` | Select the Vulcanum control-plane instance. An explicit instance replaces the previous app session. |
| `--password-stdin` | Read a single-user instance password from standard input. |
| `--auth-code <CODE>` | Exchange an existing multi-user one-time code. |
| `--no-browser` | Print the multi-user login URL without opening the default browser. |

With no `--instance`, interactive login offers the previously saved instance. Single-user login pins the instance's default team. An implicit multi-user login to the same canonical instance preserves the existing team pin.

App sessions are stored in `~/.vulcanum/app.json`. Access and refresh tokens are never printed by the CLI. App-facing commands refresh the session before issuing read requests.

## Team selection

Commands that accept `--team <UUID>` resolve the effective team in this order:

1. The command's `--team` override
2. The locally pinned team
3. The account's first available team

The command-line override applies to one command and does not change the saved pin.

### Pin a team

```bash
vulcanum settings team set <UUID>
```

The command refreshes the login session, verifies access to the team, and saves the pin locally.

### Clear a team pin

```bash
vulcanum settings team clear
```

On a multi-user instance, this removes the local pin. On a single-user instance, it resets the pin to the migrated default team UUID:

```text
00000000-0000-0000-0000-000000000001
```

## Worker inspection

### List workers

```bash
vulcanum workers list [--team <UUID>]
```

Prints the selected team's workers with these columns:

- worker ID
- name
- status
- last-seen timestamp
- active and maximum job counts

This plural `workers` namespace is separate from the singular `worker` lifecycle namespace.

## Settings inspection

### List settings

```bash
vulcanum settings list [--team <UUID>]
```

Prints a complete snapshot of:

1. Primary and small-model selection
2. Task-tracker connections
3. Model-provider connections and OAuth account metadata
4. GitHub App connection status

The output does not include access tokens, refresh tokens, tracker API keys, model-provider credential values, or URL user information, queries, and fragments.

## Worker lifecycle

The singular `worker` namespace manages the worker installed on the current machine. `wrk` is a visible alias for `worker`.

### Set up and register a worker

```bash
vulcanum worker setup \
  [--instance <URL>] \
  [--code <CODE>] \
  [--force] \
  [--isolation <kata|docker|none>] \
  [--agent-backend <opencode|omp-rpc>]
```

| Option | Purpose |
| --- | --- |
| `--instance <URL>` | Select the control-plane instance. |
| `--code <CODE>` | Use a worker registration code from that instance. |
| `--force` | Register again even when local worker state already exists. |
| `--isolation <VALUE>` | Select `kata`, `docker`, or `none`. Non-interactive setup defaults to Docker when instance and code are supplied. |
| `--agent-backend <VALUE>` | Select `opencode` or `omp-rpc`. |

Without the required non-interactive values, setup prompts for configuration.

### Run the worker daemon

```bash
vulcanum worker daemon
```

Starts the installed worker daemon and waits for it to exit.

### Remove the local worker

```bash
vulcanum worker self-delete
```

Attempts to unregister the worker, stop and remove its service, delete local worker state, and remove worker-owned runtime data.

## Exit behavior

Successful commands exit with status `0`. Parsing failures, missing or expired login sessions, inaccessible teams, network failures, invalid server responses, and worker lifecycle failures return a nonzero status with an error message.

Common authentication recovery messages are:

```text
Not logged in. Run `vulcanum login`.
Login expired. Run `vulcanum login`.
```
