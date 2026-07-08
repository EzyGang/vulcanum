# Vulcanum

<p align="center">
  <img src="assets/banner.svg" alt="Vulcanum agentic work orchestrator banner">
</p>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/EzyGang/vulcanum/ci.yml?branch=main&style=for-the-badge" alt="CI">
  <img src="https://img.shields.io/github/v/release/EzyGang/vulcanum?style=for-the-badge" alt="GitHub release">
  <img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later">
</p>

---

**Vulcanum** is an opinionated work framework for engineers using AI agents.

It connects your task tracker to isolated execution backends, dispatches agent work to registered workers, keeps runs observable, and gives teams a control plane for what agents can access and how they finish work.


## Table of Contents

- [Why Vulcanum?](#why-vulcanum)
- [What It Looks Like](#what-it-looks-like)
- [Download / Releases](#download--releases)
- [Getting Started](#getting-started)
- [What You Get](#what-you-get)
- [Architecture](#architecture)
- [How It Works](#how-it-works)
- [Security & Isolation](#security--isolation)
- [Integrations](#integrations)
- [Roadmap](#roadmap)
- [Development](#development)
- [CI](#ci)
- [Contributing](#contributing)
- [License](#license)

> [!WARNING]
> Vulcanum is pre-1.0 orchestration infrastructure. It is intended for engineers running their own controlled environments, and some integrations, isolation modes, and secret-management paths are still evolving.

---

## Why Vulcanum?

Engineering teams using AI agents face three problems:

1. **Trust:** agents often run with broad infrastructure and secret access.
2. **Control:** teams need a structured way to decide which work agents pick up and how they run it.
3. **Visibility:** agent runs need durable history, status, output, and review links.

Vulcanum puts engineers in charge by turning agent execution into managed work runs.

| Feature | What it helps with |
| --- | --- |
| Agent orchestration | Poll a task tracker, create work runs, dispatch them to workers, and sync status back to the source task. |
| Sandboxed isolation | Run each job on the host, inside Docker, or in a Kata-backed lightweight VM depending on the deployment risk profile. |
| Secret management | Keep credentials centralized in the control plane today, with agent-vault / IronProxy planned for mediated secret access. |
| Multi-tenant control | Model teams, workers, providers, project configs, and work-run ownership in the server instead of ad hoc scripts. |
| Task tracker integration | Route work from configured pickup/progress/target columns and render task context through project prompt templates. |
| Run observability | Track run state, event streams, exit codes, token counts, duration, summaries, blocked reasons, and pull request URLs. |

> [!TIP]
> Vulcanum is most useful when you want agents to work from a queue you already trust: a project board, reviewed task template, and repository access model you control.

---

## What It Looks Like

No product screenshots are checked in yet. The repository includes a branded SVG banner so the README renders with a visual identity while the UI continues to evolve.

```text
Dashboard      Workers      Projects      Providers      Runs
   │             │             │              │             │
   ├─ health     ├─ status     ├─ repos       ├─ GitHub     ├─ status
   ├─ queues     ├─ capacity   ├─ columns     ├─ Kaneo      ├─ events
   └─ activity   └─ isolation  └─ prompts     └─ models     └─ PRs
```

The active frontend is a Preact dashboard for managing workers, projects, providers, and runs.

---

## Download / Releases

Published builds are available from the [GitHub Releases page](https://github.com/EzyGang/vulcanum/releases).

| Platform | Download |
| --- | --- |
| Windows x64 | [Latest release assets](https://github.com/EzyGang/vulcanum/releases/latest) |
| macOS ARM | [Latest release assets](https://github.com/EzyGang/vulcanum/releases/latest) |
| macOS Intel | [Latest release assets](https://github.com/EzyGang/vulcanum/releases/latest) |
| Linux x64 | [Latest release assets](https://github.com/EzyGang/vulcanum/releases/latest) |

Current release automation builds the `vulcanum` CLI and `vulcanum-server` worker daemon binaries. If a packaged build for your platform is not published yet, use the [development setup](#development) below.

> [!NOTE]
> Release artifacts may not be code-signed. Your operating system may warn that the binary is from an unknown publisher.

---

## Getting Started

### Prerequisites

- Node.js 22 with `pnpm`
- Rust stable with Cargo
- PostgreSQL 15+
- Redis
- Docker for container isolation
- Linux KVM and Kata Containers for Kata isolation

> [!NOTE]
> Host execution is the quickest local path. Docker and Kata are the safer paths for running untrusted or semi-trusted agent work.

### Quick Setup

```bash
git clone https://github.com/EzyGang/vulcanum.git
cd vulcanum
pnpm install
```

### Server

```bash
# Configure DATABASE_URL, REDIS_URL, JWT_SECRET, INSTANCE_PASSWORD,
# KANEO_INSTANCE, KANEO_API_KEY, and any GitHub App settings.
# See server/AGENTS.md for the full server environment reference.

pnpm migrate-server-up
cargo run -p vulcanum-server --bin vulcanum-web
```

Start the dispatcher in a second process:

```bash
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

<details>
<summary>GitHub App instructions</summary>

### GitHub App Setup

Vulcanum connects to repositories through a **GitHub App** instead of personal access tokens. This provides repository-scoped, short-lived tokens that are automatically rotated.

#### 1. Create a GitHub App

1. Go to **Settings → Developer settings → GitHub Apps → New GitHub App** in your GitHub account or organization.
2. Fill in the required fields:
   - **GitHub App name:** for example, `Vulcanum App`
   - **Homepage URL:** your instance URL, such as `http://localhost:8080`
   - **Callback URL:** `{your_instance}/api/v1/github/callback`
   - **Webhook:** disabled; Vulcanum does not use webhooks
3. Under **Permissions → Repository permissions**, enable:
   - **Contents:** `Read and write` for cloning and pushing branches
   - **Pull requests:** `Read and write` for creating PRs
4. Under **Where can this GitHub App be installed?**, choose **Any account** or restrict it to your organization.
5. Click **Create GitHub App**.
6. After creation, note:
   - **App ID**
   - **App slug**
   - **Private key** (`.pem` file)

#### 2. Configure the Server

Add these environment variables to your `.env`:

```bash
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY=LS0tLS1CRUdJTi...SA+PRIVATE+KEY...LS0tLS1FTkQ=
GITHUB_APP_SLUG=vulcanum-app
```

> [!TIP]
> The private key must be supplied as a single-line base64-encoded string.
>
> ```bash
> base64 -w0 /path/to/your/private-key.pem
> ```

#### 3. Install the App

1. Start the Vulcanum server.
2. Open the dashboard and navigate to **Projects**.
3. Click **Connect GitHub** to authorize the app.
4. Select the repositories Vulcanum can access, then install.
5. Return to the dashboard. The project form repo selector should show the available repositories.

#### 4. Disconnecting

Revoke access by deleting the installation from the dashboard or uninstalling the app from GitHub account settings.

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

# Or run the daemon directly if the machine is already set up.
vulcanum worker daemon

# Short aliases.
vulcanum wrk setup --instance http://<instance>:8080 --code <code>
vulcanum wrk daemon
```

---

## What You Get

A completed work run leaves a durable record in Vulcanum and usually in the connected task tracker.

```text
work-run/<uuid>/
  status: completed | failed | blocked
  source task: <tracker>/<external_task_ref>
  worker: <registered worker id>
  repository scope: <GitHub installation repositories>
  result:
    pull requests: [https://github.com/org/repo/pull/123]
    exit code: 0
    finish status: completed
    summary: <agent-provided run summary>
    blocked reason: <present when blocked>
    next column: <tracker column selected by completion logic>
  metrics:
    duration_ms: 60000
    tokens_used: 1000
    input_tokens: 0
    output_tokens: 0
    cache_read_tokens: 0
    cache_write_tokens: 0
    model_used: <provider model>
  events:
    agent lifecycle, harness output, progress messages, and cancellation checks
```

The server stores work-run state, result fields, PR URLs, token accounting, and event streams. The task tracker can be moved through configured columns, and PR links can be posted back where the integration supports it.

---

## Architecture

Vulcanum is a monorepo with Rust services, a TypeScript/Preact frontend, PostgreSQL control-plane storage, Redis dispatch coordination, and worker-side SQLite journals.

```mermaid
flowchart LR
  Tracker["Task tracker<br/>Kaneo today"] -->|poll pickup column| Server["Server control plane<br/>actix-web + PostgreSQL"]
  Frontend["Frontend dashboard<br/>Preact + Tailwind"] -->|HTTP API| Server
  Server -->|Redis dispatch flags| Dispatcher["Dispatcher"]
  Dispatcher --> Server
  Worker["Worker daemon<br/>Rust + SQLite journal"] -->|poll / ack / events / result| Server
  Worker -->|run job| Harness["Isolation harness<br/>Host / Docker / Kata"]
  Harness --> Agent["Agent backend<br/>OpenCode today"]
  Agent --> Repo["Git repository<br/>GitHub App token"]
  Server -->|sync status and PRs| Tracker
```

### Components

| Component | Role | Implementation |
| --- | --- | --- |
| Server | Control plane API, auth, integrations, work-run lifecycle, and dispatcher support | Rust, actix-web, PostgreSQL, Redis |
| Dispatcher | Assigns pending work runs to available workers | Rust binary in the server crate |
| Worker daemon | Polls for jobs, claims work, runs harnesses, reports events and results | Rust, embedded SQLite journal |
| Frontend UI | Dashboard for workers, projects, providers, and runs | Preact, `@preact/signals`, Tailwind CSS v4 |
| Docker agent image | Container image with agent tooling for isolated execution | Docker image under `docker/agent/` |

### Repository Layout

| Package | Path | Technology | Status |
| --- | --- | --- | --- |
| CLI | `cli/` | Rust | Active |
| Worker Server | `worker-server/` | Rust, SQLite | Active |
| Server | `server/` | Rust, PostgreSQL | Active |
| Shared | `shared/` | Rust | Active |
| Frontend | `frontend/` | TypeScript/Preact | Active |

All packages are managed through **pnpm workspaces** and **Turborepo**. Rust crates are also part of the root Cargo workspace.

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

A project configuration defines the tracker workspace, pickup/progress/target columns, repositories, and prompt template. Workers register with the instance, advertise capacity, and receive jobs only after the dispatcher assigns work.

---

## Security & Isolation

Every work item runs inside an isolated environment. The isolation level is configurable per deployment.

| Provider | Isolation | Runtime Flag | Requirements |
| --- | --- | --- | --- |
| **Host** | None; direct process execution | default | OpenCode installed locally |
| **Docker** | Container boundary | `--runtime=runc` | Docker |
| **Kata** | Lightweight VM through KVM | `--runtime=kata-runtime` | Docker + KVM |

Resource limits per job include maximum duration, vCPU count, and memory cap. Containers are destroyed on completion. Worker journals support crash recovery without treating a restarted daemon as a successful run.

### Token Management

Current MVP behavior sends secrets over HTTPS between server and worker for single-user or controlled deployments.

Planned behavior uses **agent-vault / IronProxy**, a sidecar proxy on the worker that mediates secret access so Vulcanum never handles plaintext secrets directly.

> [!WARNING]
> Do not point Vulcanum at repositories, tokens, or infrastructure you would not allow the selected isolation provider to touch. Host mode is convenient, not a sandbox.

---

## Integrations

### Task Trackers

Vulcanum uses an abstracted integration provider layer.

| Tracker | Status |
| --- | --- |
| **Kaneo** | Active |
| Linear | Planned |
| Jira | Planned |
| GitHub Issues | Planned |

Integration providers are configured per project:

- Pickup column: where Vulcanum finds new work.
- Progress column: where tasks move when an agent starts.
- Target column: where tasks move when work completes.
- Prompt template: how task context is rendered for the agent.

### VCS / Repo Connection

Vulcanum connects to repositories through a GitHub App.

| VCS | Status |
| --- | --- |
| **GitHub** | Active, via GitHub App |
| GitLab | Planned |
| Bitbucket | Planned |

When the GitHub App is installed, repositories are selectable from a dropdown in the project form. The GitHub App generates per-repo installation tokens for cloning and PR creation, removing the need to embed personal access tokens in URLs.

### Execution Backends

Vulcanum uses an abstracted `IsolationProvider` trait for agent execution.

| Backend | Status |
| --- | --- |
| **OpenCode** | Active |
| Claude Code | Planned |
| Codex CLI | Planned |

### Repo Readiness Checks

Automated checks for connected repos are planned before work is dispatched. These checks can validate branch protection, CI configuration, required review rules, and other integration requirements.

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

## Development

### Prerequisites

| Tool | Purpose |
| --- | --- |
| Node.js 22 with `pnpm` | Workspace scripts, frontend tooling, Turborepo orchestration |
| Rust stable with Cargo | CLI, server, worker daemon, shared crates |
| PostgreSQL 15+ | Server database and SQLx-backed tests |
| Redis | Dispatcher and worker coordination |
| Docker | Container isolation and local integration testing |
| Kata Containers | Optional Linux lightweight-VM isolation |

### Setup

```bash
git clone https://github.com/EzyGang/vulcanum.git
cd vulcanum
pnpm install
```

Set server environment variables before running migrations or the API:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/vulcanum_db
REDIS_URL=redis://localhost:6379
JWT_SECRET=<long-random-secret>
INSTANCE_PASSWORD=<admin-login-password>
KANEO_INSTANCE=<your-kaneo-instance-url>
KANEO_API_KEY=<your-kaneo-api-key>
```

Run migrations and start the server processes:

```bash
pnpm migrate-server-up
cargo run -p vulcanum-server --bin vulcanum-web
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

Run the frontend dev server:

```bash
pnpm run dev
```

### Common Commands

| Command | Description |
| --- | --- |
| `pnpm install` | Install workspace dependencies. |
| `pnpm run build` | Build every package through Turborepo. |
| `pnpm run lint` | Run Rust clippy and frontend lint tasks. |
| `pnpm run type-check` | Run frontend type checks. |
| `pnpm run validate` | Run the full lint and type-check validation pipeline. |
| `pnpm run test` | Run all repository tests. |
| `pnpm run format` | Format all Rust and frontend code. |
| `pnpm run dev` | Start the frontend development server. |
| `pnpm migrate-server-up` | Apply server database migrations. |
| `pnpm migrate-server-down` | Revert the latest server database migration. |
| `pnpm prep-queries` | Refresh SQLx query metadata after backend query changes. |
| `cargo run -p vulcanum-server --bin vulcanum-web` | Run the HTTP API server. |
| `cargo run -p vulcanum-server --bin vulcanum-dispatcher` | Run the dispatcher process. |
| `cargo run -p vulcanum-worker-server --bin vulcanum-server` | Run the worker daemon binary directly. |
| `cargo run -p vulcanum-cli --bin vulcanum` | Run the CLI from source. |

---

## CI

CI runs `pnpm run validate` and `pnpm run test` through GitHub Actions on every push.

Release automation is manually dispatched and uploads release assets from the Rust release build.

---

## Contributing

Contributions are welcome. Keep changes focused, validate the behavior you touched, and preserve the layered architecture.

### Good First Contributions

Good places to start:

- Documentation fixes and setup notes.
- Frontend UI polish that follows the existing design system.
- Tests for existing service-layer behavior and error handling.
- Worker recovery and error-message improvements.
- Integration provider edge cases.
- README examples, diagrams, and platform-specific setup notes.

### Before Larger Changes

Before starting a larger change, open an issue or draft pull request that explains:

- **Problem:** the user pain, operational gap, or project risk.
- **Approach:** how the change fits the current architecture and existing modules.
- **Tradeoffs:** dependency, runtime, security, data-model, migration, or compatibility risks.
- **Validation:** exact commands, tests, manual scenarios, and services used to prove the change.

This matters most for database schema changes, integration behavior, task state transitions, GitHub App permissions, worker isolation, secret handling, release packaging, and public API contracts.

### Pull Request Checklist

Before opening a pull request, run the checks that match your changes:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

If backend SQLx queries changed, also run:

```bash
pnpm prep-queries
```

### Engineering Guidelines

Core project rules from [AGENTS.md](AGENTS.md):

- Keep the HTTP → service → repository layering intact.
- Put business logic in services, not route handlers or repositories.
- Keep handlers thin and expose services through application state.
- Do not log secrets or leak tokens into prompts, events, or task comments.
- Avoid unnecessary abstraction; prefer boring code that future maintainers can reason about.
- Use structured Rust errors and `tracing`; avoid `unwrap()`, `expect()`, and `panic!()` in production code.
- Keep frontend API casing conversions in the frontend `fetchApi` wrapper instead of adding Rust serde renames.
- Add tests for behavior changes where they protect state transitions, validation, error handling, or business rules.

Read [AGENTS.md](AGENTS.md) and the module-specific `AGENTS.md` file before changing code in `server/`, `worker-server/`, `cli/`, `shared/`, or `frontend/`.

---

## License

Vulcanum is licensed under `AGPL-3.0-or-later`. See [LICENSE](LICENSE).
