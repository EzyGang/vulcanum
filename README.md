# Vulcanum


<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/EzyGang/vulcanum/ci.yml?style=for-the-badge" alt="CI">
  <img src="https://img.shields.io/github/v/release/EzyGang/vulcanum?style=for-the-badge" alt="GitHub release">
  <img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later">
</p>

---

**Vulcanum** is an opinionated work framework for engineers using AI agents.

It connects your task tracker to sandboxed execution backends, dispatches work to registered workers, syncs status back to the source of truth, and keeps agent execution observable, bounded, and isolated.

> [!WARNING]
> Vulcanum is active pre-1.0 infrastructure software. Use it first on owned infrastructure with repositories and secrets you are comfortable operating yourself, then tighten isolation and access controls before wider rollout.

---

## Table of Contents

- [Why Vulcanum?](#why-vulcanum)
- [Screenshots](#screenshots)
- [What You Get](#what-you-get)
- [Download / Releases](#download--releases)
- [Getting Started](#getting-started)
- [Architecture](#architecture)
- [How It Works](#how-it-works)
- [Security & Isolation](#security--isolation)
- [Integrations](#integrations)
- [Roadmap](#roadmap)
- [Repository Layout](#repository-layout)
- [Development](#development)
- [CI](#ci)
- [Contributing](#contributing)
- [License](#license)

---

## How It Works

```
Task Tracker (pickup column)  →  Server polls, creates work_run (pending)
                                       ↓
                                  Dispatcher assigns to idle worker (dispatched)
                                       ↓
Worker polls /api/v1/poll     →  Claims via /api/v1/jobs/{id}/ack (running)
                                       ↓
                                  Worker runs harness in isolated environment
                                       ↓
Task Tracker (Review)        ←  Server syncs status + PR comment  ←  Worker POSTs /result
          ↓
GitHub pull-request `closed` webhook → verify all linked PRs are terminal → move ticket to Done
```

## Security & Isolation

Engineering teams adopting AI agents usually hit three problems:

1. **Trust:** agents can end up with broad infrastructure, repository, and secret access.
2. **Control:** teams need a structured way to decide which work agents pick up and how that work is executed.
3. **Visibility:** reviewers need durable evidence of what ran, where it ran, what it changed, and why it stopped.

Vulcanum puts an orchestration layer between the task tracker and the agent runtime.

| Feature | What it helps with |
| --- | --- |
| Agent orchestration | Poll approved task-tracker columns, create work runs, dispatch jobs, and collect results through one control plane. |
| Sandboxed isolation | Run work on host, Docker, or Kata-backed environments with duration, CPU, and memory limits. |
| Secret management | Keep the secret flow explicit today and move toward agent-vault / ironproxy sidecar injection so plaintext credentials are not handed directly to agents. |
| Multi-tenant control | Model teams, workers, projects, providers, and permissions around a central API instead of ad hoc scripts. |
| Task tracker integration | Keep humans in their normal tracker while Vulcanum handles pickup, progress, completion, and PR comments. |
| Run visibility | Preserve status, output, exit codes, PR URLs, token usage, event streams, and worker metadata per work item. |

> [!NOTE]
> Vulcanum is not an agent model, editor, or hosted coding assistant. It is the control plane that decides when agent work is allowed to run, where it runs, and how the result is reported.

---

## Screenshots

The repository does not currently include product screenshots. The banner above is a branded placeholder so the GitHub page renders cleanly until real dashboard, project, worker, and run-detail screenshots are added.

Suggested screenshot targets:

| Screen | Why it matters |
| --- | --- |
| Dashboard | Shows the operational overview for active workers and runs. |
| Projects | Shows task tracker, repository, provider, and prompt-template configuration. |
| Workers | Shows worker registration, status, and isolation settings. |
| Work run detail | Shows artifacts, event stream, token usage, result status, and PR output. |

> [!TIP]
> Screenshot contributions are useful when they show realistic but non-sensitive data. Avoid real tokens, private repository names, internal task titles, and customer data.

---

## What You Get

A completed work run produces an auditable result record rather than a mystery agent session.

Typical run output looks like this conceptually:

```text
work-run/<run-id>/
  status.json              # queued, dispatched, running, succeeded, failed, or timed out
  prompt.md                # rendered task context sent to the agent
  events.jsonl             # lifecycle events from server, dispatcher, and worker
  agent-output.md          # final agent summary or terminal output
  result.json              # exit code, duration, token usage, branch, and PR URL
  artifacts/
    patch.diff             # code changes when available
    logs/
      harness.log          # harness/runtime logs
      agent.log            # agent CLI logs
```

The exact storage backend can evolve, but the contract is stable: operators should be able to answer **what task ran, on which worker, with which repository access, under which isolation policy, and what artifact or PR came out**.

---

## Download / Releases

Published release artifacts are available from the [GitHub Releases page](https://github.com/EzyGang/vulcanum/releases).

| Platform | Release status |
| --- | --- |
| Windows x64 | Not published yet; use the [development setup](#development) to build from source. |
| macOS Apple Silicon | Not published yet; use the [development setup](#development) to build from source. |
| macOS Intel | Not published yet; use the [development setup](#development) to build from source. |
| Linux x64 | [Latest release assets](https://github.com/EzyGang/vulcanum/releases/latest) currently include the un-packaged `vulcanum` CLI and `vulcanum-server` worker daemon binaries. |

Current release automation runs on a self-hosted Linux runner and uploads only the Rust CLI and worker daemon artifacts. Platform-specific installers and archives are planned but are not published yet while the project is pre-1.0.

> [!WARNING]
> Builds may not be code-signed. macOS, Windows, and some Linux desktop environments may warn that a downloaded binary is from an unknown publisher.

---

## Getting Started

### Prerequisites

- Node.js 22 with `pnpm` through Corepack or a direct pnpm install
- Rust stable with Cargo
- PostgreSQL 15+
- Redis
- Docker for container isolation
- Linux KVM plus Kata Containers when using Kata isolation

> [!TIP]
> For a first local run, start with Docker isolation and a local PostgreSQL/Redis pair. Move to Kata after the server, GitHub App, and worker registration flow are healthy.

### Server

```bash
# Install JavaScript workspace dependencies.
pnpm install

# Set up .env with DATABASE_URL, REDIS_URL, JWT_SECRET,
# INSTANCE_PASSWORD, KANEO_INSTANCE, and KANEO_API_KEY.
# See server/AGENTS.md for all supported server variables.

# Run PostgreSQL migrations.
pnpm migrate-server-up

# Start the API server.
cargo run -p vulcanum-server --bin vulcanum-web

# Start the dispatcher in a separate process.
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

<details>
<summary>GitHub App instructions</summary>

### GitHub App Setup

Vulcanum connects to repositories through a **GitHub App** instead of personal access tokens. This provides repository-scoped, short-lived tokens that are automatically rotated.

#### 1. Create a GitHub App

1. Go to **Settings → Developer settings → GitHub Apps → New GitHub App** in your GitHub account or organization.
2. Fill in the required fields:
   - **GitHub App name**: e.g. `Vulcanum App`
   - **Homepage URL**: your instance URL (e.g. `http://localhost:8080`)
   - **Callback URL**: `{your_instance}/api/v1/github/callback`
   - **Webhook URL**: `{your_instance}/api/v1/github/webhook`
   - **Webhook secret**: generate a strong random value and retain it for the server configuration
   - **Webhook active**: enabled
   - **Subscribe to events**: select **Pull request**
3. Under **Permissions → Repository permissions**, enable:
   - **Contents:** `Read and write`, required for cloning and pushing branches
   - **Pull requests:** `Read and write`, required for creating PRs
4. Under **Where can this GitHub App be installed?**, choose **Any account** or restrict it to your organization.
5. Click **Create GitHub App**.
6. After creation, note:
   - **App ID**, the numeric ID shown at the top
   - **App slug**, the URL-friendly name, for example, `vulcanum-app`
   - A generated **Private key** `.pem` file

#### 2. Configure the Server

Add these environment variables to your `.env`:

```bash
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY=LS0tLS1CRUdJTi...SA+PRIVATE+KEY...LS0tLS1FTkQ=
GITHUB_APP_SLUG=vulcanum-app
GITHUB_WEBHOOK_SECRET=replace-with-the-same-random-webhook-secret
```

> [!NOTE]
> The private key must be supplied as a single-line base64-encoded string. Generate it from your `.pem` file:
>
> ```bash
> base64 -w0 /path/to/your/private-key.pem
> ```

#### 3. Install the App

1. Start the Vulcanum server.
2. Open the dashboard and navigate to **Projects**.
3. Click **Connect GitHub**. GitHub will ask you to authorize the app.
4. Select the repositories Vulcanum should access.
5. Return to the dashboard. The repo selector in the project form will now show available repositories.

#### 4. Disconnecting

To revoke access, delete the installation from the dashboard Projects page or uninstall the app from GitHub account settings.

</details>

<details>
<summary>GitHub OAuth and team invites</summary>

### GitHub OAuth Setup

Link-based team invites require multiuser mode and GitHub OAuth. They are disabled when `IS_SINGLE_USER=true` because instance-password deployments do not authenticate GitHub users.

Set the server to multiuser mode and configure a GitHub OAuth app:

```bash
IS_SINGLE_USER=false
GITHUB_OAUTH_CLIENT_ID=your_client_id
GITHUB_OAUTH_CLIENT_SECRET=your_client_secret
GITHUB_OAUTH_REDIRECT_URL=http://localhost:8000/api/v1/auth/github/callback
```

Team owners can then generate short-lived invite links from the Teams page. Invite links are single-use, expire after 30 minutes, and can be accepted only by GitHub-authenticated users.

</details>

### Worker

```bash
# Generate a registration code from the dashboard at /workers.

# Auto-provision the machine and register the worker.
# Linux installs Docker Engine and a systemd service.
# macOS installs Docker Desktop and a launchd service.
vulcanum worker setup --instance http://<instance>:8080 --code <code>

# Linux-only: Kata isolation requires Linux KVM.
vulcanum worker setup --instance http://<instance>:8080 --code <code> --isolation kata

# Or run the daemon directly if already set up.
vulcanum worker daemon

# Short aliases.
vulcanum wrk setup --instance http://<instance>:8080 --code <code>
vulcanum wrk daemon
```

---

## Architecture

Vulcanum is organized as a monorepo with a Rust control plane, a Rust worker daemon, shared Rust types, and a Preact frontend.

```mermaid
flowchart LR
  tracker[Task Tracker\nKaneo today] --> server[Vulcanum Server\nactix-web + PostgreSQL]
  frontend[Frontend UI\nPreact + Signals] --> server
  server --> redis[(Redis dispatch flags)]
  server --> db[(PostgreSQL)]
  dispatcher[Dispatcher\nbackground binary] --> server
  dispatcher --> redis
  worker[Worker Daemon\nSQLite journal] --> server
  worker --> harness[Isolation Harness\nHost / Docker / Kata]
  harness --> agent[Agent Backend\nOpenCode today]
  agent --> repo[Git repository\nGitHub App token]
  worker --> artifacts[Run results\nlogs + PR URL + usage]
  artifacts --> server
  server --> tracker
```

### Server (Control Plane)

- `actix-web` HTTP server with PostgreSQL.
- Two binaries: `vulcanum-web` for the API and `vulcanum-dispatcher` for background dispatch.
- Background poller watches enabled integrations for new tasks.
- Dispatcher assigns pending work runs to available workers through Redis flags.
- Layered architecture: HTTP → service → repository.

### Worker Daemon

- Single binary spawned by the CLI, running a polling loop.
- Embedded SQLite journal for crash-robust job recovery.
- Spawns agents inside sandboxed harnesses: Kata Containers, Docker, or host.
- Reports exit code, PR URL, token usage, duration, and runtime status.

### Frontend UI

- Preact, `@preact/signals`, and Tailwind CSS v4.
- Dashboard, workers, projects, providers, teams, and run management.

### Docker Agent Image

- Lives in `docker/agent/`.
- Builds an image with OpenCode CLI and Kaneo CLI.
- Used by the worker daemon for container isolation.

---

## How It Works

```text
Task Tracker (pickup column)  →  Server polls, creates work_run (pending)
                                       ↓
                                  Dispatcher assigns to idle worker (dispatched)
                                       ↓
Worker polls /api/v1/poll     →  Claims via /api/v1/jobs/{id}/ack (running)
                                       ↓
                                  Worker runs harness in isolated environment
                                       ↓
Task Tracker (in-review)      ←  Server syncs status + PR comment  ←  Worker POSTs /result
```

The task tracker remains the human-facing workflow. Vulcanum owns the execution lifecycle between pickup and review.

---

## Security & Isolation

Every work item runs inside a configurable isolated environment.

| Provider | Isolation | Runtime flag | Requirements |
| --- | --- | --- | --- |
| **Host** | None, direct process execution | default | OpenCode installed locally |
| **Docker** | Container | `--runtime=runc` | Docker |
| **Kata** | Lightweight VM through KVM | `--runtime=kata-runtime` | Docker + KVM |

Resource limits per job include max duration, vCPU count, and memory cap. Containers are destroyed on completion. No persistent state should leak between isolated runs.

### Token Management

Current MVP deployments pass secrets over HTTPS between server and worker. This is acceptable for single-user setups on owned infrastructure, but it is not the final secret model.

Planned: **agent-vault / ironproxy**, a worker-side proxy that mediates secret access so Vulcanum does not hand plaintext credentials directly to agent processes.

> [!WARNING]
> Treat host isolation as a development convenience. Use Docker or Kata for untrusted repositories, broad prompts, or workers with access to sensitive credentials.

---

## Integrations

Vulcanum keeps integrations behind provider interfaces so task trackers, VCS hosts, and execution backends can evolve independently.

### Task Trackers

| Tracker | Status |
| --- | --- |
| **Kaneo** | Active |
| Linear | Planned |
| Jira | Planned |
| GitHub Issues | Planned |

Integration providers are configured per project:

- Pickup column, where new work is discovered.
- Progress column, where work moves after the agent starts.
- Target column, where completed work is sent.
- Prompt template, which renders task context for the agent.

### VCS / Repo Connection

| VCS | Status |
| --- | --- |
| **GitHub** | Active through GitHub App |
| GitLab | Planned |
| Bitbucket | Planned |

When the GitHub App is installed, repositories are selectable from a dropdown in the project form. The GitHub App generates per-repository installation tokens for cloning and PR creation, removing the need to embed personal access tokens in URLs.

### Execution Backends

Vulcanum uses an abstracted `IsolationProvider` trait for agent execution.

| Backend | Status |
| --- | --- |
| **OpenCode** | Active |
| Claude Code | Planned |
| Codex CLI | Planned |

### Repo Readiness Checks

Planned checks will validate connected repositories before work is dispatched: branch protection, CI config, required review rules, and other integration requirements.

---

## Roadmap

- **Agent-vault / IronProxy:** sidecar secret injection, no plaintext tokens in containers.
- **Built-in analysis agents:** nudge, track, and analyze work progress; surface blockers and suggest priorities.
- **Additional task tracker integrations:** Linear, Jira, GitHub Issues.
- **Additional VCS integrations:** GitLab, Bitbucket.
- **Additional execution backends:** Claude Code, Codex CLI.
- **Repo readiness checks:** validate that connected repos meet integration requirements.
- **Multi-tenant auth:** orgs, teams, row-level security.

---

## Repository Layout

| Package | Path | Technology | Status |
| --- | --- | --- | --- |
| CLI | `cli/` | Rust | Active |
| Worker Server | `worker-server/` | Rust, SQLite | Active |
| Server | `server/` | Rust, PostgreSQL | Active |
| Shared | `shared/` | Rust | Active |
| Frontend | `frontend/` | TypeScript/Preact | Active |
| Docker Agent Image | `docker/agent/` | Docker | Active |

All packages are managed through pnpm workspaces and Turborepo. Rust crates are also part of the root Cargo workspace.

---

## Development

### Prerequisites

- Node.js 22 with `pnpm`
- Rust stable with Cargo
- PostgreSQL 15+
- Redis
- Docker, plus Kata Containers and KVM for Kata isolation work

### Setup

1. Install dependencies:

   ```bash
   pnpm install
   ```

2. Create the server environment file with at least:

   ```bash
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/vulcanum
   REDIS_URL=redis://localhost:6379
   JWT_SECRET=replace-me
   INSTANCE_PASSWORD=replace-me
   KANEO_INSTANCE=http://localhost:1337
   KANEO_API_KEY=replace-me
   ```

3. Start PostgreSQL and Redis.
4. Apply database migrations:

   ```bash
   pnpm migrate-server-up
   ```

5. Run the server, dispatcher, frontend, or worker commands you need.

### Common Commands

| Command | Description |
| --- | --- |
| `pnpm install` | Install workspace dependencies. |
| `pnpm run build` | Build Rust crates and frontend packages through Turborepo. |
| `pnpm run validate` | Run linting and type-checking through package tasks. |
| `pnpm run test` | Run the workspace test suite. |
| `pnpm run format` | Format Rust, TypeScript, and workspace files. |
| `pnpm run dev` | Start the frontend development server. |
| `pnpm migrate-server-up` | Apply server PostgreSQL migrations. |
| `pnpm migrate-server-down` | Revert server PostgreSQL migrations. |
| `pnpm prep-queries` | Regenerate SQLx query metadata after backend query changes. |
| `cargo run -p vulcanum-server --bin vulcanum-web` | Start the API server. |
| `cargo run -p vulcanum-server --bin vulcanum-dispatcher` | Start the background dispatcher. |
| `cargo run -p vulcanum-worker-server --bin vulcanum-server` | Start the worker server binary directly. |
| `cargo run -p vulcanum-cli --bin vulcanum` | Run the CLI from source. |

---

## CI

CI runs on GitHub Actions. The main workflow is [`ci.yml`](https://github.com/EzyGang/vulcanum/actions/workflows/ci.yml); releases are created by [`release.yml`](https://github.com/EzyGang/vulcanum/actions/workflows/release.yml).

Before review, run the same project-level checks locally when your environment supports them:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

---

## Contributing

Contributions are welcome when they keep Vulcanum focused on safe, observable agentic work execution.

### Good First Contributions

Good places to start:

- Documentation fixes and setup notes.
- UI polish for dashboard, project, worker, and run-detail screens.
- Clearer worker setup and error messages.
- Tests for existing state transitions, validation, and error handling.
- New task tracker, VCS, or execution backend spike notes before implementation.

### Before Larger Changes

Before starting a larger change, open an issue or draft pull request that explains:

- **Problem:** the user pain, operational gap, or project risk being addressed.
- **Approach:** how the change fits the current server, worker, frontend, or integration architecture.
- **Tradeoffs:** dependency, runtime, isolation, database, security, compatibility, or UX costs.
- **Validation:** the commands, tests, fixtures, or manual scenarios that will prove the change works.

This matters most for isolation behavior, secret handling, database schema, task tracker semantics, GitHub App permissions, worker recovery, release packaging, and generated run artifacts.

### Pull Request Checklist

Before opening a pull request, run the checks that match the change:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

Also run this after backend query changes:

```bash
pnpm prep-queries
```

Include any command that could not run and the exact blocker in the PR description.

### Engineering Guidelines

The full developer conventions live in [AGENTS.md](AGENTS.md). The short version:

- Keep pull requests focused and remove obsolete code instead of leaving compatibility shims without a reason.
- Keep the HTTP → service → repository layering intact for backend work.
- Put business rules in services, not route handlers or repositories.
- Keep repositories thin and map database errors into domain errors.
- Use structured errors and structured logging; do not log secrets.
- Avoid `unwrap()`, `expect()`, `panic!()`, glob imports, and broad re-exports in production Rust.
- Add tests for state transitions, input validation, error handling, and business rules.
- Keep frontend API casing aligned with the shared `fetchApi` wrapper; do not add backend serde renames just to satisfy camelCase.
- Run `pnpm prep-queries` whenever backend SQL query metadata changes.

---

## License

Vulcanum is licensed under `AGPL-3.0-or-later`. See [LICENSE](LICENSE).
