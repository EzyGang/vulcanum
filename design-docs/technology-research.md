# Vulcanum Agent Orchestrator — Technology Research Report

**Date:** 2026-05-15
**Status:** Research complete (network unavailable — based on training knowledge; verify URLs/versions when online)
**Scope:** Agent isolation, secret injection, polling mechanisms, agent harnesses

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
| **Network isolation** | Bridge/NAT, can disable | Bridge/NAT, can disable | Configurable, can disable | gVisor netstack, can disable | Network namespace, can disable |
| **Filesystem isolation** | Copy-on-write overlay | Copy-on-write overlay | Block device | Rootfs overlay | Mount namespace + tmpfs |
| **Requires daemon** | Yes (dockerd) | No (fork-exec) | No (firecracker binary) | Yes (runsc, can be per-invocation) | No (setuid bwrap binary) |
| **Root required** | Yes (root daemon) | No (rootless) | No (but needs /dev/kvm) | No | bwrap: setuid or unpriv user namespaces |
| **macOS support** | Docker Desktop (Linux VM) | Podman Machine (Linux VM) | ❌ (Linux KVM only) | Docker Desktop (runsc) | ❌ (Linux-only) |
| **Windows support** | Docker Desktop/WSL2 | Podman on WSL2 | ❌ | Docker Desktop | ❌ (WSL2 via Linux) |

### 1.2 Detailed Analysis

#### 1.2.1 Docker (Moby)
- **Pros:** Massive ecosystem, well-known, easy OCI image management, Docker Compose for multi-service, rich tooling, works cross-platform via Docker Desktop.
- **Cons:** Requires root daemon (dockerd), daemon is a single point of failure, images can be large, cold start slow, container escape surface is real (shared kernel). Docker-in-Docker is messy.
- **Verdict:** Solid baseline. Rootless mode exists but is less battle-tested. Better for dev/staging than untrusted multi-tenant workloads.

#### 1.2.2 Podman (rootless by default)
- **Pros:** Daemonless architecture — fork/exec model, no long-running daemon required. True rootless containers using user namespaces (UID mapping). OCI-compatible, can pull/push same images as Docker. Pod support (Kubernetes-style). systemd integration for lifecycle. `podman play kube` for Kubernetes YAML. Much safer default posture.
- **Cons:** macOS requires a Linux VM (podman machine). Networking in rootless mode can be tricky (slirp4netns/pasta). Slightly less ecosystem maturity than Docker but catching up fast.
- **Verdict:** **Strong recommendation for Vulcanum.** Rootless by default, no daemon lifecycle headaches, good security baseline. On macOS, falls back to a lightweight Linux VM.

#### 1.2.3 Firecracker (AWS microVM)
- **Pros:** Per-workload microVM with its own minimal Linux kernel. Cold start in ~125ms. KVM-based hardware virtualization — strongest isolation short of a full VM. Memory overhead ~5MB per microVM. Used by AWS Lambda and Fargate at enormous scale. No legacy device emulation. `jailer` binary for additional sandboxing of the VMM process itself.
- **Cons:** Linux-only (requires KVM). Each microVM needs its own kernel and rootfs image. More complex setup than containers. No OCI image support natively (you build rootfs images separately). No native cross-platform support. Requires the worker machine to have KVM available.
- **Verdict:** **Best isolation available** but at what cost? For Vulcanum running untrusted third-party agent code, this is the gold standard. However, setup complexity and lack of cross-platform support mean it should be a tiered option: Firecracker for high-security Linux workers, container-based for others.

#### 1.2.4 gVisor (application kernel / sentry)
- **Pros:** Intercepts syscalls in user space (sentry process) rather than passing to host kernel. Much smaller attack surface (~20% of Linux syscalls implemented). Runs as an OCI runtime (`runsc`) — drop-in replacement for runc. Integration with Docker and Podman. Used in Google Cloud Run and GKE Sandbox.
- **Cons:** Not all syscalls implemented; some workloads break. ~5–15% performance overhead (improving rapidly). Higher memory overhead than namespaces (sentry process per container). Still ultimately a user-space process (not hardware isolation).
- **Verdict:** **Strong middle ground.** Better isolation than plain containers, less complexity than Firecracker. Drop-in OCI compatibility is huge. If Vulcanum workers need to run arbitrary code that might try syscall-based exploits, gVisor is a pragmatic upgrade.

#### 1.2.5 bubblewrap (bwrap) + nsjail
- **Pros:** Extremely lightweight — bubblewrap creates a new mount/user/PID/IPC/net namespace in milliseconds with near-zero memory overhead. No image management, no daemon. Used by Flatpak (bubblewrap). Perfect for "just give me an isolated process with specific file/dir access and no network." nsjail (Google) adds seccomp-bpf filtering, cgroup resource limits, and a "clone" mode for process-level isolation.
- **Cons:** Not a full container — no image layering, no networking setup, no OCI ecosystem. Linux-only. Requires user namespace support (`kernel.unprivileged_userns_clone=1`). You build the rootfs yourself (can be as simple as bind-mounting `/usr` read-only + tmpfs `/tmp`).
- **Verdict:** **Best for ultra-lightweight isolation when you control what's inside.** If the agent harness is a single binary (e.g., a Rust-compiled tool or a single Python script with all deps vendored), bubblewrap provides excellent isolation with milliseconds of startup overhead. nsjail adds cgroup resource limits. This is the approach used by many CI/CD systems and sandboxed code execution services.

