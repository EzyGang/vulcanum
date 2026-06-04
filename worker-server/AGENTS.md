# AGENTS.md - vulcanum-worker-server

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Host machine worker daemon that polls the Vulcanum server for jobs, executes them via opencode, and captures session message history.

## Architecture

### Layers

| Layer       | Path                   | Responsibility                                         |
| ----------- | ---------------------- | ------------------------------------------------------ |
| Daemon      | `src/daemon/`          | Main loop, polling, job dispatch                       |
| Isolation   | `src/harness/`         | Environment preparation (host, Docker, Kata, gVisor)   |
| State       | `src/state/`           | Local SQLite journal for job state                     |
| Runtime     | `src/runtime/`         | OpenCode service orchestration, agent runtime          |
| OpenCode    | `src/opencode/`        | HTTP client for the opencode server API                |
| Session     | `src/session/`         | Running session wrapper, event handling                |
| Recovery    | `src/recovery/`        | Crash recovery: reconnecting to live sessions          |
| Storage     | `src/storage/`         | Local message history persistence                      |

### Isolation Layer

The harness module provides environment isolation via the `IsolationProvider` trait (defined in `vulcanum-shared`).

- `HostIsolation` — runs on the host directly, creating workdir and cloning the repo.
- `DockerIsolation` — runs inside a Docker container with a configurable runtime.
- `KataIsolation` — delegates to `DockerIsolation` with `kata-runtime`.
- `GvisorIsolation` — delegates to `DockerIsolation` with `runsc`.
- `IsolationKind` — enum dispatch selecting the provider at runtime via `create_isolation_provider()`.

The isolation layer only prepares the environment. The runtime layer handles opencode server launch and session execution.

### Daemon Flow

**Startup**: Load worker config from `~/.vulcanum/config.json` and worker identity from `~/.vulcanum/worker.json`.

**Poll loop**: Acquire a `tokio::sync::Semaphore` permit (size = `max_concurrent_jobs` received from the server at registration).
- `GET /api/v1/poll` → `job_id` or 204.
- If `job_id`: spawn a background `tokio` task that holds the permit for the job duration.
- The loop never blocks on individual jobs — it returns to polling immediately.
- Fatal API errors (401/403) from spawned tasks are propagated to the main loop via a `watch` channel.

**Job execution**: Fetch job details, ack, insert journal entry, prepare isolation, launch opencode server, create session, enter multi-turn loop, submit result, capture message history, cleanup.

### Concurrent Job Model

- `max_concurrent_jobs` is delivered by the server in `ConnectResponse` and persisted in `WorkerState`.
- A `Semaphore` with that many permits gates concurrent execution.
- If all permits are taken, incoming job IDs are enqueued in a local `VecDeque` and retried on the next poll tick.

### State Journal (SQLite)

Located at `~/.vulcanum/worker.db`.

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
    error_message TEXT,
    turn_count INTEGER NOT NULL DEFAULT 0,
    session_id TEXT,
    max_turns INTEGER NOT NULL DEFAULT 1
);
```

Statuses: `running` → `completed` | `failed` | `lost` → `submitted`.

### Message History

After each job completes, the daemon fetches the full message history from the opencode server via `GET /session/:id/message` and persists it to `~/.vulcanum/sessions/{work_run_id}/{session_id}.json` as raw JSON. This happens before the isolation provider is cleaned up (i.e., while the opencode server is still running).

### Composition Pattern

```
create_isolation_provider(config) → IsolationKind
IsolationProvider::prepare() → IsolatedEnvironment
OpenCodeServeRuntime::execute() → RunningSession
run_turn_loop() → multi-turn with continuation prompts
[message history capture via GET /session/:id/message]
IsolationProvider::cleanup()
```

## Configuration

All worker configuration lives in `~/.vulcanum/config.json`. On first run, defaults are written automatically.

| Field | Default | Description |
| ----- | ------- | ----------- |
| `harness` | `"host"` | Which isolation to use: `host`, `docker`, `kata`, or `gvisor` |
| `image` | `"ghcr.io/ezygang/vulcanum/agent:latest"` | Docker image for container isolation providers |
| `log_format` | `null` | Set to `"json"` for JSON-formatted logs |
| `debug` | `false` | Enable debug-level logging |
| `poll_interval_secs` | `15` | Seconds to sleep between polls when no jobs are available |

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-worker-server --bin vulcanum-server

# Or from this directory
cd worker-server
cargo run --bin vulcanum-server
```
