# Vulcanum Architecture — Gap Analysis & Recommendations

**Date:** 2026-05-15
**Status:** Initial analysis based on architecture discussion

---

## 1. Gaps Identified

### 1.1 Work Queue & Persistence Model (Critical)

**Current state:** Not defined.

**Problem:** What happens when the server restarts mid-dispatch? When the worker crashes mid-task? Without explicit at-least-once delivery semantics, work items will be silently lost.

**Recommendation:**
- Implement a persistent work queue in PostgreSQL with status states: `pending → dispatched → acknowledged → running → completed/failed`
- Dispatch sets `status=dispatched, dispatched_at=NOW()`. If no ack within 10s, reset to `pending`
- Worker sends periodic heartbeats. If heartbeat stops for 90s, consider the worker dead and re-queue its in-flight work
- Results are submitted idempotently (by `work_id`). Server deduplicates
- This also enables the "committed" requirement — results are stored server-side as the source of truth

### 1.2 Worker Authentication Model (Critical)

**Current state:** Magic-link auth exists for human users, but worker daemons are not human users.

**Problem:** How does a headless daemon authenticate? It can't click magic links.

**Recommendation:**
- Workers authenticate via pre-provisioned API keys or mTLS certificates
- Bootstrap flow: User creates a "worker registration token" in the Main App UI → CLI control tool uses this token to register the machine → server issues a persistent worker credential (API key or client certificate)
- Worker stores credential locally (`~/.vulcanum/credentials`) with restrictive file permissions (0600)
- All subsequent connections use this credential. Token rotation supported via server-initiated re-key

### 1.3 CLI ↔ Host Server IPC (Critical)

**Current state:** CLI and host-server are separate crates, both placeholders. No IPC defined.

**Problem:** The CLI control tool needs to communicate with the local host-server daemon for operations like "register this machine", "check worker status", "view logs". This IPC channel isn't designed.

**Recommendation:**
- Host server listens on a Unix domain socket (`/run/vulcanum/daemon.sock`) or localhost TCP (`127.0.0.1:9091`)
- CLI control tool connects to this socket for local operations
- Protocol: Simple JSON-RPC or HTTP REST over the local socket
- TUI polls this socket for real-time status updates
- The TUI does NOT connect directly to the Main App — it goes through the daemon, keeping the daemon as the single source of truth on the worker machine

### 1.4 Overseer / Agent Server Architecture (Important)

**Current state:** Described as "part of the CLI" — an agent server that monitors worker harness, nudges, and controls.

**Problem:** The overseer is underspecified. Key questions:
- Is it a separate process, a thread in the host-server, or a mode of the CLI?
- What does "nudging" mean concretely — retry? different prompt? escalate to human?
- How does it detect deadlocks vs. slow legitimate work?

**Recommendation:**
- Make the overseer a **component of the host-server daemon**, not the CLI. The daemon is always running; the CLI may not be
- The TUI connects to the daemon to display overseer state visually
- Deadlock detection strategies:
  1. **Output staleness:** No stdout/stderr for N seconds (configurable, default 120s)
  2. **Tool call loops:** Detecting repeated identical tool calls (hash last 5 tool invocations)
  3. **Token budget exhaustion:** Approaching max_turns without meaningful output
  4. **Wall clock timeout:** Hard deadline regardless of activity
- Nudge actions: inject a "hey, are you stuck?" system message → if no response after 2 nudges, SIGTERM → report as `stalled`
- Validation: after harness completes, run validation steps (see §1.5)

### 1.5 Validation Loop (Important)

**Current state:** Mentioned — "validation artifacts and checklists will be attached to the work", "might involve manual tests via playwright or computer use."

**Problem:** The validation model is vague. What validates what? How does the system know if work is "done and running"?

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
- The overseer runs validation steps after the harness completes
- If validation fails, the overseer can: retry (max N times), report failure upstream, or escalate
- For manual/visual validation (playwright, computer use), the spec includes a screenshot/diff check
- Deadlock avoidance: validation itself has a timeout. If a validation step hangs, it's killed and reported

### 1.6 Multi-Tenant Isolation (Important)

**Current state:** Not addressed. Users register and manage servers.

**Problem:** If User A and User B both use Vulcanum, what prevents User A's worker from claiming User B's work? What prevents queue starvation?

**Recommendation:**
- Every worker is owned by a user (foreign key `workers.user_id → users.id`)
- Work items are scoped to a user. Workers only receive work from their owner
- Server-side rate limiting per user: max concurrent work items, max workers, API rate limits
- Queue fairness: round-robin or weighted fair queuing across users
- Row-level security in PostgreSQL as defense-in-depth

### 1.7 Artifact & Result Storage (Important)

**Current state:** "Metadata had to be saved locally and submitted upstream with usage details."

**Problem:** What's the lifecycle of results? How long are they kept? Where are they stored?

**Recommendation:**
- **Local:** Worker keeps results in `~/.vulcanum/results/{work_id}/` until successfully submitted + acked by server. After ack, local copy is deleted
- **Server:** Results stored in PostgreSQL (metadata) + object storage or filesystem (artifacts: diffs, logs, files). TTL: 30 days default, configurable per user
- **Local metadata:** `work_id`, timestamps, harness used, tokens consumed, cost estimate, exit code, validation results
- **Usage details:** Token counts, wall time, CPU time, peak memory — submitted as structured JSON alongside the result for billing/analytics

