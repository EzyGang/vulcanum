# Vulcanum

**An opinionated work framework for engineers utilizing AI.**

Vulcanum orchestrates AI agent execution inside isolated environments, giving engineers control over what gets automated, how it runs, and what agents can access. It connects your task tracker to sandboxed execution backends: polling for work, dispatching to workers, syncing status back, keeping agents contained and secrets managed.

## Why Vulcanum?

Engineering teams using AI agents face three problems:

1. **Trust**: agents run with full access to your infrastructure and secrets
2. **Control**: no structured way to decide what work agents pick up and how
3. **Visibility**: no orchestration layer tracking what agents did and why

Vulcanum puts engineers in charge:

- **You define triggers**: connect a task tracker, pick a column, agents only run what you route
- **You control execution**: choose the isolation level (host, Docker, Kata, gVisor), resource limits, timeouts
- **You manage secrets**: tokens never float as plaintext env vars (agent-vault/ironproxy planned for secure injection)
- **You see everything**: full run history, token usage, PR results, event streams per work item

## Architecture

Four components communicate over HTTP:

### Server (Control Plane)

- **actix-web** HTTP server with PostgreSQL
- Two binaries: `vulcanum-web` (API) and `vulcanum-dispatcher` (background dispatcher)
- Background poller watches enabled integrations for new tasks
- Dispatcher assigns pending work runs to available workers via Redis flags
- Full layered architecture (HTTP → Service → Repository)

### Worker Daemon

- Single binary spawned by the CLI, runs a polling loop
- Embedded SQLite journal for crash-robust job recovery
- Spawns agents inside sandboxed harnesses (Kata Containers, gVisor, Docker, or host)
- Reports results: exit code, PR URL, token usage, duration

### Frontend UI

- Preact + @preact/signals + Tailwind CSS v4
- Dashboard, workers, projects, providers, and runs management

### Docker Agent Image

- Lives in `docker/agent/`, builds an image with OpenCode CLI and Kaneo CLI
- Used by worker daemon inside container isolation

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
Task Tracker (in-review)      ←  Server syncs status + PR comment  ←  Worker POSTs /result
```

## Security & Isolation

Every work item runs inside an isolated environment. The isolation level is configurable per deployment:

| Provider | Isolation | Runtime Flag | Requirements |
|----------|-----------|-------------|--------------|
| **Host** | None (direct) | default | OpenCode installed locally |
| **Docker** | Container | `--runtime=runc` | Docker |
| **gVisor** | Application kernel (sandbox) | `--runtime=runsc` | Docker + gVisor |
| **Kata** | Lightweight VM (KVM) | `--runtime=kata-runtime` | Docker + KVM |

Resource limits per job: max duration (default 30 min), vCPU count, memory cap. Containers are destroyed on completion. No persistent state leaks.

### Token Management (Planned)

Current (MVP): secrets flow over HTTPS between server and worker. Acceptable for single-user setups on owned infrastructure.

Planned: **agent-vault / ironproxy**, a sidecar proxy on the worker that mediates secret access so Vulcanum never handles plaintext secrets.

## Integrations

### Task Trackers

Vulcanum uses an abstracted integration provider layer:

| Tracker | Status |
|---------|--------|
| **Kaneo** | Active |
| Linear | Planned |
| Jira | Planned |
| GitHub Issues | Planned |

Integration providers are configured per-project:

- Pickup column (where to find new work)
- Progress column (set when agent starts)
- Target column (set on completion)
- Prompt template (how to render task context for the agent)

### VCS / Repo Connection

Vulcanum connects to repositories through a GitHub App:

| VCS | Status |
|-----|--------|
| **GitHub** | Active (via GitHub App) |
| GitLab | Planned |
| Bitbucket | Planned |

When the GitHub App is installed, repos are selectable from a dropdown in the project form. No manual URL entry needed. The GitHub App generates per-repo installation tokens for cloning and PR creation, removing the need to embed PATs in URLs.

### Execution Backends

Vulcanum uses an abstracted `IsolationProvider` trait for agent execution:

| Backend | Status |
|---------|--------|
| **OpenCode** | Active |
| Claude Code | Planned |
| Codex CLI | Planned |

### Repo Readiness Checks (Planned)

Automated checks for connected repos before work is dispatched: validating branch protection, CI config, required review rules, and other integration requirements.

## Roadmap

- **Agent-vault / IronProxy**: sidecar secret injection, no plaintext tokens in containers
- **Built-in analysis agents**: nudge, track, and analyze work progress; surface blockers and suggest priorities
- **Additional task tracker integrations**: Linear, Jira, GitHub Issues
- **Additional VCS integrations**: GitLab, Bitbucket
- **Additional execution backends**: Claude Code, Codex CLI
- **Repo readiness checks**: validate that connected repos meet integration requirements
- **Multi-tenant auth**: orgs, teams, row-level security

## Repository Layout

| Package | Path | Technology | Status |
|---------|------|------------|--------|
| CLI | `cli/` | Rust | Active |
| Worker Server | `worker-server/` | Rust, SQLite | Active |
| Server | `server/` | Rust, PostgreSQL | Active |
| Shared | `shared/` | Rust | Active |
| Frontend | `frontend/` | TypeScript/Preact | Active |

All packages are managed via **pnpm workspaces** and **Turborepo**. Rust crates are also part of a Cargo workspace.

## Getting Started

### Prerequisites

- PostgreSQL 15+
- Redis
- Node.js 20+ with pnpm
- Rust toolchain (see `rust-toolchain.toml`)
- (Worker) Docker + Kata/gVisor runtime for containerized isolation

### Server

```bash
# Clone and install
pnpm install

# Set up .env with DATABASE_URL, REDIS_URL, JWT_SECRET, INSTANCE_PASSWORD, KANEO_INSTANCE, KANEO_API_KEY
# See server/AGENTS.md for all supported env vars

# Run migrations
pnpm migrate-server-up

# Start API server
cargo run -p vulcanum-server --bin vulcanum-web

# Start dispatcher (in a separate process)
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

### Worker

```bash
# Generate a registration code from the dashboard (/workers)

# Auto-provision the machine (installs Docker, Kata/gVisor, OpenCode, agent image, systemd service)
vulcanum worker setup --instance http://<instance>:8080 --code <code> --isolation kata

# Or run the daemon directly if already set up
vulcanum worker daemon

# Short aliases
vulcanum wrk setup --instance http://<instance>:8080 --code <code>
vulcanum wrk daemon
```

## Development

```bash
pnpm install              # Install dependencies
pnpm run build            # Build everything (Rust + frontend)
pnpm run validate         # Lint + type-check (clippy, biome, tsc)
pnpm run test             # Run all tests
pnpm run format           # Format everything
pnpm run dev              # Frontend dev server
pnpm migrate-server-up    # Run database migrations
pnpm migrate-server-down  # Revert database migrations
pnpm prep-queries         # Prepare SQLx query cache
```

## CI

CI runs on every push: `pnpm run validate` (format, clippy, lint, type-check) and `pnpm run test`.