### 1.3 Recommended Isolation Strategy (Tiered)

```
Tier 1 (Default, Linux): bubblewrap + nsjail
  - Startup: 10-50ms
  - Overhead: ~2-5MB
  - Use when: running trusted harness binaries with known dependencies
  - Rootfs: bind-mount host /usr + /lib read-only, tmpfs /tmp, /home as ephemeral workdir

Tier 2 (Enhanced, Linux): Podman rootless + gVisor (runsc)
  - Startup: 0.5-2s
  - Overhead: ~30-80MB
  - Use when: running untrusted code, need full OCI image with complex deps
  - Runtime: runsc (gVisor) for syscall filtering

Tier 3 (Maximum, Linux): Firecracker microVM
  - Startup: 125-300ms
  - Overhead: ~10-20MB
  - Use when: running completely untrusted third-party agents, maximum isolation required
  - Rootfs: pre-built minimal Linux + harness binary

Tier macOS: Podman machine (Linux VM) + containerd/runsc
Tier Windows: WSL2 + Podman or Docker
```

### 1.4 Ephemeral Filesystems & Cleanup

**Approach: tmpfs + mount namespace destruction**

After each work item completes:
1. The harness's mount namespace is destroyed (all mounts evaporate)
2. Work directory should ALWAYS be a tmpfs mount created specifically for that work item
3. Secrets directory should be a separate tmpfs (never on disk)
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
  --dev /dev \
  --unshare-all \
  --die-with-parent \
  --seccomp 10 \
  --new-session \
  /path/to/harness
```

When bwrap exits, ALL tmpfs mounts are automatically cleaned up by the kernel. No cleanup daemon needed.

### 1.5 Resource Limits

All approaches support Linux cgroups v2 for resource enforcement:

| Resource | Mechanism | Notes |
|---|---|---|
| CPU | `cpu.max` (cgroup v2) | CPU bandwidth limit (e.g., "200000 100000" = 2 cores) |
| Memory | `memory.max` + `memory.high` | Hard limit + soft limit for throttling before OOM |
| Disk I/O | `io.max` | Read/write bandwidth + IOPS limits |
| PIDs | `pids.max` | Prevent fork bombs |
| Time | `timeout` or cgroup freezer | SIGKILL after N seconds |
| Network | Network namespace (no netns = no network) | Simplest: `--unshare-net` in bwrap |

Implementation options:
- **nsjail:** Native cgroup support (`--cgroup_mem_max`, `--cgroup_cpu_ms_per_sec`, `--time_limit`)
- **bubblewrap:** Set cgroup manually before/after spawn (bwrap doesn't manage cgroups natively — but you can use `systemd-run --scope` or write to cgroupfs directly)
- **systemd-run:** `systemd-run --user --scope -p MemoryMax=512M -p CPUQuota=200% -p RuntimeMaxSec=300 ./harness`

**Recommendation:** Use `systemd-run --user --scope` when available (it handles cgroup lifecycle automatically). Fall back to direct cgroupfs manipulation or nsjail on non-systemd systems.

---

## 2. SECURE CREDENTIAL / SECRET INJECTION

### 2.1 Threat Model

For Vulcanum, secrets (API keys, tokens, environment variables) must:
1. Be securely delivered from the central server to the worker daemon
2. Be injected into the isolated harness process
3. **Never** touch disk unencrypted
4. Be destroyed after work completion
5. Not leak into logs, artifacts, or agent output
6. Survive a compromised harness that actively tries to exfiltrate them

### 2.2 Delivery: Server → Worker Daemon

| Approach | Security | Complexity | Best For |
|---|---|---|---|
| **mTLS channel** | High | Medium | Encrypted transport for all comms |
| **HashiCorp Vault** | Very High | High | Dynamic secrets, audited access, token wrapping |
| **SOPS (Mozilla)** | Medium-High | Low | Encrypt secrets at rest in git/config |
| **Sealed Secrets (Bitnami)** | Medium-High | Medium | Kubernetes-native encrypted secrets |
| **NACL/age encryption** | High | Low | Simple key-pair encryption, wrap per-message |
| **JWT with claims** | Medium | Low | Stateless short-lived tokens |

**Recommended approach for Vulcanum:**

1. **Worker ↔ Server channel:** mTLS with certificate pinning. The worker generates a keypair on first run, the admin approves the CSR (or uses pre-shared tokens for initial bootstrap). All subsequent communication is encrypted.

2. **Secret wrapping (one-time-use tokens):** Modeled on HashiCorp Vault's wrapping pattern:
   - Server encrypts the secret with a one-time key
   - Server sends the wrapped secret + a "wrapping token" to the worker
   - Worker unwraps it in-memory, injects into the harness
   - Wrapping token is single-use; any replay is detectable
   - After TTL expires, the wrapping token is invalid

3. **Simple age-based encryption** as a lightweight alternative to Vault:
   ```
   Server: echo "$SECRET" | age -r $WORKER_PUBKEY > wrapped_secret.age
   Worker:  age -d -i $WORKER_PRIVKEY wrapped_secret.age | inject_into_harness
   ```
   The worker's private key never leaves memory. `age` (https://github.com/FiloSottile/age) is a simple, modern encryption tool — ~2000 lines of Go, auditable, no complex PKI.

### 2.3 Injection: Worker → Harness

**In-memory only approaches (ranked):**

#### Option A: Environment variables + memfd (Recommended)
```rust
// Pseudocode for Rust worker daemon
let secret_value = decrypt_wrapped_secret(&wrapped);

