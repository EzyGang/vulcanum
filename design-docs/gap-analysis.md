# Vulcanum Architecture — Gap Analysis & Recommendations

**Date:** 2026-05-16 (revised after architecture review)
**Status:** Updated — task manager integration replaces PG work queue, Vulcanum Agent added, polling primary

---

## 1. Gaps Identified

### 1.1 Task Manager Integration (Replaces PG Work Queue) — Critical

**Previous approach:** PostgreSQL as a work queue (`pending → dispatched → running → completed`).

**Revised approach:** Vulcanum does NOT own work items. Work lives in external task managers (Kaneo, Linear, Jira, etc.). Vulcanum is a bridge/orchestrator — it polls the task manager for tasks, dispatches them to workers, and syncs status back.

**Why this change:**
- Vulcanum shouldn't try to be Yet Another Task Manager
- Kaneo, Linear, etc. already have excellent task management UIs, search, labeling, comments — Vulcanum would never catch up
- Users already have workflows in these tools; Vulcanum should plug into them, not replace them

**Architecture:**
```
Kaneo/Linear/Jira          Vulcanum Main App              Worker Daemon
     │                           │                            │
     │  poll for ready tasks     │                            │
     │◄──────────────────────────│                            │
     │                           │                            │
     │  return tasks with        │                            │
     │  vulcanum:ready label     │                            │
     │──────────────────────────►│                            │
     │                           │  dispatch work             │
     │                           │───────────────────────────►│
     │                           │                            │
     │                           │  poll: status/progress     │
     │                           │◄───────────────────────────│
     │                           │                            │
     │  update task status       │                            │
     │  (vulcanum:running)       │                            │
     │◄──────────────────────────│                            │
     │                           │                            │
     │                           │  result submitted          │
     │                           │◄───────────────────────────│
     │                           │                            │
     │  post result as comment   │                            │
     │  update status            │                            │
     │◄──────────────────────────│                            │
```

**What's needed:**
1. **Pollers** for each supported task manager — configurable interval, label/status filter
2. **Webhook listeners** (optional) — for task managers that support push notifications (Linear has webhooks)
3. **Status mapping** — Kaneo column/Linear status ↔ Vulcanum internal state
4. **Bidirectional sync** — Vulcanum updates the task manager when status changes, task manager updates trigger dispatch
5. **Idempotency** — same task can't be dispatched twice (track by external task ID)

