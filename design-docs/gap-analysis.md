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

### 1.6 Vulcanum Agent (NEW — Meta-Agent for System Operations) — Important

**Gap:** No mention of an agent that helps *operate* Vulcanum itself. The system orchestrates agents but has no agent to manage the system.

**What this is:**
- A "Vulcanum Agent" (Hermes-managed, or a Claude-based harness) that has access to Vulcanum's internal APIs
- It can: create work items in Kaneo/Linear on behalf of users, check worker status, debug failed runs, triage errors, suggest worker configuration
- Users interact through existing channels (Discord, Telegram, CLI): "hey vulcanum, why did work item X fail?"
- This is distinct from worker harnesses that execute user code — this is a management/operations agent

**Concrete capabilities:**
1. **Work creation:** "Create a task in Kaneo to fix clippy warnings in the vulcanum repo" → agent creates properly formatted task with validation spec
2. **Status inquiry:** "What's worker-3 doing right now?" → agent queries internal state
3. **Failure triage:** "Why did work item abc123 fail?" → agent reads run logs, checks validation output, suggests cause
4. **Worker management:** "Register a new worker on this machine" → agent guides through bootstrap flow
5. **System health:** "Are any workers disconnected?" → agent queries worker liveness

**Implementation approach:**
- Expose Vulcanum's internal APIs (work run history, worker status, task manager bridge) as tools/skills the agent can call
- Could be a Hermes skill initially, graduating to a dedicated agent interface
- The agent needs read access to: work run records, worker registry, task manager status
- The agent needs write access to: create/update tasks in task manager, trigger re-queues, rotate worker credentials

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

### 1.12 Secret Store on Server Side — Moderate

Secrets (API keys, config templates) stored in `secrets` table:
- `id, user_id, name, encrypted_value, created_at, rotated_at`
- Encrypted at rest with age (server's master key)
- Decrypted only at dispatch time — wrapped per-work-item with a one-time wrapping key
- Never returned in plaintext via REST — only via the encrypted channel to the worker
- See technology-research.md §2 for the full exposure minimization strategy

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
| **P2** | Vulcanum Agent (basic: work creation, status inquiry) | System usability; self-serve operations |
| **P2** | State machine + work run tracking | Operational visibility |
| **P3** | Overseer + validation loop | Quality of life, not blocking |
| **P3** | TUI | Nice to have, daemon works headless |
| **P3** | Multi-tenant isolation | Important but can be retrofitted |
| **P3** | Linear integration (webhook support) | After Kaneo is stable |
| **P4** | Self-update mechanism | Can update manually during MVP |
| **P4** | Enhanced isolation (Podman rootless) | Bubblewrap covers 90% of use cases |
| **P4** | Frontend UI | Control via CLI + Vulcanum Agent first |
| **P4** | SSE optional upgrade | Only if polling latency is a bottleneck |

---

## 4. Component Architecture (Revised)

```
┌─────────────────────────────────────────────────────────────────┐
│                     VULCANUM MAIN APP                             │
│                                                                   │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐ │
│  │ Task Manager    │  │ Worker Registry   │  │ Work Run Store  │ │
│  │ Bridge          │  │ (PostgreSQL)      │  │ (PostgreSQL)    │ │
│  │ (Kaneo, Linear) │  │ - workers         │  │ - run history   │ │
│  │ - poll/webook   │  │ - capabilities    │  │ - artifacts     │ │
│  │ - status sync   │  │ - liveness        │  │ - token usage   │ │
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
│  │  Vulcanum Agent API (internal)                                 │ │
│  │  - Query work runs, worker status                             │ │
│  │  - Create tasks in task manager                               │ │
│  │  - System health checks                                       │ │
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