// Create an in-memory file descriptor (never touches disk)
let memfd = memfd::MemfdOptions::default()
    .allow_sealing(true)
    .create("secret_env")?;
memfd.as_file().write_all(secret_value.as_bytes())?;
memfd.add_seals(&[FileSeal::SealWrite, FileSeal::SealGrow, FileSeal::SealShrink])?;

// Pass the fd to the harness process
let fd_num = memfd.as_raw_fd();
// Spawn with env: SECRETS_FD=3 (the inherited fd)
Command::new("bwrap")
    .env("SECRETS_FD", fd_num.to_string())
    .prepend_fd(memfd.as_file()) // keep fd 3 alive for child
    .spawn()?;
```
The harness reads from the inherited file descriptor. FDs can be sealed (immutable). When the process exits, the memfd is freed. This never touches disk.

#### Option B: tmpfs mount (still in-memory, visible as path)
```bash
mount -t tmpfs -o size=1M,uid=1000 tmpfs /run/vulcanum/secrets/$WORK_ID
echo "$SECRET" > /run/vulcanum/secrets/$WORK_ID/token
# Pass path to harness
bwrap --bind /run/vulcanum/secrets/$WORK_ID /secrets ... ./harness
# After harness exits:
umount /run/vulcanum/secrets/$WORK_ID
```
tmpfs is RAM-backed, not disk-backed. But it's visible as a filesystem path, and if the harness can escape its mount namespace, it could read other secrets. Less ideal than memfd but simpler for tools that expect file paths.

#### Option C: stdin pipe (simplest, most secure)
```bash
echo "$SECRET" | bwrap ... ./harness --read-secret-from-stdin
```
Single secret, no filesystem involvement at all. But only works for one secret and requires the harness to support stdin reading.

**Recommended:** Option A (memfd) for multiple secrets, with Option C (stdin) as a fallback for single-secret workloads.

### 2.4 Preventing Secret Leakage

| Vector | Mitigation |
|---|---|
| **Agent logs** | Run agent with log level that suppresses secrets; grep/filter output for known secret patterns before forwarding to server |
| **Agent artifacts** | Scan artifact files for secret patterns; never include /secrets in artifact collection paths |
| **Environment extraction** | Use memfd, not env vars — `/proc/$PID/environ` is world-readable for same UID; memfd is only accessible to the process |
| **Core dumps** | Set `ulimit -c 0` or `PR_SET_DUMPABLE(0)` before spawning |
| **ptrace debugging** | Set `prctl(PR_SET_PTRACER, PR_SET_PTRACER_ANY)` to disable, or use seccomp filter to block ptrace |
| **Network exfiltration** | Run harness with `--unshare-net` (no network). If network is needed, use a restricted netns with egress filtering |

### 2.5 Secret Lifecycle

```
┌──────────┐     mTLS      ┌──────────┐    memfd    ┌──────────┐
│  Server  │ ──────────────→│  Worker  │───────────→│ Harness  │
│          │  wrapped       │  Daemon  │  unwrapped │ (bwrap)  │
│ encrypts │  secret +      │ decrypts │  via fd    │ reads    │
│ w/ age   │  TTL           │ in RAM   │            │ from fd  │
└──────────┘                └──────────┘            └──────────┘
                                  │                      │
                                  ▼                      ▼
                            Secret destroyed      Process exits,
                            from worker memory    memfd freed,
                            after injection       namespace destroyed