**Initial integration target: Kaneo** (self-hosted, CLI available, the user's preferred tool). Linear as secondary (webhooks available).

**What PostgreSQL DOES store (operational metadata, not canonical work items):**
- Worker registry (id, owner, capabilities, status, last_seen)
- Work run history (run_id, external_task_ref, worker_id, harness, model, tokens_used, timestamps, exit_code)
- Secret store (encrypted at rest with age)
- Audit log
- User/org/team settings

### 1.2 Worker Authentication Model — Critical

**Current state:** Magic-link auth exists for human users, but worker daemons are not human users.

**Problem:** How does a headless daemon authenticate? It can't click magic links.

**Recommendation:**
- Workers authenticate via pre-provisioned API keys or mTLS certificates
- Bootstrap flow: User creates a "worker registration token" in the Main App UI → CLI control tool uses this token to register the machine → server issues a persistent worker credential
- Worker stores credential locally (`~/.vulcanum/credentials`) with restrictive file permissions (0600)
- All subsequent connections use this credential. Token rotation supported via server-initiated re-key

### 1.3 CLI ↔ Host Server IPC — Critical

**Current state:** CLI and host-server are separate crates, both placeholders. No IPC defined.

**Recommendation:**
- Host server listens on a Unix domain socket (`/run/vulcanum/daemon.sock`) or localhost TCP (`127.0.0.1:9091`)
- CLI control tool connects to this socket for local operations
- Protocol: Simple JSON-RPC or HTTP REST over the local socket
- TUI polls this socket for real-time status updates
- CLI does NOT connect directly to the Main App — it goes through the daemon

### 1.4 Overseer / Agent Server Architecture — Important

**Current state:** Underspecified. "Part of the CLI" — unclear if separate process, thread, or mode.

**Recommendation:**
- Make the overseer a **component of the host-server daemon**, not the CLI. The daemon is always running; the CLI may not be
- TUI connects to the daemon to display overseer state
- Deadlock detection strategies:
  1. **Output staleness:** No stdout/stderr for N seconds (configurable, default 120s)
  2. **Tool call loops:** Detecting repeated identical tool calls (hash last 5 tool invocations)
  3. **Token budget exhaustion:** Approaching max_turns without meaningful output
  4. **Wall clock timeout:** Hard deadline regardless of activity
- Nudge actions: inject "hey, are you stuck?" system message → if no response after 2 nudges, SIGTERM → report as `stalled`

### 1.5 Validation Loop — Important

**Current state:** Vague — "validation artifacts and checklists will be attached."

**Recommendation:**
- Each work item carries a `validation_spec`:
  ```json
  {
    "type": "checklist",
    "items": [
      {"id": "1", "description": "cargo build succeeds", "check": "exit_code == 0", "command": "cargo build 2>&1"},
      {"id": "2", "description": "tests pass", "check": "exit_code == 0", "command": "cargo test 2>&1"},
      {"id": "3", "description": "no clippy warnings", "check": "exit_code == 0", "command": "cargo clippy -- -D warnings"}
    ]
  }
  ```
- Overseer runs validation steps after harness completes
- If validation fails: retry (max N times), report failure upstream, or escalate
- Validation itself has a timeout — if a step hangs, it's killed and reported

### 1.6 Vulcanum API Skill (NEW — Agent-Native Control Surface) — Important

**Gap:** No way for users to control Vulcanum through their own agents. Every operation requires the CLI or TUI.

**Approach: A `vulcanum-api` SKILL.md bundled with the CLI.**

Instead of building a separate "Vulcanum Agent" service, ship a SKILL.md that maps 1:1 to Vulcanum's API surface. Users drop this skill into their agent of choice (Hermes, Claude Code, Codex CLI) and get full programmatic control over the orchestrator.

**Why a skill instead of a separate agent:**
- Zero new infrastructure — leverages the agent the user already has
- SKILL.md is versioned alongside the CLI and API (same release, no drift)
- Portable — works with Hermes, Claude Code, any agent that loads SKILL.md files
- The skill is the API documentation, the tool definitions, and the usage guide in one file
- 1:1 mapping to API endpoints means the skill stays in sync automatically — adding an API endpoint means adding a section to the skill

**What the skill covers (mapped to API):**

| API Endpoint | Skill Tool / Section | Description |
|---|---|---|
| `GET /workers` | List workers | All registered workers, status, capabilities, last seen |
| `GET /workers/:id` | Get worker details | Specific worker health, current work, metrics |
| `POST /workers/register` | Register worker | Bootstrap a new worker machine |
| `DELETE /workers/:id` | Remove worker | Decommission a worker |
| `GET /runs` | List work runs | Recent work runs with filters (status, worker, harness) |
| `GET /runs/:id` | Get run details | Full run info: logs, artifacts, token usage, validation results |
| `POST /runs/:id/retry` | Retry work run | Re-queue a failed run |
| `POST /runs/:id/cancel` | Cancel work run | Terminate a running work item |
| `GET /tasks/poll` | Poll for ready tasks | (Internal — used by workers, but exposed for agent visibility) |
| `POST /tasks/dispatch` | Dispatch work | Manually dispatch a task to a worker |
| `POST /tasks/create` | Create work item | Create a new task in the connected task manager (Kaneo/Linear) |
| `GET /secrets` | List secret refs (names + providers, never values) | What secrets are configured and where they live |
| `POST /secrets` | Add/rotate secret | Add or rotate an API key or config template |
| `GET /metrics` | System metrics | Queue depth, dispatch rate, success rate, token spend |
| `GET /health` | Health check | Orchestrator + connected task managers status |

**SKILL.md structure:**

```
skills/vulcanum-api/
├── SKILL.md          # Main skill: overview, auth setup, tool definitions
├── references/
│   ├── api.md        # Full API reference (auto-generated from OpenAPI spec)
│   └── examples.md   # Common workflows (create work, debug failure, add worker)
└── templates/
    └── work-item.json  # Work item template
```

**Concrete usage examples (what users say to their agent):**

- "Create a vulcanum task to fix all clippy warnings in the vulcanum repo — use claude-sonnet-4, max 25 turns, validate with cargo build + cargo test"
- "Show me the last 5 failed work runs and what went wrong"
- "Worker-3 has been disconnected for 2 hours — what's its last known state and what was it working on?"
- "Rotate the Anthropic API key — the old one was leaked"
- "How many tokens did we burn this week across all workers?"

**CLI integration:**

```bash
# Install the skill for an agent
vulcanum install-skill --agent hermes
vulcanum install-skill --agent claude-code
vulcanum install-skill --agent codex

# This copies SKILL.md + references to the agent's skills directory
# and prompts the user to configure the API endpoint + credentials
```

**Delivery:** The SKILL.md ships inside the CLI binary (embedded via `include_str!`) and is extracted on `vulcanum install-skill`. It's also available in the vulcanum repo at `skills/vulcanum-api/SKILL.md` for manual installation.

### 1.7 Multi-Tenant Isolation — Important

**Current state:** Not addressed. Users register and manage servers.

**Recommendation:**
- Every worker is owned by a user (foreign key `workers.user_id → users.id`)
- Work items are scoped to a user. Workers only receive work from their owner
- Server-side rate limiting per user: max concurrent work items, max workers, API rate limits
- Queue fairness: round-robin or weighted fair queuing across users
- Row-level security in PostgreSQL as defense-in-depth

### 1.8 Artifact & Result Storage — Important

**Recommendation:**
- **Local:** Worker keeps results in `~/.vulcanum/results/{work_id}/` until submitted + acked by server. After ack, local copy deleted
- **Server:** Results stored in PostgreSQL (metadata) + object storage or filesystem (artifacts). TTL: 30 days default, configurable
- **Local metadata:** `work_id`, timestamps, harness used, tokens consumed, cost estimate, exit code, validation results
- Results synced back to task manager as comments with summary + link to artifacts

### 1.9 Bootstrap & Installation Flow — Moderate

**Recommendation:**
- Single binary with subcommands:
  - `vulcanum daemon` — starts the host-server daemon
  - `vulcanum tui` — opens the TUI control panel
  - `vulcanum register --token <registration-token>` — registers this machine, generates local credentials
  - `vulcanum install-skill --agent <hermes|claude-code|codex>` — installs the vulcanum-api SKILL.md for agent-native control
  - `vulcanum status` — quick health check
- Installation: `curl | sh` one-liner. The binary contains all modes
- `vulcanum register` installs a systemd user service so the daemon survives reboots

### 1.10 Observability — Moderate

**Recommendation:**
- Use `tracing` crate instead of `log`
- Export to OpenTelemetry (OTLP) for metrics, traces, logs
- Key metrics: work queue depth, dispatch latency, harness success rate, token consumption per user/worker
- Structured log format (JSON) in production

### 1.11 Daemon Updates — Moderate

**Recommendation:**
- Self-update mechanism: daemon polls for new versions, downloads to staging path
- On SIGUSR1 or graceful shutdown, replaces itself (execve)
- Systemd `ExecStartPre` can verify binary checksum before starting
- Server advertises minimum supported worker version; outdated workers receive `upgrade_required`

### 1.12 Secret Management — Reference Model (Not Storage)

**Previous approach:** Age-encrypted values stored in PostgreSQL `secrets` table. Server holds master key, decrypts at dispatch.

**Revised approach:** Vulcanum never stores secrets. It stores **references** to an external secret manager. This eliminates ciphertext custody, key rotation liability, and audit trail responsibility — all solved problems with mature tools.

**Architecture:**
```
Vulcanum Main App                    HashiCorp Vault / Infisical / AWS SM
     │                                        │
     │  GET secret/vulcanum/anthropic_api_key  │
     │───────────────────────────────────────►│
     │                                        │
     │  sk-ant-... (plaintext, over mTLS)     │
     │◄───────────────────────────────────────│
     │                                        │
     │  Wrap with one-time age key + TTL       │
     │  (plaintext never persisted)            │
     │  Send wrapped secret to worker          │
```

**`secret_refs` table (replaces `secrets`):**
```sql
CREATE TABLE secret_refs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,              -- "ANTHROPIC_API_KEY", "GITHUB_TOKEN"
    provider TEXT NOT NULL,          -- "vault" | "infisical" | "env"
    external_path TEXT NOT NULL,     -- "secret/vulcanum/anthropic_api_key"
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Supported providers:**

| Provider | Deployment | Best for |
|---|---|---|---|
| **HashiCorp Vault** | Self-hosted (single binary, file backend) | Primary — open source MPL, REST API, audit logging, dynamic secrets |
| **Infisical** | Self-hosted (Docker Compose) or Cloud | Lighter alternative — MIT, simpler than Vault |
| **env** | Reads from Main App env vars | Dev only — zero setup, not for production |

**Vault single-node deployment (VPS-scale, zero external deps):**
```bash
vault server -config=/etc/vault/config.hcl

# config.hcl — file backend, localhost only
storage "file" { path = "/var/lib/vault/data" }
listener "tcp" { address = "127.0.0.1:8200"; tls_disable = true }
api_addr = "http://127.0.0.1:8200"
disable_mlock = true  # VPS without mlock privileges
```
No Consul, no Raft, no HCP. Single binary, ~50MB RAM, file-backed. For a single-tenant orchestrator, this is perfectly adequate — you're not running a bank.

Dropped AWS Secrets Manager and GCP Secret Manager — cloud-vendor lock-in makes no sense for a self-hosted orchestrator.

**At dispatch time:**
1. Work item specifies `secrets: ["ANTHROPIC_API_KEY", "GITHUB_TOKEN"]`
2. Main App looks up `secret_refs` → resolves provider + path
3. Fetches plaintext from provider (cached for duration of dispatch, never persisted)
4. Wraps with one-time age key + 5min TTL
5. Sends wrapped secret to worker via the polling response
6. Worker decrypts → injects via memfd → destroys after harness exit

**What we no longer own:**
- Encryption at rest → provider handles it
- Key rotation → provider handles it
- Audit logging → provider handles it
- Access policies → provider handles it
- Dynamic/rotating secrets → provider handles it (Vault)

**What we still own (and should):**
- Per-dispatch wrapping (one-time key + TTL) — defense-in-depth for the transport leg
- memfd injection on worker — sandbox-level isolation
- Config file generation in tmpfs — harness-level isolation
- Output sanitization — safety net

**Fallback for dev/single-user:** The `env` provider reads from environment variables on the Main App server. Not recommended for production, but zero setup for local development.

### 1.13 CLI Mode Structure — Low

**Recommendation:**
- Do NOT create a separate crate. Use subcommands within the `cli` crate:
  ```
  vulcanum tui        → launches TUI
  vulcanum daemon     → starts host-server daemon
  vulcanum register   → registers this machine
  ```
- The `host-server` crate becomes a **library** that both the daemon and the CLI import. The CLI crate is the binary entry point

---

## 2. Suggested Architecture Improvements

### 2.1 Unify host-server and cli Binaries

**Current:** Two separate binaries.

**Proposed:** Single binary (`vulcanum`) with subcommands. `host-server` crate becomes `vulcanum-daemon` library. CLI crate becomes binary entry point with TUI + control tool + daemon mode.

### 2.2 Task Manager Abstraction Layer

Vulcanum needs a pluggable task manager interface so adding new integrations is trivial:

```rust
trait TaskManager {
    /// Poll for tasks ready to be worked on
    async fn poll_ready(&self, filter: &TaskFilter) -> Result<Vec<ExternalTask>>;

    /// Update task status (e.g., move to "In Progress")
    async fn update_status(&self, task_id: &str, status: &str) -> Result<()>;

    /// Post a comment/result to the task
    async fn post_comment(&self, task_id: &str, comment: &str) -> Result<()>;

    /// Register a webhook for push notifications (if supported)
    async fn register_webhook(&self, url: &str, events: &[&str]) -> Result<()>;
}
```

Initial implementations: `KaneoTaskManager`, `LinearTaskManager`.

### 2.3 Work Specification Format

Work items are derived from external task manager tasks, enriched with Vulcanum-specific config:

```json
{
  "external_task_ref": "kaneo:task_abc123",
  "harness": "claude_code",
  "model": "claude-sonnet-4-20250514",
  "prompt": "Fix all clippy warnings in this Rust project",
  "workdir_ref": "git@github.com:user/repo.git#branch",
  "max_turns": 25,
  "timeout_secs": 600,
  "isolation_tier": "default",
  "allow_network": false,
  "validation": { ... },
  "secrets": ["ANTHROPIC_API_KEY", "GITHUB_TOKEN"],
  "artifacts": ["diff.patch", "files/"]
}
```

The `external_task_ref` is the canonical link back to the task manager. Vulcanum stores work *runs*, not work *items*.

### 2.4 State Machine for Work Runs

```
                    ┌──────────┐
                    │  PENDING  │  (task polled from task manager, not yet dispatched)
                    └─────┬─────┘
                          │ dispatch
                    ┌─────▼──────┐
               ┌────│ DISPATCHED │
               │    └─────┬──────┘
               │          │ worker ack (10s timeout)
               │    ┌─────▼─────────┐
               │    │ ACKNOWLEDGED  │
               │    └─────┬─────────┘
               │          │ harness started
               │    ┌─────▼─────┐
               │    │  RUNNING  │
               │    └──┬───┬───┘
               │       │   │
          timeout    done  │ stalled/crashed
               │       │   │
               ▼       ▼   ▼
          ┌────────┐ ┌──────────┐ ┌────────┐
          │PENDING │ │VALIDATING│ │ FAILED │
          │(requeue│ └────┬─────┘ └────────┘
          │ )      │      │
          └────────┘ ┌────▼─────┐
                     │ COMPLETED │
                     └──────────┘
                          │
                          ▼
                     Task manager updated
                     (status + comment with results)
```

### 2.5 Connection Architecture (Polling Primary)

```
Worker Daemon                    Vulcanum Main App
     │                                │
     │  GET /poll?worker_id=X         │
     │  (every 15s, configurable)     │
     │───────────────────────────────►│
     │                                │
     │  200 OK { work: {...} }        │
     │  or 204 No Content             │
     │◄───────────────────────────────│
     │                                │
     │  POST /ack { work_id, status } │
     │───────────────────────────────►│
     │                                │
     │  POST /progress { work_id, ..} │
     │───────────────────────────────►│
     │                                │
     │  POST /result { work_id, ..}   │
     │───────────────────────────────►│
     │                                │
     │  200 OK { ack: true }          │
     │◄───────────────────────────────│
```

No persistent connections. Stateless HTTP. Horizontally scalable behind any load balancer.

---

## 3. Implementation Priority

| Priority | Item | Rationale |
|----------|------|-----------|
| **P0** | Worker auth model | Workers can't function without it |
| **P0** | HTTP polling endpoint (worker ↔ server) | Core communication channel |
| **P0** | Task manager integration (Kaneo) | Where work comes from; without this, nothing to dispatch |
| **P1** | Basic harness spawning (Claude Code) | MVP: just run a harness and get output |
| **P1** | Sandboxing (bubblewrap tier) | Security baseline before any real use |
| **P1** | Result submission + task manager sync | Close the loop |
| **P2** | Secret injection (memfd + tmpfs) | Required before handling real API keys |
| **P2** | CLI control tool + daemon IPC | User-facing bootstrap flow |
| **P2** | Vulcanum API SKILL.md + `install-skill` CLI command | Agent-native control surface; drop-in for Hermes/Claude Code/Codex |
| **P2** | State machine + work run tracking | Operational visibility |
| **P3** | Overseer + validation loop | Quality of life, not blocking |
| **P3** | TUI | Nice to have, daemon works headless |
| **P3** | Multi-tenant isolation | Important but can be retrofitted |
| **P3** | Linear integration (webhook support) | After Kaneo is stable |
| **P4** | Self-update mechanism | Can update manually during MVP |
| **P4** | Enhanced isolation (Podman rootless) | Bubblewrap covers 90% of use cases |
| **P4** | Frontend UI | Control via CLI + API skill first; skill covers 90% of UI use cases |
| **P4** | SSE optional upgrade | Only if polling latency is a bottleneck |

---

## 4. Component Architecture (Revised)

```
┌─────────────────────────────────────────────────────────────────┐
│                     VULCANUM MAIN APP                             │
│                                                                   │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐ │
│  │ Task Manager    │  │ Worker Registry   │  │ Work Runs +       │ │
│  │ Bridge          │  │ (PostgreSQL)      │  │ Secret Refs       │ │
│  │ (Kaneo, Linear) │  │ - workers         │  │ (PostgreSQL)      │ │
│  │ - poll/webhook  │  │ - capabilities    │  │ - run history     │ │
│  │ - status sync   │  │ - liveness        │  │ - artifact refs   │ │
│  │                 │  │                   │  │ - token usage     │ │
│  │                 │  │                   │  │ - secret refs →   │ │
│  │                 │  │                   │  │   Vault/Infisical │ │
│  └────────┬────────┘  └────────┬─────────┘  └────────┬────────┘ │
│           │                    │                      │          │
│  ┌────────▼────────────────────▼──────────────────────▼────────┐ │
│  │                      API Layer (axum)                        │ │
│  │  GET  /poll           → work dispatch                        │ │
│  │  POST /ack            → work acknowledgment                  │ │
│  │  POST /progress       → progress updates                     │ │
│  │  POST /result         → result submission                    │ │
│  │  POST /register       → worker registration                  │ │
│  │  GET  /events         → SSE (optional, push upgrade)          │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  Vulcanum API (full control surface)                           │ │
│  │  - All endpoints exposed for agent-native control              │ │
│  │  - 1:1 mapped in skills/vulcanum-api/SKILL.md                  │ │
│  │  - Users drop skill into Hermes/Claude Code/Codex for control  │ │
│  │  - No separate agent service — skill IS the interface          │ │
│  └──────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     VULCANUM WORKER DAEMON                        │
│  (Rust binary, runs as user service)                             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Poll Loop                                                │   │
│  │  - GET /poll every N seconds (configurable, default 15s)  │   │
│  │  - No persistent connection state                          │   │
│  │  - Backoff only on server unreachable                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Work Executor                                            │   │
│  │  - Decrypts wrapped secrets (age) into memfd              │   │
│  │  - Generates config files in tmpfs (read-only mount)      │   │
│  │  - Prepares sandbox (bwrap/nsjail or podman rootless)     │   │
│  │  - Creates ephemeral tmpfs workdir                        │   │
│  │  - Spawns harness (Claude Code / Codex / OpenCode)        │   │
│  │  - Enforces CPU/mem/time limits via cgroups v2            │   │
│  │  - Network egress filtering (api.anthropic.com only)      │   │
│  │  - Collects artifacts, diff, sanitizes output             │   │
│  │  - Destroys sandbox, tmpfs, memfd on completion           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Harness Registry                                         │   │
│  │  - Claude Code adapter (primary)                          │   │
│  │  - Codex CLI adapter                                      │   │
│  │  - Generic (shell command) adapter                        │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```