### 1.8 Bootstrap & Installation Flow (Moderate)

**Current state:** CLI has a "control tool" mode that "connects the current machine as a worker."

**Problem:** This implies the CLI installs/configures the host-server daemon. But how? Is the daemon a separate binary? Does the CLI install it?

**Recommendation:**
- Single binary with subcommands:
  - `vulcanum daemon` — starts the host-server daemon
  - `vulcanum tui` — opens the TUI control panel
  - `vulcanum register --token <registration-token>` — registers this machine, generates local credentials, sets up systemd service
  - `vulcanum status` — quick health check
- Installation: `curl | sh` one-liner or package manager. The binary contains all modes
- `vulcanum register` installs a systemd user service (`~/.config/systemd/user/vulcanum-daemon.service`) so the daemon survives reboots

### 1.9 Observability (Moderate)

**Current state:** `log` and `pretty_env_logger` used in main-app. No metrics, no tracing.

**Problem:** Debugging distributed agent workflows across server + multiple workers is hard without structured observability.

**Recommendation:**
- Use `tracing` crate (already in the project conventions) instead of `log`
- Export to OpenTelemetry (OTLP) for metrics, traces, logs
- Key metrics: work queue depth, dispatch latency, harness success rate, token consumption per user/worker
- Structured log format (JSON) in production
- Worker submits telemetry as part of heartbeat: CPU load, memory usage, disk space, harness version

### 1.10 Daemon Updates (Moderate)

**Current state:** Not addressed.

**Problem:** How does a running daemon get updated? Can't overwrite the binary while it's running.

**Recommendation:**
- Self-update mechanism: daemon polls for new versions, downloads new binary to a staging path
- On SIGUSR1 or graceful shutdown, replaces itself (execve or similar)
- Systemd `ExecStartPre` can verify binary checksum before starting
- Version compatibility: server advertises minimum supported worker version; outdated workers receive `upgrade_required` message and refuse work

### 1.11 Secret Store on Server Side (Moderate)

**Current state:** Not designed. Research recommends age-encrypted at rest.

**Problem:** Where does the Main App store user API keys (Claude, OpenAI, etc.) that need to be injected into workers?

**Recommendation:**
- Secrets stored in `secrets` table: `id, user_id, name, encrypted_value, created_at, rotated_at`
- Encrypted at rest with age (server's master key)
- Decrypted only at dispatch time — wrapped per-work-item with a one-time wrapping key
- Never returned in plaintext via REST — only via the encrypted WebSocket channel
- Audit log: every secret access is logged (who, when, for which work item)

### 1.12 The "Two Ways" CLI Mode Confusion (Low)

**Current state:** "CLI can be used in 2 ways — TUI control panel and control tool."

**Problem:** "This is why we probably need another Rust crate." The user is unsure how to structure this.

**Recommendation:**
- Do NOT create a separate crate. Use subcommands within the `cli` crate:
  ```
  vulcanum tui        → launches TUI
  vulcanum daemon     → starts host-server (replaces host-server crate? or shares lib?)
  vulcanum register   → registers this machine
  ```
- The `host-server` crate becomes a **library** that both the daemon and the CLI import. The CLI crate is the binary entry point
- This avoids crate proliferation while keeping concerns separated at the module level

---

## 2. Suggested Architecture Improvements

### 2.1 Unify host-server and cli Binaries

**Current:** Two separate binaries (cli placeholder, host-server placeholder).

**Proposed:** Single binary (`vulcanum`) with subcommands. The `host-server` crate becomes `vulcanum-daemon` library. The `cli` crate becomes the binary with TUI + control tool + daemon mode.

**Rationale:** One install step. No version skew between CLI and daemon. Shared code (auth, config, connection management) lives in the daemon library.

### 2.2 Add a "Gateway" Concept

**Current:** Workers connect directly to Main App.

**Proposed:** Consider a lightweight gateway/relay for environments where direct WSS isn't possible:
- Gateway runs as a separate process (or sidecar), relays between worker and server
- Useful for: corporate proxies that block WebSocket, air-gapped networks with an HTTP proxy
- Could be as simple as SSE + HTTP POST relay

### 2.3 Work Specification Format

**Current:** Not defined — what does a "work item" look like?

**Proposed schema:**
```json
{
  "work_id": "uuid",
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

### 2.4 State Machine for Work Items

```
                    ┌──────────┐
                    │  PENDING  │
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
```

---

## 3. Implementation Priority

| Priority | Item | Rationale |
|----------|------|-----------|
| **P0** | Work queue + state machine | Foundation for everything |
| **P0** | Worker auth model | Workers can't function without it |
| **P1** | WebSocket connection (worker ↔ server) | Core communication channel |
| **P1** | Basic harness spawning (Claude Code) | MVP: just run a harness and get output |
| **P1** | Sandboxing (bubblewrap tier) | Security baseline before any real use |
| **P2** | Secret injection (age + memfd) | Required before handling real API keys |
| **P2** | CLI control tool + daemon IPC | User-facing bootstrap flow |
| **P2** | Result submission + artifact storage | Close the work loop |
| **P3** | Overseer + validation loop | Quality of life, not blocking |
| **P3** | TUI | Nice to have, daemon works headless |
| **P3** | Multi-tenant isolation | Important but can be retrofitted |
| **P4** | Self-update mechanism | Can update manually during MVP |
| **P4** | Frontend UI | Control via CLI first |
