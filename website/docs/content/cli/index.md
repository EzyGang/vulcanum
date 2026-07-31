# CLI

The `vulcanum` CLI manages a local worker and provides authenticated access to control-plane data.

## Install

The release installer installs `vulcanum` and the `vulcanum-server` worker daemon:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh | sh
```

Run:

```bash
vulcanum --help
vulcanum <COMMAND> --help
```

## Command groups

| Command | Purpose |
| --- | --- |
| `login` | Create or replace the saved application session |
| `workers` | List or rename registered workers |
| `runs` | List work runs |
| `projects` | Add and configure projects and repositories |
| `board` | Read and change provider tasks |
| `settings` | Manage team selection, providers, models, and GitHub |
| `skills` | Install or print bundled agent skills |
| `worker` | Set up, run, or remove the local worker daemon |

`vulcanum wrk` is an alias for `vulcanum worker`. The plural `workers` command manages server-side worker records. The singular `worker` command manages the current host.

## Log in

```bash
vulcanum login [--instance <URL>]
```

For a single-user instance, enter the instance password. Scripts can read it from standard input:

```bash
printf '%s' "$VULCANUM_PASSWORD" | \
  vulcanum login --instance https://<host> --password-stdin
```

For a multi-user instance, the CLI opens a browser. Use `--no-browser` to print the URL. You can also exchange an existing one-time code with `--auth-code <CODE>`.

The CLI stores application session state in `~/.vulcanum/app.json`. It does not store access tokens, refresh tokens, or integration credentials in that file.

## Team selection

For commands that accept `--team`, the CLI uses this order:

1. the command `--team <UUID>`;
2. the locally pinned team;
3. the account's first available team.

Pin a team:

```bash
vulcanum settings team set <UUID>
```

Clear the pin:

```bash
vulcanum settings team clear
```

A command-specific `--team` does not change the saved pin.

## Non-interactive credential input

Provider commands support `--credentials-stdin`. Send one JSON object with non-empty string values. Do not put secrets in command arguments because process lists and shell history can expose them.

Example:

```bash
printf '%s' '{"ANTHROPIC_API_KEY":"value"}' | \
  vulcanum settings model-providers add anthropic \
  --credentials-stdin
```

The CLI does not echo credentials or write them to local application state.

## Exit status

A successful command returns status `0`. Authentication, parsing, authorization, network, server-response, and worker lifecycle failures return a nonzero status and an error message.

See the [command reference](reference.md) for all supported commands.