```

### 2.6 Specific Tool Recommendations

| Tool | Use Case | Notes |
|---|---|---|
| **[age](https://github.com/FiloSottile/age)** | File/secret encryption | Simple, modern, auditable. Better than GPG for operational use. |
| **[memfd](https://man7.org/linux/man-pages/man2/memfd_create.2.html)** (Linux) | In-memory secret storage | Kernel feature since 3.17. File sealing since 4.3. |
| **[HashiCorp Vault](https://www.vaultproject.io/)** | Enterprise secret management | Dynamic secrets, wrapping, audit logging. Heavy but comprehensive. |
| **[SOPS](https://github.com/getsops/sops)** | Git-friendly encrypted config | Encrypts values in YAML/JSON/ENV files. Works with age, GPG, AWS KMS, GCP KMS, Azure Key Vault. |
| **[librwap](https://github.com/hashicorp/go-kms-wrapping)** | Token wrapping primitives | Go library; port concepts to Rust. |

---

## 3. LONG/SHORT POLLING MECHANISMS

### 3.1 Protocol Comparison

| Protocol | Direction | NAT-Friendly | Connection Overhead | Rust Ecosystem | Real-Time | Best For |
|---|---|---|---|---|---|---|
| **WebSocket** | Bidirectional | ✅ (client initiates) | Low (persistent TCP) | `tokio-tungstenite`, `axum` | ✅ | Low-latency command streaming |
| **Long Polling** | Client→Server | ✅ | Medium (re-establish) | Built-in HTTP | ~✅ (near real-time) | Simple, firewall-friendly |
| **SSE (Server-Sent Events)** | Server→Client | ✅ (client initiates HTTP) | Low (persistent HTTP) | `reqwest` + streaming, `axum` SSE | ✅ | One-way event streaming |
| **gRPC Streaming** | Bidirectional | ✅ (client-side stream) | Low (HTTP/2) | `tonic` | ✅ | High-perf typed contracts |
| **Polling (short)** | Client→Server | ✅ | High (frequent requests) | Built-in HTTP | ❌ (~delay) | Fallback only |

### 3.2 NAT/Firewall Considerations

All five approaches work behind NAT/firewalls because **the worker initiates the connection** to the server. This is a hard requirement for Vulcanum — workers are on user machines, often behind NAT, with no inbound port forwarding.

**Key consideration:** The connection must be outbound-only. The server never tries to connect to the worker.

### 3.3 Detailed Analysis

#### 3.3.1 WebSocket (Recommended Primary)

**Pros:**
- True bidirectional communication — server can push work immediately
- Single persistent TCP connection; no per-message overhead
- Well-understood, battle-tested, works everywhere
- Upgrade from HTTP(S) — passes through most proxies
- `tokio-tungstenite` is fast (Rust to Rust ~1M messages/sec on localhost)
- Supports Ping/Pong frames for keepalive (application-level, not TCP keepalive)

**Cons:**
- Connection state management (reconnection, backoff)
- Some proxies don't support WebSocket (rare in 2025, but possible)
- No built-in request/response correlation — you implement your own message IDs

**Implementation pattern for Rust:**
```rust
// Using tokio-tungstenite + futures
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

let (ws_stream, _) = connect_async("wss://server/vulcanum/ws").await?;
let (mut write, mut read) = ws_stream.split();

// Heartbeat: send ping every 30s
tokio::spawn(async move { loop { write.send(Ping).await; sleep(Duration::from_secs(30)).await; } });

