# Vulcanum

**Kaneo-to-OpenCode automation bridge.** Poll Kaneo for tasks, dispatch to workers, sync status back. Thin metadata/trigger broker — doesn't validate code, doesn't run agents itself.

## Architecture

![Architecture Diagram](design-docs/architecture-diagram.html)

Vulcanum has three components:

### Server (Control Panel)
- **actix-web** HTTP server with PostgreSQL
- Two binaries: `vulcanum-web` (API server) and `vulcanum-dispatcher` (background dispatcher) — both must be running
- Background poller watches enabled Kaneo projects for new tasks
- Dispatcher assigns pending work runs to available workers via Redis flags
- Full layered architecture (HTTP → Service → Repository)

### Worker Daemon
- Single binary: `vulcanum worker connect <instance> --code <code>` — registers, gets token pair
- Daemonized with systemd, runs polling loop
- Short-polls server via `/api/v1/poll`, dispatches jobs, acks them to start execution
- Spawns OpenCode inside a sandboxed harness (Kata Containers or host)
- Reports results back via `/api/v1/jobs/{id}/result`

### Frontend UI
- Preact + @preact/signals + Tailwind CSS v4
- Dashboard, workers, projects, and runs management
- Served via nginx alongside the API

### Docker Agent Image
- Lives in `docker/agent/` — builds an image with OpenCode CLI and Kaneo CLI
- Used by worker daemon inside container isolation (Kata Containers, gVisor, or host)

## How It Works

```
Kaneo (to-do column)   →  Server polls, creates work_run (pending)
                               ↓
                          Dispatcher assigns to idle worker (dispatched)
                               ↓
Worker polls /api/v1/poll  →  Claims via /api/v1/jobs/{id}/ack (running)
                               ↓
                          Worker runs harness, OpenCode does work
                               ↓
Kaneo (in-review)       ←  Server syncs status + PR comment  ←  Worker POSTs /result
```

## Repository Layout

| Package | Path | Technology | Status |
|---------|------|------------|--------|
| CLI | `cli/` | Rust | Active |
| Worker Server | `worker-server/` | Rust | Active |
| Server | `server/` | Rust | Active |
| Shared | `shared/` | Rust | Active |
| Frontend | `frontend/` | TypeScript/Preact | Active |

All packages are managed via **pnpm workspaces** and **Turborepo**. Rust crates are also part of a Cargo workspace.

## Getting Started

```bash
# Run main server (needs DATABASE_URL, REDIS_URL, JWT_SECRET, INSTANCE_PASSWORD in .env)
cargo run -p vulcanum-server --bin vulcanum-web
```

## CI

CI runs on every push via self-hosted runners executing `pnpm run validate` (format, clippy, lint, type-check) and `pnpm run test` (migrations + test suite). Configured in `.github/workflows/ci.yml`.

## Design Docs

- [Technology Research](design-docs/technology-research.md) — isolation, secrets, polling, harnesses
- [Gap Analysis](design-docs/gap-analysis.md) — architectural gaps, MVP scope decisions
- [Architecture Diagram](design-docs/architecture-diagram.html) — visual system overview
- [MVP Implementation Tasks](design-docs/mvp-implementation-tasks.md) — what needs to be built
