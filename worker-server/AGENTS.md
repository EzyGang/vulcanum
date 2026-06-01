# AGENTS.md - vulcanum-worker-server

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Host machine worker daemon that polls the Vulcanum server for jobs and executes them via opencode.

## Architecture

### Layers

| Layer       | Path                   | Responsibility                                         |
| ----------- | ---------------------- | ------------------------------------------------------ |
| Daemon      | `src/daemon/`          | Main loop, polling, job dispatch                       |
| Isolation   | `src/harness/`         | Environment preparation (host, Kata, gVisor)           |
| State       | `src/state/`           | Local SQLite journal for job state                     |

### Isolation Layer

The harness module provides environment isolation via the `IsolationProvider` trait (defined in `vulcanum-shared`).

- `HostIsolation` — runs on the host directly, creating workdir and cloning the repo.
- `DockerIsolation` — runs inside a Docker container with a configurable runtime.
- `KataIsolation` — delegates to `DockerIsolation` with `kata-runtime`.
- `GvisorIsolation` — delegates to `DockerIsolation` with `runsc`.
- `IsolationKind` — enum dispatch selecting the provider at runtime via `create_isolation_provider()`.

The isolation layer only prepares the environment. The runtime execution (`AgentRuntime`) is stubbed with `todo!()` until VLC-40 implements it.

### Daemon Flow

**Startup**: Load worker identity and tokens from `~/.config/vulcanum/worker.json`.

**Poll loop**: Acquire a `tokio::sync::Semaphore` permit (size = `max_concurrent_jobs` received from the server at registration).
- `GET /api/v1/poll` → `job_id` or 204.
- If `job_id`: spawn a background `tokio` task that holds the permit for the job duration.
- The loop never blocks on individual jobs — it returns to polling immediately.
- Fatal API errors (401/403) from spawned tasks are propagated to the main loop via a `watch` channel.

**Job execution**: Fetch job details, ack, insert journal entry, prepare isolation, `todo!()` for runtime execution.

### Concurrent Job Model

- `max_concurrent_jobs` is delivered by the server in `ConnectResponse` and persisted in `WorkerState`.
- A `Semaphore` with that many permits gates concurrent execution.
- If all permits are taken, incoming job IDs are enqueued in a local `VecDeque` and retried on the next poll tick.

### State Journal (SQLite)

Located at `~/.config/vulcanum/worker.db`.

Schema:

```sql
CREATE TABLE job_journal (
    job_id TEXT PRIMARY KEY,
    workdir TEXT NOT NULL,
    container_name TEXT,
    harness_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',
    started_at TEXT NOT NULL,
    finished_at TEXT,
    exit_code INTEGER,
    tokens_used INTEGER,
    pr_url TEXT,
    duration_ms INTEGER,
    error_message TEXT
);
```

Statuses: `running` → `completed` | `failed` | `lost` → `submitted`.

### Composition Pattern

```
create_isolation_provider(harness_type) → IsolationKind
IsolationProvider::prepare() → IsolatedEnvironment
todo!("AgentRuntime not yet implemented — see VLC-40")
IsolationProvider::cleanup()
```

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-worker-server --bin vulcanum-server

# Or from this directory
cd worker-server
cargo run --bin vulcanum-server
```

## Environment Variables

| Variable | Default | Description |
| -------- | ------- | ----------- |
| `VULCANUM_HARNESS` | `host` | Which isolation to use: `host`, `kata`, or `gvisor` |
| `VULCANUM_IMAGE` | `ghcr.io/ezygang/vulcanum/agent:latest` | Docker image for container isolation providers |