// Read work messages
while let Some(msg) = read.next().await {
    match msg {
        Message::Text(text) => handle_work(text).await,
        Message::Pong(_) => continue,
        Message::Close(_) => break,
    }
}
```

#### 3.3.2 Server-Sent Events (SSE) + HTTP POST (Good Alternative)

**Pros:**
- SSE for server→worker (work dispatch, config updates)
- HTTP POST for worker→server (acknowledgments, progress, results)
- SSE uses standard HTTP, works through every proxy
- Built-in auto-reconnect with `Last-Event-ID` header
- Simpler than WebSocket — unidirectional data flow matches work dispatch naturally

**Cons:**
- Two connections (one SSE, one for POST responses). But POSTs are transient.
- Cannot push data from worker to server without an active POST connection
- HTTP/1.1 limitation: max ~6 concurrent SSE connections per domain in browsers (not relevant for native client)
- Binary data requires base64 encoding in SSE

**Implementation pattern:**
```
Worker opens SSE:  GET /vulcanum/events?worker_id=X
Server pushes:     event: work\ndata: {"work_id":"abc","task":"..."}\n\n
Worker acknowledges: POST /vulcanum/acknowledge {"work_id":"abc","status":"accepted"}
Worker reports progress: POST /vulcanum/progress {"work_id":"abc","percent":50}
Worker submits result:   POST /vulcanum/result {"work_id":"abc","artifacts":[...]}
Server pushes next work: event: work\n...
```

#### 3.3.3 gRPC Streaming (Best for Type Safety)

**Pros:**
- Strongly typed contracts (Protobuf)
- Bidirectional streaming with `rpc Connect(stream WorkerMessage) returns (stream ServerMessage)`
- HTTP/2 multiplexing — efficient connection usage
- Excellent Rust support via `tonic`
- Built-in deadline propagation, cancellation, flow control
- TLS/mTLS built-in

**Cons:**
- Heavier setup — Protobuf compilation, generated code
- Less debuggable (binary protocol)
- Some HTTP/2 implementations struggle with certain proxies
- Overkill for simple "get work, do work, return result" patterns

**Implementation pattern (tonic):**
```protobuf
service Vulcanum {
  rpc Connect(stream WorkerMessage) returns (stream ServerMessage);
}
message ServerMessage {
  oneof payload {
    WorkAssignment work = 1;
    HeartbeatAck heartbeat_ack = 2;
  }
}
message WorkerMessage {
  oneof payload {
    Heartbeat heartbeat = 1;
    WorkAck ack = 2;
    ProgressUpdate progress = 3;
    WorkResult result = 4;
  }
}
```

### 3.4 Reconnection & Backoff Strategy

**Recommended algorithm: Exponential backoff with jitter + capped delay**

Inspired by AWS SDK retry strategy and Google's gRPC backoff:

```
attempt 0: 0s (immediate)
attempt 1: 1s + rand(0, 1s)   → ~1-2s
attempt 2: 2s + rand(0, 2s)   → ~2-4s
attempt 3: 4s + rand(0, 4s)   → ~4-8s
attempt 4: 8s + rand(0, 8s)   → ~8-16s
...
cap at 60s, then retry every 60s + rand(0, 30s)
max attempts: infinite (workers should never give up)
```

After 5 consecutive failures, the worker should:
1. Log a warning
2. Start sending periodic "I'm alive but disconnected" telemetry via a separate mechanism if possible
3. Continue retrying on the primary channel

**Pseudo-code:**
```rust
fn connect_with_backoff() -> Result<Connection> {
    let mut attempt = 0;
    loop {
        match try_connect() {
            Ok(conn) => return Ok(conn),
            Err(e) if attempt < 10 => {
                let delay = (2u64.pow(attempt.min(6)) * 1000)
                    + thread_rng().gen_range(0..1000);
                attempt += 1;
                log::warn!("Connection failed (attempt {attempt}): {e}. Retrying in {delay}ms");
                sleep(Duration::from_millis(delay));
            }
            Err(e) => {
                log::error!("Connection failed after {attempt} attempts: {e}");
                sleep(Duration::from_secs(60));
                attempt = 0; // reset and try again
            }
        }
    }
}
```

### 3.5 Heartbeats & Work Acknowledgment

**Pattern:** The worker maintains a heartbeat loop independent of work processing.

```
Worker heartbeat (every 30s):
  → PING frame (WebSocket) or heartbeat message (gRPC/SSE)
  → Server responds with PONG or heartbeat_ack

Server-side timeout:
  → If no heartbeat for 90s, mark worker as disconnected
  → If worker was processing work, re-queue the work item

Work acknowledgment (3-phase commit lite):
  1. Server sends:  {"type":"work","work_id":"abc","...","ttl":300}
  2. Worker responds: {"type":"ack","work_id":"abc"}         // within 10s
  3. Worker sends:   {"type":"progress","work_id":"abc",...}  // periodic
  4. Worker sends:   {"type":"result","work_id":"abc","status":"success",...}
  5. Server acknowledges result: {"type":"result_ack","work_id":"abc"}

If step 2 doesn't arrive within 10s:
  → Server considers work unacknowledged, re-queues

If step 5 doesn't arrive:
  → Worker retries result submission (idempotent by work_id)
```

### 3.6 Progress Reporting

Progress should be structured and lightweight:

```json
{
  "type": "progress",
  "work_id": "abc123",
  "timestamp": "2026-05-15T23:51:00Z",
  "status": "running",
  "phase": "executing_agent",
  "message": "Claude Code analyzing repository structure...",
  "percent": 45.0,
  "metrics": {
    "tokens_used": 12500,
    "files_examined": 34,
    "tool_calls": 12,
    "wall_time_ms": 45000
  }
}
```

### 3.7 Recommendation

**Primary: WebSocket** — Best balance of simplicity, performance, and bidirectional capability. Use `tokio-tungstenite` with `native-tls` or `rustls` for TLS.

**Fallback: SSE + HTTP POST** — When WebSocket isn't available (corporate proxy blocking upgrades). Simple to implement, works everywhere.

**Future: gRPC streaming** — If Vulcanum's API surface grows complex enough to benefit from strong typing. The Protobuf contract becomes the source of truth for the protocol.

---

## 4. AGENT HARNESSES

### 4.1 Harness Survey

#### 4.1.1 Claude Code (Anthropic)

**What it is:** Anthropic's official CLI agent for software engineering tasks. Built on the Claude API with tool use capabilities.

**Key interfaces:**
- **CLI:** `claude` command (global npm install or one-shot via `npx @anthropic-ai/claude-code`)
- **Modes:** Interactive (default TUI), `-p/--print` for non-interactive (prints response and exits), `--output-format json` for structured output
- **Prompt passing:** `claude -p "Your prompt here"` or pipe via stdin: `echo "prompt" | claude`
- **Workdir:** `claude --cwd /path/to/project -p "task"`
- **Permission model:** `--permission-mode` (acceptEdits, bypassPermissions, default, plan)
- **Output capture:** stdout for text, stderr for diagnostics, exit code 0 (success) / non-zero (error/refusal)
- **Artifacts:** Files modified/created in the workdir; diff available via `--print` mode
- **Configuration:** `.claude.json` or `.claude/settings.json` for project-level config; `CLAUDE_API_KEY` env var or `--api-key` flag

**Headless/non-interactive:** ✅ Supported via `--print` (`-p`). Example:
```bash
CLAUDE_API_KEY=sk-ant-... claude -p "Fix the lint errors in src/main.rs" --cwd /work --permission-mode acceptEdits
```

**Output structure (JSON mode):**
```json
{
  "result": "...",
  "usage": {"input_tokens": 5000, "output_tokens": 1500},
  "model": "claude-sonnet-4-20250514",
  "stop_reason": "end_turn",
  "tool_use": [...]
}
```

**Spawn programmatically (Rust):**
```rust
use std::process::Command;

