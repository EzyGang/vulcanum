# Agent Workflows

How Vulcanum automates Kaneo tasks through AI agents — from setup to completion.

## Architecture

```
Kaneo board (pickup column)
        │
        ▼
┌──────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Main App       │────▶│  PostgreSQL   │◀────│  Worker Daemon  │
│ (actix-web)      │     │ (work_runs,   │     │ (CLI binary)    │
│                  │     │  workers,     │     │                 │
│ Poller ──────────┤     │  project_     │     │ Poll ───────────┤
│ (30s interval)   │     │  configs)     │     │ (5s interval)   │
│                  │     │               │     │                 │
│ REST API ────────┤     └──────────────┘     │ Spawn ──────────┤
│ /api/v1/*        │                          │ opencode/prompt │
└──────────────────┘                          └─────────────────┘
        │                                              │
        ▼                                              ▼
┌──────────────┐                              ┌─────────────────┐
│  Frontend UI │                              │  Harness         │
│ (Preact)     │                              │ host / kata      │
└──────────────┘                              └─────────────────┘
```

Two components run independently:
- **Main App** — HTTP server, database, background poller, REST API
- **Worker Daemon** — Single binary per machine; polls for work, runs agents, reports results

Communication is HTTP-only. Workers short-poll a lightweight in-memory flag (not the database) to check for work.

---

## Setting Up Automation

### 1. Connect a Kaneo Project

Use the **Connect Project** form in the dashboard (`/projects/connect`). You need:

| Field | Purpose |
|-------|---------|
| **Kaneo Project ID** | The project slug from Kaneo (e.g. `k5s7dwb5f89anmaui2d814h9`). The form fetches available columns once you enter an ID. |
| **Enabled** | Toggle to start/stop automation for this project. |

#### Column Mapping

Configures which Kaneo columns drive automation state:

| Field | Default | Role |
|-------|---------|------|
| **Pickup Column** | `to-do` | Tasks here are picked up and dispatched to workers |
| **Progress Column** | `in-progress` | Reserved for future use (not automated yet) |
| **Target Column** | `in-review` | Tasks move here on successful completion |

On failure, tasks remain in the Pickup Column for retry.

#### Prompt Template

Template used to generate the agent's work prompt. Three variables are available:

| Variable | Source |
|----------|--------|
| `{{task_title}}` | Kaneo task title |
| `{{task_body}}` | Kaneo task description (markdown body) |
| `{{repo_url}}` | The repo URL configured below |

Example:
```
Review the following task and implement the required changes:

# {{task_title}}

{{task_body}}

Repository: {{repo_url}}
```

The rendered prompt is written to `prompt.md` in the agent's work directory.

#### Repo URL

Git repository URL that Vulcanum clones into the agent's work directory before launching the agent. The clone happens at `workdir/repo/` and the agent is pointed at it via `--dir`.

For private repositories, embed a PAT (personal access token) directly in the URL:
```
https://<token>@github.com/org/repo
```

The agent Docker image includes `git` and `openssh-client` for cloning.

#### Agents.md

Project-specific instructions written to `~/.config/opencode/AGENTS.md` inside the agent's environment. This location is separate from the cloned repository, so a repo's own `AGENTS.md` won't be overwritten — both files are read by opencode.

Use it for:
- Coding conventions and style guides
- Build and test commands
- Environment setup steps
- Any project-specific context the agent needs

---

## Worker Setup

### Registration

1. **Generate a code** in the dashboard (`/workers`) — creates a 16-character one-time code valid for 10 minutes.

2. **Connect the worker machine:**
   ```bash
   vulcanum worker connect http://<instance>:8080 --code <code>
   ```
   This registers the worker, receives JWT credentials, and saves state to `~/.config/vulcanum/worker.json`.

### Running the Daemon

```bash
vulcanum worker daemon
```

The daemon:
- Validates that `opencode` is installed
- Refreshes tokens automatically before expiry
- Polls for work every 5 seconds
- Executes jobs via the configured harness
- Handles SIGINT for graceful shutdown

### Automated Setup (Linux)

```bash
vulcanum worker setup
```

Provisions the machine with:
- Docker
- Kata Containers runtime
- OpenCode binary
- The agent Docker image (`ghcr.io/vulcanum/agent:latest`)
- A systemd service (`vulcanum-worker.service`) configured with `VULCANUM_HARNESS=kata`

---

## Job Execution Flow

```
1. Poller finds task in pickup column
       │
2. Renders prompt template → creates work_run (status: pending)
       │
3. Notifies all workers (in-memory flag)
       │
4. Worker polls → receives job_id
       │
5. Worker GET /jobs/{id} → prompt, repo_url, agents_md
       │
6. Worker POST /jobs/{id}/ack → status: running
       │
7. Worker creates temp directory /tmp/vulcanum-work-{id}/
       │
8. Writes prompt.md to workdir
       │
9. Clones repo into workdir/repo/ (if repo_url is set)
       │
10. Writes AGENTS.md to workdir/home/.config/opencode/AGENTS.md
       │
11. Spawns opencode with HOME=workdir/home --dir workdir/repo --prompt prompt.md
       │
12. opencode does work, may open a PR
       │
13. Worker POST /jobs/{id}/result → exit_code, pr_url, tokens, duration
       │
14. Main app syncs Kaneo:
    - Success → moves task to target column + posts PR comment
    - Failure → task stays in pickup column for retry
       │
15. Worker cleans up temp directory
```

### Concurrency Protection

- Only one active work run per Kaneo task (partial unique index)
- Acknowledge uses `WHERE status = 'pending'` to prevent double-claiming
- Result submission validates worker ownership
- Poller uses `ON CONFLICT DO NOTHING` to skip already-active tasks

---

## Harness System

