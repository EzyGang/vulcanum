# Vulcanum Agent Orchestrator — Technology Research (MVP)

**Date:** 2026-05-17 (MVP revision)
**Status:** Scoped for MVP — Linux-only, Kata Containers, OpenCode, Kaneo

---

## 1. AGENT ISOLATION — Kata Containers

### Why Kata Containers

For MVP, we standardize on Kata Containers. It provides hardware-level isolation (KVM) by running each container inside its own lightweight VM, with an OCI-compatible interface. Used by major cloud providers for secure multi-tenant container workloads.

| Property | Kata Containers |
|---|---|
| **Isolation type** | Lightweight VM per container (KVM) |
| **Isolation strength** | Very High (KVM hardware virtualization) |
| **Startup time** | ~200ms–500ms |
| **Memory overhead** | ~20–30MB per VM + guest kernel |
| **CPU overhead** | Near-native |
| **Disk overhead** | Container image layers |
| **Network isolation** | Configurable, egress-only default |
| **Filesystem** | Docker volume mount, tmpfs workdir |
| **Linux only** | Yes (requires KVM) |
| **Daemon required** | Docker + kata-runtime |

### Per-Work Container Run Flow

1. Worker receives work spec from main app
2. Pre-built Docker image with OpenCode, git, curl, SSH already pulled on worker machine
3. Docker runs container with `--runtime=kata-runtime`:
   - Ephemeral workdir mounted as Docker volume
   - Network egress-only via Kata network policy
   - CPU/memory limits via Docker resource flags
   - Secrets injected as container environment variables
4. OpenCode runs, does work, submits PR, exits
5. Worker collects output, reports result
6. Container destroyed — all state gone

### Worker Machine Setup

```bash
# Docker + kata-runtime must be pre-installed
# Container image pre-built with:
# - OpenCode CLI
# - git + curl + basic tooling
# - SSH/git config for repo access

vulcanum setup-worker  # installs Docker + kata-runtime, pulls container image, configures systemd
```

---

## 2. SECRETS — Plain HTTPS (MVP)

For the single-user, self-hosted MVP, secrets flow through the main app over plain HTTPS. This is acceptable because:

- Single user on their own infrastructure (Tailscale network)
- No multi-tenant attack surface
- Secrets exist in memory on the main app briefly during dispatch

**Flow:**
1. User configures secrets in main app (API keys, GitHub tokens)
2. When dispatching work, main app includes secrets in the work spec response
3. Worker receives secrets in the `/jobs/:id` response
4. Worker injects secrets into Kata VM via env vars
5. OpenCode reads from env

**V2:** Replace with agent-vault proxy on worker (sidecar pattern). Harness reads from `localhost:8200`, Vulcanum never touches plaintext secrets. See gap analysis for full design.

---

## 3. COMMUNICATION — HTTP Short Polling

No persistent connections. Stateless HTTP, horizontally scalable.

### Worker → Server

| Endpoint | Purpose |
|---|---|
| `POST /workers/connect` | Register with short-lived code → token pair |
| `GET /poll?worker_id=X` | Lightweight check (in-memory boolean cache, not DB) |
| `GET /jobs/:id` | Full work spec (prompt, secrets, config) |
| `POST /jobs/:id/ack` | Acknowledge receipt |
| `POST /jobs/:id/result` | Final result (PR URL, exit code, tokens, duration) |
| `POST /workers/refresh` | Refresh access token |

### Server → Kaneo

- Poll Kaneo API per enabled project for tasks in configured "pickup" column
- Filter: only tasks not yet in Vulcanum DB (`ON CONFLICT DO NOTHING`)
- Update task status (→ "in progress", → "in review") via Kaneo API
- Post PR link as comment on task

### Polling Strategy

- Worker polls `/poll` every 15 seconds (configurable)
- Cache flag: boolean per worker, flipped when a new work_run is created for that worker
- If worker is idle and flag is false → 204 No Content (fast, no DB hit)
- Backoff only on server unreachable (exponential, max 60s)

---

## 4. AGENT HARNESS — OpenCode

OpenCode is the sole harness for MVP. It's a CLI tool that:
- Takes a task prompt and a repository context
- Works autonomously to complete the task
- Submits a PR when done
- Reports exit code, token usage, and the PR URL

### Prompt Template

Stored per-project in `project_configs.prompt_template`. Variables interpolated from Kaneo task:

```
Task: {task_title}
Description: {task_body}

Repository: {repo_url}
Branch: {branch}

Complete this task and submit a PR. Follow conventions in the repository's AGENTS.md.
```

OpenCode has its own built-in system prompt — Vulcanum's template provides only the task context.

### Harness Interface (Rust trait)

```rust
trait AgentHarness {
    async fn spawn(
        &self,
        prompt: &str,
        workdir: &Path,
        secrets: &HashMap<String, String>,
        limits: &ResourceLimits,
    ) -> Result<HarnessResult>;
}

struct HarnessResult {
    exit_code: i32,
    tokens_used: u64,
    pr_url: Option<String>,
    duration_ms: u64,
}
```

OpenCode adapter implements this trait. Additional harnesses (Claude Code, Codex CLI) can be added later via the same trait.

---

## 5. WORKER AUTH — Code-Based Registration

### Bootstrap Flow

1. User generates a short-lived registration code in the main app (`POST /workers/codes`)
2. Code is valid for 10 minutes, single-use
3. Worker runs: `vulcanum connect --instance https://vulcanum.example.com --code <code>`
4. Server validates code, generates token pair (access + refresh)
5. Worker stores tokens locally (`~/.vulcanum/credentials`, 0600 permissions)
6. Worker uses access token for all API calls, refreshes when expired
7. User can revoke from main app → worker disconnected, no more jobs dispatched

### Token Lifecycle

- Access token: short-lived (1 hour), used for API calls
- Refresh token: long-lived, stored hashed in DB, used only to get new access tokens
- Revocation: delete from workers table → refresh token invalid → worker can't refresh → disconnected
- Worker status → `disconnected` when `last_seen` exceeds threshold → no new work dispatched

---

## 6. KANEO INTEGRATION

### Project Configuration

Per-project, user configures:
- **Kaneo project ID** — which project to watch
- **Enabled/disabled** — toggle automation on/off
- **Pickup column** — which column to poll (e.g. "Todo")
- **Progress column** — set when worker starts (e.g. "In Progress")
- **Target column** — set when work completes (e.g. "In Review")
- **Prompt template** — how to render tasks for OpenCode
- **Repo URL** — where the code lives

### Polling

- Background `tokio::spawn` with `tokio::time::interval`
- Polls each enabled project every N seconds (configurable, default 30s)
- Fetches tasks in pickup column, filters against Vulcanum DB
- Idempotent: `INSERT ... ON CONFLICT DO NOTHING`

### Status Sync

- When worker acknowledges → PATCH Kaneo task to progress column
- When worker submits result → PATCH Kaneo task to target column + post comment with PR link
- If worker fails/stalls → PATCH Kaneo task back to pickup column with failure note

---

## 7. DATABASE (PostgreSQL)

Work runs, worker registry, and project configs live in PostgreSQL. Tasks are canonical in Kaneo — Vulcanum only stores operational metadata.

Key tables: `project_configs`, `workers`, `work_runs` (see gap analysis for full schema).

Uniqueness constraint prevents re-dispatching the same task while it's active:
```sql
CONSTRAINT unique_active_task UNIQUE (external_task_ref, status)
    WHERE status IN ('pending', 'dispatched', 'running')
```