let output = Command::new("claude")
    .arg("-p")
    .arg("Fix all compiler warnings in this Rust project")
    .arg("--cwd")
    .arg("/work/repo")
    .arg("--output-format")
    .arg("json")
    .arg("--permission-mode")
    .arg("acceptEdits")
    .env("CLAUDE_API_KEY", api_key) // or use memfd approach
    .output()?;
```

**Timeout enforcement:** Claude Code respects SIGTERM. Use `Command::spawn()` + `child.wait_timeout(Duration::from_secs(N))`.

**Key considerations for Vulcanum:**
- API key must be provided per invocation (good — one-time secret injection)
- `--permission-mode acceptEdits` disables all interactive prompts
- `--max-turns` to limit tool call iterations
- Rate limits: See Anthropic docs for tier-specific limits
- Anthropic's MCP (Model Context Protocol) servers can be configured if needed

#### 4.1.2 Codex CLI (OpenAI)

**What it is:** OpenAI's open-source CLI agent (Rust-based). Runs in terminal, uses OpenAI models, has tool-using capabilities.

**Key interfaces:**
- **CLI:** `codex` binary (Rust, compiled). Install via `cargo install codex-cli` or prebuilt binary.
- **Modes:** Interactive TUI (default), `exec` subcommand for non-interactive single-turn
- **Prompt passing:** `codex exec "Your prompt"` for non-interactive mode
- **Workdir:** Respects current working directory or `--cwd`
- **Configuration:** `~/.config/codex/config.toml` or env vars. `OPENAI_API_KEY` required.
- **Session management:** `codex session` to manage persistent sessions
- **Output:** stdout, structured output with `--json`

**Headless/non-interactive:** ✅ Supported via `codex exec`:
```bash
OPENAI_API_KEY=sk-... codex exec "Write a function that sorts a list" --json
```

**Key considerations:**
- Open source (Apache 2.0) — can be bundled with Vulcanum
- Written in Rust — good synergy with a Rust worker daemon
- OpenAI API dependency — needs API key, cost tracking
- `--model` flag to specify gpt-4o, gpt-4.1, etc.
- Codex's tool use is less extensive than Claude Code's, but rapidly evolving
- Session resumption: `codex session resume <id>` — useful for long-running tasks

#### 4.1.3 OpenCode / opencode

**What it is:** Terminal-based AI coding agent (Rust + TypeScript ecosystem). Focused on code editing workflows.

**Key interfaces:**
- **CLI:** `opencode` command
- **Modes:** Interactive TUI, pipe-based non-interactive
- **Prompt passing:** `opencode "prompt"` or `echo "prompt" | opencode`
- **Workdir:** Current directory
- **Configuration:** `.opencode.json`, `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`
- **Output:** Writes to files, stdout for progress

**Key considerations:**
- Less mature than Claude Code or Codex CLI
- Multi-provider support (OpenAI + Anthropic)
- Good for programmatic code modification tasks

#### 4.1.4 ForgeCode

**What it is:** A newer agent harness focused on "forge" workflows — building, testing, and iterating on code. Part of the broader agent ecosystem.

**Key interfaces:**
- **CLI:** `forge` command
- **Modes:** Interactive and `--non-interactive` flag
- **Prompt passing:** `forge run "task description"`
- **Workdir:** `--workspace` flag
- **Output:** Structured JSON output with `--output json`

**Key considerations:**
- Less widely adopted; verify current state
- May not be as reliable as Claude Code or Codex CLI for production orchestration

#### 4.1.5 General Agent Harness Patterns

Beyond specific tools, Vulcanum should support a **generic harness interface**:

```rust
trait AgentHarness {
    fn name(&self) -> &str;
    fn spawn_command(&self, config: &WorkConfig) -> Command;
    fn parse_output(&self, output: &ProcessOutput) -> Result<WorkResult>;
    fn required_env_vars(&self) -> Vec<SecretSpec>;
    fn timeout(&self) -> Duration;
}
```

This allows plugging in any harness that:
1. Accepts a prompt
2. Runs in a working directory
3. Produces output (stdout/files/exit code)
4. Respects environment variables for secrets

### 4.2 Harness Spawning & Management

#### 4.2.1 Generic Spawn Pattern (Rust)

```rust
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::process::Command as AsyncCommand; // for async I/O
use tokio::time::timeout;

