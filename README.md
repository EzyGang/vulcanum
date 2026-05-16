# Vulcanum 🔥

**Symphony-like agentic work orchestrator.** Dispatch, isolate, execute, and validate AI agent tasks across distributed worker machines.

## Architecture

![Architecture Diagram](design-docs/architecture-diagram.html)

Vulcanum is a distributed system with three main components:

### Main App (Control Panel)
- **actix-web** HTTP server with magic-link authentication
- **PostgreSQL** for work queue, worker registry, audit logs, and encrypted secrets
- **WebSocket server** for real-time bidirectional communication with workers
- REST API for user management, server configuration, and work dispatch

### Worker Daemon
- Lightweight Rust daemon running on user machines (Linux primary)
- Connects to Main App via **outbound WebSocket** (works behind NAT/firewalls)
- Receives work, decrypts secrets, spawns agent harnesses in **sandboxed environments**
- Reports progress, submits results, auto-cleans on completion

### CLI (Dual-Mode)
- **TUI Control Panel:** Dashboard for monitoring workers, viewing work status
- **Control Tool:** Bootstrap — authenticate, register machine, install daemon

## How It Works

```
User creates work → Main App dispatches → Worker picks up via WebSocket
                                          → Worker decrypts secrets (age)
                                          → Spawns agent harness (Claude Code, etc.)
                                          → Inside sandbox (bwrap, tmpfs, no network)
                                          → Secrets injected via memfd (never on disk)
                                          → Results streamed back
                                          → Validation checks run
                                          → Sandbox destroyed, cleanup automatic
```

## Key Design Decisions

| Domain | Decision |
|--------|----------|
| **Isolation** | Tiered: bubblewrap+tmpfs (default) → Podman+gVisor → Firecracker microVMs |
| **Secrets** | age (X25519) wrapping, memfd injection, destroyed on process exit |
| **Communication** | WebSocket (primary), SSE+POST (fallback) over mTLS |
| **Harnesses** | Claude Code P0, Codex CLI P1, pluggable trait for more |
| **Language** | Rust (workspace: cli, host-server, main-app, shared) |
| **Database** | PostgreSQL via SQLx |

## Repository Layout

| Crate | Purpose | Status |
|-------|---------|--------|
| `main-app/` | Control panel server (actix-web + SQLx + auth) | Active |
| `host-server/` | Worker daemon (polling, sandboxing, harness orchestration) | Placeholder |
| `cli/` | TUI + control tool (registration, monitoring) | Placeholder |
| `shared/` | Shared types and utilities | Empty |
| `design-docs/` | Architecture analysis, technology research, diagrams | New |

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
- [Gap Analysis](design-docs/gap-analysis.md) — architectural gaps, recommendations, priority
- [Architecture Diagram](design-docs/architecture-diagram.html) — visual system overview