The harness abstracts where and how `opencode` executes. Selected at runtime via `VULCANUM_HARNESS` env var.

### Host Harness (`VULCANUM_HARNESS=host`, default)

Spawns `opencode` directly on the host machine:
```
HOME=<workdir>/home opencode --prompt <workdir>/prompt.md [--dir <workdir>/repo]
```

The repo is cloned to `<workdir>/repo` before opencode is launched.

Secrets (when implemented) become environment variables on the process.

### Kata Harness (`VULCANUM_HARNESS=kata`)

Spawns `opencode` inside a Kata Containers VM via Docker:
```
docker run --runtime=kata-runtimes --rm \
  -v <workdir>:/workdir \
  -e HOME=/workdir/home \
  --cpus=2 --memory=1024m \
  <image> opencode --prompt /workdir/prompt.md [--dir /workdir/repo]
```

The repo is cloned to `<workdir>/repo` (mounted as `/workdir/repo`) before the container starts.

Resource limits (per job):
| Limit | Default |
|-------|---------|
| Max duration | 1800s (30 min) |
| vCPUs | 2 |
| Memory | 1024 MiB |

Override the Docker image via `KATA_IMAGE` env var. Default: `ghcr.io/vulcanum/agent:latest`.

### Timeout Behavior

If a job exceeds `max_duration_secs`:
1. SIGTERM sent to the process
2. 5-second grace period
3. SIGKILL if still running
4. Reported as failed

---

## Secrets Management

**Current state (MVP):** Secrets infrastructure exists in code but is not yet wired to a storage backend.

- The harness accepts `HashMap<String, String>` for secrets
- Secrets become environment variables on the `opencode` process (host) or `docker run -e` flags (kata)
- There are no secret management CLI commands, API endpoints, or database storage

**Planned (v2):** Agent vault for secret storage and injection.

**Workaround for now:** Include sensitive values (tokens, keys) in the Agents.md field, or use environment variables set on the worker machine.

---

## Monitoring & Control

### Dashboard

The web UI provides:

- **Projects** (`/projects`) — List, connect, edit, delete project configs. Shows Kaneo project ID, enabled status, column mapping, and repo URL per config.
- **Workers** (`/workers`) — Generate registration codes, list connected workers with name/status/last-seen, delete workers.
- **Work Runs** (`/runs`) — Filterable table of all job executions. Shows task reference, status, worker, duration, PR link, and creation time. Filter by status (pending, dispatched, running, completed, failed, stalled). Paginated.

### Work Run Statuses

| Status | Meaning |
|--------|---------|
| `pending` | Created by poller, awaiting worker pickup |
| `dispatched` | Reserved (not currently used) |
| `running` | Worker has acknowledged and is executing |
| `completed` | Agent finished successfully (exit code 0) |
| `failed` | Agent exited with non-zero code or timed out |
| `stalled` | Reserved (not currently used) |

### API Reference

All endpoints under `/api/v1`.

**Instance auth** (dashboard) — `POST /auth/instance-login` with instance password, receive Bearer token.

**Worker auth** (JWT) — `POST /workers/connect` with one-time registration code.

| Scope | Method | Path | Purpose |
|-------|--------|------|---------|
| Public | `POST` | `/auth/instance-login` | Dashboard login |
| Public | `POST` | `/workers/connect` | Worker registration |
| Public | `POST` | `/workers/refresh` | Refresh worker JWT |
| Public | `GET` | `/status` | Token TTL config info |
| Worker | `GET` | `/poll` | Check for work (returns `job_id` or 204) |
| Worker | `GET` | `/jobs/{id}` | Get job payload |
| Worker | `POST` | `/jobs/{id}/ack` | Acknowledge → running |
| Worker | `POST` | `/jobs/{id}/result` | Submit result |
| Instance | `POST` | `/workers/codes` | Generate registration code |
| Instance | `GET` | `/workers` | List workers |
| Instance | `DELETE` | `/workers/{id}` | Delete worker |
| Instance | `GET` `/POST` | `/projects` | List / Create project configs |
| Instance | `GET` `/PUT` `/DELETE` | `/projects/{id}` | CRUD project config |
| Instance | `GET` | `/projects/columns` | Fetch Kaneo columns |
| Instance | `GET` | `/runs` | List work runs |

---

## Environment Variables

### Main App

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | Yes | — | Secret for worker JWT signing |
| `INSTANCE_PASSWORD` | Yes | — | Password for dashboard login |
| `KANEO_INSTANCE` | No | `cloud.kaneo.app` | Kaneo API hostname |
| `KANEO_API_KEY` | No | (empty) | Kaneo API key |
| `MAX_CONNS` | No | `32` | PostgreSQL pool size |
| `POLL_PERIOD_SECS` | No | `30` | Kaneo poll interval (seconds) |
| `STALE_WORKER_THRESHOLD_SECS` | No | `120` | Seconds before worker marked disconnected |

### Worker CLI

| Variable | Default | Purpose |
|----------|---------|---------|
| `VULCANUM_HARNESS` | `host` | Harness selection: `host` or `kata` |
| `KATA_IMAGE` | `ghcr.io/vulcanum/agent:latest` | Docker image for Kata harness |
| `RUST_LOG` | `info` | Tracing level filter |
| `LOG_FORMAT` | (text) | Set to `json` for structured logging |

---

## CLI Reference

```
vulcanum worker connect <instance> --code <code>
    Register a worker machine with the main app.

vulcanum worker daemon
    Start the worker event loop. Polls for work, executes jobs, reports results.

vulcanum worker setup
    Provision the machine: installs Docker, Kata Containers, OpenCode,
    pulls the agent image, configures systemd service.
```

Short aliases are available:
```
vulcanum wrk connect ...
vulcanum wrk daemon
vulcanum wrk setup
```