async fn run_harness(
    harness: &dyn AgentHarness,
    work: &WorkConfig,
    secrets: &HashMap<String, Secret>,
    sandbox: &Sandbox,
) -> Result<WorkResult> {
    let mut cmd = harness.spawn_command(work);

    // Apply sandboxing
    sandbox.wrap_command(&mut cmd);

    // Inject secrets (via memfd or env)
    for (name, secret) in secrets {
        cmd.env(name, secret.value()); // or use memfd approach
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null()); // no interactive input

    let mut child = cmd.spawn()?;

    // Stream output in real-time for progress reporting
    let stdout_reader = BufReader::new(child.stdout.take().unwrap());
    let stderr_reader = BufReader::new(child.stderr.take().unwrap());

    // Enforce timeout
    let result = timeout(harness.timeout(), async {
        let status = child.wait().await?;
        // Read remaining output...
        Ok::<_, Error>(status)
    }).await;

    match result {
        Ok(Ok(status)) => {
            if status.success() {
                harness.parse_output(&collected_output)
            } else {
                Err(WorkError::HarnessFailed(status.code()))
            }
        }
        Ok(Err(e)) => Err(WorkError::ProcessError(e)),
        Err(_elapsed) => {
            child.kill().await.ok();
            Err(WorkError::Timeout)
        }
    }
}
```

#### 4.2.2 Output & Artifact Capture

**Strategy:**
1. **stdout:** Parse for structured output (JSON lines, exit code signals)
2. **stderr:** Capture for diagnostics, scan for secrets before forwarding
3. **Workdir files:** After harness exits, diff the workdir against a pre-run snapshot. Collect new/modified files as artifacts.
4. **Exit code:** 0 = success, 1-127 = error, 128+N = killed by signal N

**Artifact naming convention:**
```
/artifacts/{work_id}/
├── stdout.log
├── stderr.log
├── exit_code
├── metrics.json          # tokens used, time taken, etc.
├── files/                # modified/created files (relative paths)
│   ├── src/main.rs
│   └── Cargo.toml
└── diff.patch            # unified diff of all changes
```

#### 4.2.3 Multi-Harness Dispatch

Vulcanum should support a registry pattern:

```toml
# Worker config: /etc/vulcanum/harnesses.toml
[harnesses.claude_code]
command = "claude"
version_check = "claude --version"
install_hint = "npm install -g @anthropic-ai/claude-code"
default_model = "claude-sonnet-4-20250514"
supports = ["code_review", "refactoring", "debugging", "general"]

[harnesses.codex]
command = "codex"
version_check = "codex --version"
install_hint = "cargo install codex-cli"
default_model = "gpt-4.1"
supports = ["code_generation", "general"]

