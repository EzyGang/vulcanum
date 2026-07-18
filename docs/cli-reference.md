# Vulcanum CLI Reference

The `vulcanum` binary manages the local worker and provides authenticated access to selected control-plane data.

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

App sessions are stored in `~/.vulcanum/app.json`. Access tokens, refresh tokens, and integration credentials are never printed or stored there. App-facing commands refresh the session before issuing requests.

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

## Work run inspection

### List work runs

```bash
vulcanum runs list [--team <UUID>]
```

Prints the selected team's work runs in API order with:

- work run ID
- related ticket reference and title
- implementation or pull-request-review type
- current status
- total token usage with input, output, and combined cache usage
- model, duration, and creation timestamp

Missing usage and timing values are rendered as `-`. Ticket titles, references, and model names are terminal-escaped before display.

## Settings

All settings commands use the team-selection precedence described above and require the control-plane permissions for the resolved team. `settings list` prints stable IDs for task trackers and model providers; mutation commands use those IDs rather than names.

### List settings

```bash
vulcanum settings list [--team <UUID>]
```

Prints primary and small-model selection, task-tracker connections, model-provider connections and OAuth metadata, and GitHub App status. The output excludes tokens, credential values, and URL user information, queries, and fragments.

### Select models

```bash
vulcanum settings models primary set <PROVIDER_KEY> <MODEL_ID> [--team <UUID>]
vulcanum settings models primary clear [--team <UUID>]
vulcanum settings models small set <PROVIDER_KEY> <MODEL_ID> [--team <UUID>]
vulcanum settings models small clear [--team <UUID>]
```

`set` requires a connected provider and a model present in that provider's catalog. Provider and model are updated as one pair. `clear` clears both values in the selected pair.

### Manage task trackers

```bash
vulcanum settings task-trackers add --name <NAME> --instance-url <URL> [--credentials-stdin] [--team <UUID>]
vulcanum settings task-trackers update <UUID> [--name <NAME>] [--instance-url <URL>] [--credentials-stdin | --prompt-credentials] [--team <UUID>]
vulcanum settings task-trackers remove <UUID> [--team <UUID>]
```

Add prompts with hidden input for `Task tracker API key` unless `--credentials-stdin` is supplied. Standard-input credentials consume one JSON object:

```json
{"api_key":"value"}
```

Update changes only supplied fields and leaves credentials unchanged unless a credential mode is selected. Remove permanently deletes the task-tracker connection identified by the ID from `settings list`.

### Manage model providers

```bash
vulcanum settings model-providers add <PROVIDER_KEY> [--name <NAME>] [--auth <api-key|none>] [--credentials-stdin] [--team <UUID>]
vulcanum settings model-providers update <UUID> [--name <NAME>] [--auth <api-key|none>] [--credentials-stdin | --prompt-credentials] [--team <UUID>]
vulcanum settings model-providers remove <UUID> [--team <UUID>]
vulcanum settings model-providers connect-openai [--name <NAME>] [--no-browser] [--team <UUID>]
```

API-key creation defaults to hidden prompts for the provider catalog's credential fields. Blank prompted fields are omitted, but at least one value is required. `--credentials-stdin` instead consumes a non-empty JSON object whose field names and values are non-empty strings:

```json
{"ANTHROPIC_API_KEY":"value"}
```

`--auth none` accepts no credential flags and clears stored server-side credentials. Update leaves credentials unchanged unless a credential mode is selected. Replacing credentials on a `none` or device-OAuth provider requires `--auth api-key`. Remove permanently deletes the provider connection but does not silently change team model selections.

OpenAI connection prints the device verification URL and user code, attempts to open the browser unless `--no-browser` is set, and polls until connected or expired. Browser-launch failure is non-fatal because the printed handoff remains usable.

### Manage the GitHub App

```bash
vulcanum settings github connect [--no-browser] [--team <UUID>]
vulcanum settings github disconnect [--team <UUID>]
```

Connect prints the short-lived installation URL and opens it unless `--no-browser` is set. It reports initiation rather than completion because the browser callback persists the installation. Disconnect is idempotent when no installation exists; otherwise it permanently deletes the current installation row.

Credential input is read only after authentication and team resolution. Prompting requires a terminal; scripts and agents must pass `--credentials-stdin`. Secrets are sent once and are never echoed, written to local app state, rendered in success output, or attached to sanitized request errors.

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
