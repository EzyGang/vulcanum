# AGENTS.md - vulcanum-worker-server

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Host machine worker daemon that polls the Vulcanum server for jobs, executes them via an agent provider, and captures session message history.

## Architecture

### Layers

| Layer       | Path                         | Responsibility                                         |
| ----------- | ---------------------------- | ------------------------------------------------------ |
| Daemon      | `src/daemon/`                | Main loop, polling, job dispatch                       |
| Isolation   | `src/isolation/`             | Environment preparation (host, Docker, Kata)         |
| State       | `src/state/`                 | Local SQLite journal for job state                     |
| Providers   | `src/providers/<name>/`      | Agent backend implementations (OpenCode, future Codex) |
| Recovery    | `src/recovery/`              | Crash recovery: reconnecting to live sessions          |
| Storage     | `src/storage/`               | Local message history persistence                      |

### Isolation Layer

The isolation module provides environment isolation via the `IsolationProvider` trait (defined in `vulcanum-shared`).

| File / Directory | Role |
|------------------|------|
| `factory.rs`     | `IsolationKind` enum + `create_isolation_provider()` dispatch |
| `github_credentials.rs` | GitHub credential bridge: token file, git config, askpass, gh wrapper, env builders |
| `workspace.rs`   | Workdir creation, env-file setup, repo clone, finish tool install |
| `providers/`     | Concrete isolation implementations |

Provider implementations:

- `HostIsolation` — runs on the host directly, creating workdir and cloning the repo.
  > ⚠️ Host isolation provides no security boundary. Jobs share the host UID, filesystem, and network namespace. Use only in single-tenant or trusted environments where users accept the risk of arbitrary code execution on the host. Host mode is designed for correctness (parallel jobs do not interfere) rather than security isolation.
- `DockerIsolation` — runs inside a Docker container with a configurable runtime.
- `KataIsolation` — delegates to `DockerIsolation` with `kata-runtime`.
- `IsolationKind` — enum dispatch selecting the provider at runtime via `create_isolation_provider()`.

The isolation layer only prepares the environment. The provider layer handles agent server launch and session execution.

### Provider Layer

Each agent backend lives under `src/providers/<name>/`. The only current backend is **OpenCode** (`src/providers/opencode/`).

| File | Role |
|------|------|
| `api.rs` | HTTP client for session CRUD, messages, status |
| `events.rs` | SSE event stream connection |
| `health.rs` | Health-check probe |
| `spawn.rs` | Launch opencode server in host or container |
| `runtime.rs` | `OpenCodeServeRuntime` implementing `AgentRuntime` |
| `runner.rs` | `OpenCodeRunningSession` struct + constructor |
| `runner_session.rs` | `impl RunningSession` trait |
| `event_mapper.rs` | SSE event → `AgentEvent` mapping |
| `reporter.rs` | Fire-and-forget event reporting to Vulcanum server |
| `cleanup.rs` | Container removal helper |

Future backends (e.g. Codex) add a sibling directory under `src/providers/` and are wired in by the daemon's job orchestrator.

### Daemon Job Files

| File | Role |
|------|------|
| `orchestrate.rs` | Main job orchestration: fetch, ack, isolation, run, store, cleanup |
| `turn_loop.rs` | Multi-turn session loop with continuation prompts |
| `submit.rs` | Submit results (failed or completed) to the Vulcanum server |
| `artifact.rs` | Read and parse the agent's `finish_artifact.json` |
| `prompts.rs` | Continuation prompt text generation |
| `finish_tool.rs` | Embedded TypeScript `finish_run` tool that agents must call to signal completion |

### Daemon Flow

**Startup**: Load worker config from `~/.vulcanum/config.json` and worker identity from `~/.vulcanum/worker.json`.

**Poll loop**: Acquire a `tokio::sync::Semaphore` permit (size = `max_concurrent_jobs` received from the server at registration).
- `GET /api/v1/poll` → `job_id` or 204.
- If `job_id`: spawn a background `tokio` task that holds the permit for the job duration.
- The loop never blocks on individual jobs — it returns to polling immediately.
- Fatal API errors (401/403) from spawned tasks are propagated to the main loop via a `watch` channel.

**Job execution**: Fetch job details, ack, insert journal entry, prepare isolation, launch agent server, create session, enter multi-turn loop, submit result, capture message history, cleanup.

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

After each job completes, the daemon fetches the full message history from the agent server via `GET /session/:id/message` and persists it to `~/.vulcanum/sessions/{work_run_id}/{session_id}.json` as raw JSON. This happens before the isolation provider is cleaned up (i.e., while the agent server is still running).

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
| `harness` | `"host"` | Which isolation to use: `host`, `docker`, or `kata` |
| `image` | `"ghcr.io/ezygang/vulcanum/agent:latest"` | Docker image for container isolation providers |
| `log_format` | `null` | Set to `"json"` for JSON-formatted logs |
| `debug` | `false` | Enable debug-level logging |
| `poll_interval_secs` | `30` | Seconds to sleep between polls when no jobs are available |
| `auto_update_enabled` | `false` | Enable verified automatic updates of the CLI and worker daemon release pair |
| `update_check_interval_secs` | `86400` | Seconds between automatic update checks |

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-worker-server --bin vulcanum-server

# Or from this directory
cd worker-server
cargo run --bin vulcanum-server
```