[harnesses.opencode]
command = "opencode"
version_check = "opencode --version"
supports = ["code_editing"]
```

The server can specify `harness: "claude_code"` or let the worker auto-select based on capability matching.

### 4.3 Existing Orchestration Patterns

| Project | Pattern | Relevance |
|---|---|---|
| **SWE-bench** | Docker-based harness spawning, evaluation | Gold standard for benchmarking agent coding tasks |
| **OpenHands (formerly OpenDevin)** | Docker sandbox, agent loop, event stream | Similar architecture to Vulcanum |
| **Aider** | Direct CLI agent with git integration | Single-agent focus, good patterns for edit tracking |
| **Devin** (Cognition) | Proprietary agent orchestration | Market leader in autonomous coding agents |
| **CodeScene** | Agent-based code analysis pipelines | Pattern for multi-step agent workflows |
| **Sourcegraph Cody** | Agent in IDE + CLI | Patterns for context gathering before agent invocation |

### 4.4 Recommendation

**Primary harnesses to support (in order):**
1. **Claude Code** — Most capable, best docs, excellent headless mode, JSON output, strong permission model
2. **Codex CLI** — Open source, Rust-based (good synergy), `exec` mode works well
3. **OpenCode** — Multi-model, simpler setup

**For initial Vulcanum MVP:**
- Start with Claude Code as the sole harness
- Build the generic `AgentHarness` trait so adding more harnesses is trivial
- Test against repositories in /work directories with various task types

**Key design decisions:**
- Always use `--permission-mode acceptEdits` (or equivalent) with `--max-turns` for safety
- Always set a hard timeout (wall clock) — 5 minutes default, configurable
- Always run in a sandbox with no network (unless the task explicitly requires it)
- Never allow interactive mode — all harnesses must be spawned in non-interactive/headless mode

---

## 5. PUTTING IT ALL TOGETHER: Reference Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        VULCANUM SERVER                           │
│  - PostgreSQL (work queue, worker registry, audit log)           │
│  - mTLS endpoint for worker connections                          │
│  - Secret store (age-encrypted at rest, unwrapped per-dispatch)  │
│  - WebSocket server (tokio-tungstenite + axum)                   │
└────────────────────────┬────────────────────────────────────────┘
                         │  WSS (mTLS)
                         │  WebSocket bidirectional
                         │
┌────────────────────────▼────────────────────────────────────────┐
│                     VULCANUM WORKER DAEMON                        │
│  (Rust binary, runs as user service)                             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Connection Manager                                       │   │
│  │  - WebSocket client with exponential backoff              │   │
│  │  - Heartbeat (30s), work ack (10s)                        │   │
│  │  - Progress/status streaming                              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Work Executor                                            │   │
│  │  - Receives work from Connection Manager                  │   │
│  │  - Decrypts wrapped secrets (age) into memfd              │   │
│  │  - Prepares sandbox (bwrap/nsjail or podman+runsc)        │   │
│  │  - Creates ephemeral tmpfs workdir                        │   │
│  │  - Spawns harness (Claude Code / Codex / OpenCode)        │   │
│  │  - Enforces CPU/mem/time limits via cgroups v2            │   │
│  │  - Streams output back to server                          │   │
│  │  - Collects artifacts, diff                               │   │
│  │  - Destroys sandbox, tmpfs, memfd on completion           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Harness Registry                                         │   │
│  │  - Claude Code adapter                                    │   │
│  │  - Codex CLI adapter                                      │   │
│  │  - Generic (shell command) adapter                        │   │
│  │  - Auto-detection (which harnesses are installed?)        │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. CONCRETE RECOMMENDATIONS SUMMARY

### Isolation (pick one per tier)
| Tier | Technology | When to use |
|---|---|---|
| **Default** | bubblewrap + tmpfs + cgroups v2 | Everyday work from trusted sources |
| **Enhanced** | Podman rootless + gVisor (runsc) | Running code from less-trusted sources |
| **Maximum** | Firecracker microVM | Multi-tenant, completely untrusted code |

### Secrets
| Component | Recommendation |
|---|---|
| **Encryption** | age (X25519) for wrapping secrets |
| **Transport** | mTLS WebSocket |
| **Injection** | memfd (Linux 3.17+) with file sealing |
| **Cleanup** | Auto-destroyed on process exit (memfd + namespace destruction) |

### Polling
| Component | Recommendation |
|---|---|
| **Protocol** | WebSocket (primary), SSE+POST (fallback) |
| **Libraries** | `tokio-tungstenite` + `axum` (server), `tokio-tungstenite` (client) |
| **Backoff** | Exponential with jitter, 1s–60s cap |
| **Heartbeat** | 30s interval, 90s timeout before marking disconnected |

### Harnesses
| Priority | Harness | Status |
|---|---|---|
| **P0** | Claude Code (`claude -p`) | Production-ready, best in class |
| **P1** | Codex CLI (`codex exec`) | Open source, Rust, good |
| **P2** | OpenCode | Multi-model, simpler |
| **P3** | Generic shell adapter | Fallback for any CLI tool |

### Cross-Platform Strategy
| Platform | Isolation | Notes |
|---|---|---|
| **Linux** | bubblewrap/nsjail or Podman+gVisor or Firecracker | Full isolation spectrum |
| **macOS** | Podman machine (Linux VM) or macOS sandbox-exec | Limited; `sandbox-exec` is underdocumented |
| **Windows** | WSL2 + Podman/Docker | Windows-native sandboxing is weak |

**Recommendation:** Target Linux as the primary worker platform. Treat macOS/Windows as "best effort" with Podman-based isolation through their respective Linux VM layers.

---

## 7. OPEN QUESTIONS / FURTHER RESEARCH

- [ ] **Firecracker rootfs packaging:** What's the minimal rootfs needed to run Claude Code? Can we build a <50MB rootfs?
- [ ] **Age key rotation:** How to handle worker key rotation without breaking in-flight work?
- [ ] **WebSocket message framing:** Max message size? Fragment large artifacts?
- [ ] **Claude Code max-turns testing:** What's a safe default for different task types?
- [ ] **macOS native sandboxing:** `sandbox-exec` profiles vs. just using Podman machine — worth the effort?
- [ ] **Windows Defender interaction:** Does it block bwrap/namespace operations in WSL2?
- [ ] **Cost tracking:** Per-work API token usage reporting to the server
- [ ] **Artifact size limits:** What's the max artifact bundle to transfer over WebSocket?
