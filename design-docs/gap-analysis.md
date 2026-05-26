# Vulcanum Architecture — Gap Analysis & MVP Scope

**Date:** 2026-05-17
**Status:** Revised for MVP — single-user, Linux/Kata, OpenCode-only, Kaneo-driven

---

## MVP Scope Decisions

Vulcanum is a **thin metadata/trigger broker** between Kaneo and OpenCode. It polls Kaneo for tasks, dispatches to sandboxed workers, and syncs status back. It does NOT validate code, run agents itself, or own the task lifecycle.

### What's IN

- Single-user, self-hosted, no auth gating on API (auth code remains in DB for future)
- Linux-only workers with Kata Containers isolation
- OpenCode as the sole agent harness
- Kaneo as the sole task source — per-project configurable opt-in
- HTTP short-polling (in-memory boolean cache flags, stateless)
- Worker auth: short-lived registration codes → token pair (refresh + access), revocable
- Secrets: plain HTTPS (single-user, own infra; agent-vault for v2)
- Status sync: main app moves Kaneo tasks between columns (pickup → in progress → in review)

### What's OUT (for MVP)

- Multi-user, orgs, teams, row-level security
- WebSockets, SSE — HTTP polling only
- Claude Code, Codex CLI harnesses — OpenCode only
- Agent-vault proxy (secrets flow through main app)
- Vulcanum API SKILL.md (agent-native control surface)
- TUI
- macOS support (Kata Containers requires Linux KVM)
- Verifier agent (second OpenCode run)
- CI status checking in main app (it's a broker, not a CI watcher)
- Linear/Jira integrations — Kaneo only

---

## 1. Architecture (MVP)

```
Kaneo                          Server                        Worker
  │                                │                               │
  │  poll "todo" column            │                               │
  │  (per enabled project)         │                               │
  │◄───────────────────────────────│                               │
  │                                │                               │
  │  return tasks (filtered:       │                               │
  │  not yet in Vulcanum DB)       │                               │
  │───────────────────────────────►│                               │
  │                                │                               │
  │                                │  INSERT work_runs              │
  │                                │  (ON CONFLICT DO NOTHING)      │
  │                                │  flip cache flag for worker    │
  │                                │                               │
  │                                │◄──── GET /poll?worker_id=X ───│
  │                                │       (checks bool cache)      │
  │                                │──── 200 {job_id} ────────────►│
  │                                │                               │
  │                                │◄──── GET /jobs/:id ───────────│
  │                                │──── 200 {work_spec} ─────────►│
  │                                │    (prompt, secrets, config)   │
  │                                │                               │
    │                                │       Worker:                 │
    │                                │       - writes OpenCode config │
    │                                │       - runs Kata container    │
    │                                │       - spawns OpenCode        │
    │                                │       - waits for completion   │
  │                                │                               │
  │  PATCH task → "in progress"    │                               │
  │◄───────────────────────────────│                               │
  │                                │                               │
  │                                │◄──── POST /result ────────────│
  │                                │     {pr_url, usage, exit_code} │
  │                                │                               │
  │  PATCH task → "in review"      │                               │
  │  post comment (PR link)        │                               │
  │◄───────────────────────────────│                               │
  │                                │                               │
    │                                │       container destroyed      │
```

---

## 2. API Surface

### Worker Endpoints

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/workers/connect` | Register worker with short-lived code → returns `{access_token, refresh_token}` |
| `GET` | `/poll?worker_id=X` | Lightweight check — hits in-memory boolean cache. 200 `{job_id}` or 204 No Content |
| `GET` | `/jobs/:id` | Full work spec: prompt, secrets, config, repo info |
| `POST` | `/jobs/:id/ack` | Worker acknowledges receipt, status → `running` |
| `POST` | `/jobs/:id/progress` | Optional: streaming progress updates |
| `POST` | `/jobs/:id/result` | Final: `{pr_url, exit_code, tokens_used, duration_ms}` |
| `POST` | `/workers/refresh` | Refresh access token |

### User/Config Endpoints

| Method | Path | Purpose |
|---|---|---|
| `GET/POST` | `/projects` | List/add Kaneo project configs |
| `PUT` | `/projects/:id` | Toggle enabled, update status mappings, prompt template |
| `POST` | `/projects/:id/columns` | Fetch available columns from Kaneo (for mapping UI) |
| `POST` | `/workers/codes` | Generate a 10-min registration code |
| `GET` | `/workers` | List registered workers, status |
| `DELETE` | `/workers/:id` | Revoke credentials, disconnect worker |
| `GET` | `/runs` | Work run history |

---

## 3. Data Model

```sql
-- Which Kaneo projects Vulcanum watches
CREATE TABLE project_configs (
    id UUID PRIMARY KEY,
    kaneo_project_id TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    pickup_column TEXT NOT NULL DEFAULT 'todo',
    target_column TEXT NOT NULL DEFAULT 'in review',
    progress_column TEXT NOT NULL DEFAULT 'in progress',
    prompt_template TEXT NOT NULL,
    repo_url TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Registered worker machines
CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    access_token_hash TEXT NOT NULL,
    access_expires_at TIMESTAMPTZ NOT NULL,
    last_seen TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'idle',  -- idle, busy, disconnected
    capabilities JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Work runs (not canonical tasks — those live in Kaneo)
CREATE TABLE work_runs (
    id UUID PRIMARY KEY,
    external_task_ref TEXT NOT NULL,
    project_config_id UUID NOT NULL REFERENCES project_configs(id),
    worker_id UUID REFERENCES workers(id),
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, dispatched, running, completed, failed, stalled
    prompt_text TEXT NOT NULL,
    result_pr_url TEXT,
    result_exit_code INTEGER,
    tokens_used INTEGER,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_active_task UNIQUE (external_task_ref, status)
        WHERE status IN ('pending', 'dispatched', 'running')
);
```

---

## 4. Worker Isolation (Kata Containers)

- Worker machine needs Docker + `kata-runtime` pre-installed
- Pre-built container image with OpenCode, git, curl, SSH
- Per-work Kata container:
  - Ephemeral workdir mounted as Docker volume
  - Network egress-only via Kata network policy
  - CPU/memory limits via Docker resource flags
  - Secrets injected as container environment variables
- Container destroyed on completion — all state gone

---

## 5. Prompt Template

Stored per-project in `project_configs.prompt_template`:

```
Task: {task_title}
Description: {task_body}

Repository: {repo_url}
Branch: {branch}

Complete this task and submit a PR. Follow conventions in the repository's AGENTS.md.
```

`{task_title}`, `{task_body}`, `{repo_url}`, `{branch}` interpolated from Kaneo task + project config.

---

## 6. Status Mapping (User-Configurable)

Per Kaneo project, user configures three column slugs:

- **Pickup column** — which column to poll for new work (e.g. "Todo")
- **Progress column** — set when worker starts (e.g. "In Progress")
- **Target column** — set when work completes (e.g. "In Review")

Fetched live from Kaneo API. If a project lacks a progress column, that transition is skipped.

---

## 7. Implementation Priority (MVP)

| Priority | Item | Rationale |
|---|---|---|
| **P0** | Server background poller + Kaneo integration | Source of work — nothing happens without it |
| **P0** | Work run DB schema + unique constraint | Idempotency, duplicate prevention |
| **P0** | Worker auth: registration codes → token pair | Workers can't connect without it |
| **P0** | HTTP polling endpoints (worker ↔ server) | Core communication channel |
| **P0** | Project config CRUD + status column mapping | Users configure what gets automated |
| **P1** | Worker daemon: poll loop, Kata container run | Actual work execution |
| **P1** | OpenCode harness adapter + prompt rendering | Agent execution |
| **P1** | Result submission + Kaneo status sync | Close the loop |
| **P2** | In-memory boolean cache for poll endpoint | Optimize — avoid DB hits on every short poll |
| **P2** | Worker heartbeat / liveness (last_seen, timeouts) | Prevent silent failures |
| **P2** | CLI: `vulcanum connect` command | Bootstrap flow for workers |
| **P3** | Worker setup script (Docker + kata-runtime) | Automation for worker machine setup |
| **P3** | Token rotation / revocation UI | Security lifecycle |
