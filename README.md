# Vulcanum 🔥

**Kaneo-to-OpenCode automation bridge.** Poll Kaneo for tasks, dispatch to sandboxed workers, sync status back. Thin metadata/trigger broker — doesn't validate code, doesn't run agents itself.

## Architecture

![Architecture Diagram](design-docs/architecture-diagram.html)

Vulcanum has two components:

### Main App (Control Panel)
- **actix-web** HTTP server
- **PostgreSQL** for work runs, worker registry, project configs, run history
- Background poller watches enabled Kaneo projects for new tasks
- In-memory boolean cache per worker for lightweight "work pending?" checks
- HTTP API for worker communication and user configuration
- Single-user MVP — auth exists in code/DB but API not gated

### Worker Daemon
- Single binary: `vulcanum connect --instance <url> --code <code>` → registers, gets token pair, daemonizes
- Short-polls main app for pending work (hits cache flag, not DB)
- Spawns OpenCode inside a Firecracker microVM on each work item
- Reports results back, then idles
- Linux-only (requires KVM for Firecracker)

## How It Works

```
Kaneo (todo column)  →  Main App polls, creates work_run  →  Worker polls cache flag
                                                              ↓
                                                         Worker boots Firecracker μVM
                                                         OpenCode does work, submits PR
                                                              ↓
Kaneo (in review)    ←  Main App syncs status + comment  ←  Worker POSTs /result
```

## Key Design Decisions (MVP)

| Domain | Decision |
|---|---|
| **Harness** | OpenCode only |
| **Isolation** | Firecracker microVMs (Linux/KVM required) |
| **Secrets** | Plain HTTPS (single-user, own infra; agent-vault for v2) |
| **Communication** | HTTP polling (in-memory cache flags, stateless) |
| **Task source** | Kaneo only — per-project opt-in with configurable column mapping |
| **Worker auth** | Short-lived registration codes → token pair (refresh + access), revocable |
| **Verification** | Mechanical: PR exists? Worker reports exit code. Main app doesn't validate. |
| **Language** | Rust (workspace: cli, host-server, main-app, shared) |
| **Database** | PostgreSQL via SQLx |

## Repository Layout

| Crate | Purpose | Status |
|---|---|---|
| `main-app/` | Control panel server (actix-web + SQLx) | Active |
| `host-server/` | Worker daemon (polling, Firecracker μVM, harness spawning) | Placeholder |
| `cli/` | Worker bootstrap (`vulcanum connect`) + future TUI | Placeholder |
| `shared/` | Shared types and utilities | Empty |
| `design-docs/` | Architecture analysis, technology research, diagrams | Active |

## Getting Started

```bash
# Build everything
cargo build --workspace

# Run main app (needs DATABASE_URL in .env)
cargo run -p vulcanum-main-app

# Run checks
make check
```

## Design Docs

- [Technology Research](design-docs/technology-research.md) — isolation, secrets, polling, harnesses
- [Gap Analysis](design-docs/gap-analysis.md) — architectural gaps, MVP scope decisions
- [Architecture Diagram](design-docs/architecture-diagram.html) — visual system overview
- [MVP Implementation Tasks](design-docs/mvp-implementation-tasks.md) — what needs to be built
