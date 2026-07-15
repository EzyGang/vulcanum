# Vulcanum

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/EzyGang/vulcanum/ci.yml?style=for-the-badge" alt="CI">
  <img src="https://img.shields.io/github/v/release/EzyGang/vulcanum?style=for-the-badge" alt="GitHub release">
  <img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later">
</p>

---

**Vulcanum is a self-hosted orchestration layer for agentic software work.**

Its goal is to sit between task-tracker backends, implementation-agent backends, and higher-level harnesses such as OpenClaw, Hermes, or custom automation. Vulcanum owns the work lifecycle—pickup, dispatch, execution, review, and status synchronization—without requiring the rest of a stack to be built around one tracker, model provider, or coding agent.

The architecture is designed to be AI-, agent-, tracker-, and harness-agnostic. The current implementation is narrower than that goal: it ships a Kaneo task-tracker adapter, GitHub App repository access, and OpenCode and OMP RPC agent backends. Other adapters remain future work.

> [!WARNING]
> Vulcanum is active pre-1.0 infrastructure software. Interfaces and migrations can change. Operate it on infrastructure you control, start with repositories and credentials you can safely expose to the deployment, and treat the host execution mode as trusted-development-only.

## Table of Contents

- [Why Vulcanum?](#why-vulcanum)
- [Current Capabilities](#current-capabilities)
- [Architecture](#architecture)
- [How It Works](#how-it-works)
- [Getting Started](#getting-started)
- [Security and Isolation](#security-and-isolation)
- [Integrations](#integrations)
- [Releases](#releases)
- [Roadmap](#roadmap)
- [Repository Layout](#repository-layout)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## Why Vulcanum?

Agentic work often starts as a script that reads a ticket and invokes a coding agent. That works until the system needs durable state, multiple workers, constrained repository access, recovery after crashes, review automation, or a second tracker or agent backend.

Vulcanum centralizes those concerns behind one control plane:

- Humans continue working in the task tracker they already use.
- Higher-level harnesses can treat Vulcanum as the execution and lifecycle boundary rather than managing workers directly.
- Workers advertise their agent and isolation capabilities, poll for compatible work, and execute several jobs up to their configured capacity.
- Agent backends are selected through a shared runtime contract instead of being embedded in task-tracker logic.
- Run state, events, usage, pull requests, and review results return to the control plane and the source task.

Vulcanum is not a model, coding agent, editor, task tracker, or general-purpose CI system. It coordinates those systems.

## Current Capabilities

| Area | What exists today |
| --- | --- |
| Task-driven execution | Poll enabled Kaneo projects, create implementation runs from a configured pickup column, move tasks through progress/review/done columns, and synchronize comments and lifecycle labels. |
| Work-run lifecycle | Persist pending, dispatched, running, completed, failed, and stalled runs in PostgreSQL. Support cancellation, explicit failure, result submission, and stale-run recovery. |
| Dispatch | Match pending work to workers with compatible agent-backend capabilities and available concurrency, using PostgreSQL for durable state and Redis for dispatch/cancellation signals. |
| Workers | Register workers with short-lived access and refresh tokens, execute concurrent jobs, journal local state in SQLite, and recover or retire interrupted sessions after restart. |
| Agent execution | Run OpenCode or OMP through a common runtime interface with multi-turn continuation and an explicit `finish_run` result contract. |
| Isolation | Prepare per-run workspaces on the host, in Docker, or in Docker with the Kata runtime. Host mode is not a security boundary. |
| Repository access | Use a GitHub App to obtain repository-scoped installation tokens, clone one or more configured repositories, and associate returned GitHub pull requests with the source task. |
| Review automation | Optionally create a separate pull-request review run for each implementation PR, record the review comment, and leave the task in review until all linked PRs are closed or merged. |
| Observability | Store ordered run events plus exit status, duration, model, token counts, result summary, worker, and PR URLs. The UI exposes dashboards, task boards, workers, runs, teams, and settings. |
| Model configuration | Load the model catalog from [models.dev](https://models.dev/), store model-provider credentials encrypted at rest, and render backend-specific model configuration for jobs. |
| Auth | Support single-user instance-password mode and multiuser GitHub OAuth with teams and expiring invite links. |

### Current Boundaries

These limitations are intentional to state plainly:

- Kaneo is the only task-tracker adapter currently implemented.
- GitHub is the only repository/VCS integration currently implemented.
- OpenCode and OMP RPC are the only agent backends currently implemented.
- There is no dedicated OpenClaw or Hermes adapter yet. The stable automation contract and higher-level harness adapters are roadmap work; today, external automation can drive the existing tracker-facing workflow.
- Vulcanum does not currently publish the conceptual per-run artifact directory previously described here. Durable server-side data consists of run records, PR associations, usage fields, and events. Workers also save exported session messages locally under `~/.vulcanum/sessions/` when the backend provides them.
- CPU and memory policy are not yet a complete configurable enforcement layer. Do not treat the current runtime defaults as a hardened multi-tenant sandbox policy.
- Model-provider credentials are encrypted in PostgreSQL, but they are decrypted by the server and delivered to the selected worker for execution. A worker-side credential broker/vault is planned.

---

## Architecture

```mermaid
flowchart LR
  tracker[Task-tracker backend] <--> tracker_adapter[Task-tracker adapter]
  tracker_adapter <--> control[Vulcanum control plane\nHTTP API + poller + lifecycle services]

  ui[Preact control UI] --> control
  higher[Higher-level harnesses\nOpenClaw / Hermes / custom] -. planned stable automation API .-> control

  control <--> postgres[(PostgreSQL\ndurable state)]
  control <--> redis[(Redis\ndispatch, cancellation, webhook queues)]
  dispatcher[Dispatcher process] <--> postgres
  dispatcher <--> redis

  workers[Worker pool] -->|poll, ack, events, result| control
  workers <--> journal[(Per-worker SQLite journal)]
  workers --> isolation[Isolation provider\nHost / Docker / Kata]
  isolation --> runtime[Agent runtime adapter]
  runtime --> opencode[OpenCode]
  runtime --> omp[OMP RPC]

  control <--> github[GitHub App\nrepos, tokens, PR webhooks]
  github --> repos[Git repositories]
  isolation --> repos
```

The dashed higher-level-harness edge describes the intended public boundary, not a claim that dedicated OpenClaw or Hermes integrations already ship.

### Control Plane

The `vulcanum-web` binary provides the Actix Web API, runs PostgreSQL migrations at startup, polls enabled task-tracker projects, processes GitHub webhooks, and serves the services used by the frontend and workers. Business logic follows the HTTP → service → repository layering in `server/`.

The separate `vulcanum-dispatcher` binary finds pending runs and assigns them to compatible workers with available capacity. PostgreSQL remains the source of durable state; Redis carries dispatch, cancellation, registration-code, OAuth-device-flow, and webhook-work signals.

### Worker Plane

The worker daemon polls the control plane, acknowledges assigned jobs, prepares repository workspaces, launches the selected agent backend, forwards events, and submits the final result. Its SQLite journal records in-flight execution state so startup recovery can reconnect to supported live sessions or mark lost work consistently.

Workers advertise both agent-backend and isolation capabilities. A worker configured for OMP RPC will not be selected for a run requiring OpenCode, and worker capacity is enforced independently of the worker's idle/busy display state.

### Integration Boundaries

Task trackers implement the server-side task-provider interfaces. Agent backends implement the worker-side runtime/session interfaces. Isolation providers prepare the environment independently of the agent runtime. This separation is what allows the project to add adapters without coupling tracker semantics to an agent process.

GitHub is currently more tightly integrated because repository installation tokens, pull-request association, review runs, and close/merge webhooks all depend on the GitHub App flow. Generalizing that boundary to other VCS providers remains planned work.

---

## How It Works

```text
Configured pickup column
        |
        v
Server poller creates a pending implementation run
        |
        v
Dispatcher reserves capacity on a compatible worker
        |
        v
Worker polls -> fetches job -> acknowledges -> task moves to progress
        |
        v
Worker clones configured repositories with a GitHub App token
        |
        v
Selected backend (OpenCode or OMP RPC) runs in host/Docker/Kata environment
        |
        +--> ordered events and usage are reported during execution
        |
        v
Agent calls finish_run -> worker submits summary, usage, status, and PR URLs
        |
        +--> no review agent: task moves to review
        |
        +--> review enabled: one review run is queued per linked PR
        |                       |
        |                       v
        |              review comment is recorded; task moves to review
        v
GitHub PR close/merge webhook verifies every linked PR is terminal
        |
        v
Task moves from review to done
```

Failed or blocked runs remain failed in Vulcanum and do not automatically advance the task. Cancellation is signaled through the control plane and consumed by the worker session. Lifecycle labels and tracker comments expose the execution state alongside the task.

---

## Getting Started

### Prerequisites

Required for the control plane:

- Node.js 22 or newer and pnpm 11
- Rust stable with Cargo, rustfmt, and Clippy
- PostgreSQL 15 or newer
- Redis

Required according to worker mode:

- OpenCode or OMP installed for host execution
- Docker for Docker isolation
- Linux, KVM, Docker, and Kata Containers for Kata isolation

The provisioning CLI supports Linux services through systemd and macOS services through launchd. Kata setup is Linux-only. Windows release/setup automation is not currently provided.

### 1. Configure and Start the Control Plane

Install workspace dependencies:

```bash
pnpm install
```

Create a root `.env` file. These values are the minimum required by `vulcanum-web`:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/vulcanum
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=replace-with-a-long-random-secret
INSTANCE_PASSWORD=replace-with-a-login-password
IS_SINGLE_USER=true
MODEL_PROVIDER_SECRET_KEY=replace-with-a-base64-encoded-32-byte-key
```

`MODEL_PROVIDER_SECRET_KEY` must decode to exactly 32 bytes and must remain stable; changing it prevents existing model-provider credentials from being decrypted.

Apply migrations and start the API and dispatcher in separate terminals:

```bash
pnpm migrate-server-up
cargo run -p vulcanum-server --bin vulcanum-web
```

```bash
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

The API listens on `http://localhost:8000`. Start the frontend development server separately:

```bash
pnpm run dev --filter=@repo/frontend
```

The frontend development server listens on `http://localhost:5173` and uses `http://localhost:8000` as its default API URL.

Kaneo credentials are not global environment variables. After logging in, configure a task-tracker provider in **Settings**, then connect a tracker project to repositories and workflow columns from the task-board/project configuration UI.

### 2. Configure GitHub

Repository cloning and PR lifecycle automation use a GitHub App. Create an app with:

- Callback URL: `http://localhost:8000/api/v1/github/callback`
- Webhook URL: `http://localhost:8000/api/v1/github/webhook`
- Webhook event: **Pull request**
- Repository permissions:
  - **Contents:** read and write
  - **Pull requests:** read and write

Add the app configuration to `.env`:

```bash
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY=base64-encoded-private-key-pem
GITHUB_APP_SLUG=vulcanum-app
GITHUB_WEBHOOK_SECRET=replace-with-the-app-webhook-secret
```

`GITHUB_APP_PRIVATE_KEY` is the base64 encoding of the complete PEM file. Restart `vulcanum-web`, then install or connect the app from the GitHub section in **Settings**.

GitHub OAuth is separate from the GitHub App. It is only needed for multiuser login and team invites:

```bash
IS_SINGLE_USER=false
GITHUB_OAUTH_CLIENT_ID=your-client-id
GITHUB_OAUTH_CLIENT_SECRET=your-client-secret
GITHUB_OAUTH_REDIRECT_URL=http://localhost:8000/api/v1/auth/github/callback
```

### 3. Configure Models and Team Defaults

Use **Settings → Model providers** to add model-provider credentials. Use **Settings → Model selection** and **Team defaults** to choose the primary/small models, agent backend, prompt templates, maximum turns, review behavior, and per-team in-progress limit.

The server encrypts stored model credentials with `MODEL_PROVIDER_SECRET_KEY`. The selected worker still receives the decrypted execution configuration for the duration of a job; see [Security and Isolation](#security-and-isolation).

### 4. Register a Worker

Generate a worker registration code from the **Workers** page, then run setup on the worker host:

```bash
# Docker isolation with the default OpenCode backend.
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation docker \
  --agent-backend opencode

# Docker isolation with OMP RPC.
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation docker \
  --agent-backend omp-rpc

# Linux only: Kata runtime.
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation kata
```

`vulcanum wrk` is an alias for `vulcanum worker`. Passing `--isolation none` selects host execution. If `--instance` and `--code` are supplied without `--isolation`, non-interactive setup defaults to Docker.

The CLI configures and starts the platform service. To run an already-installed daemon directly:

```bash
vulcanum worker daemon
```

---

## Security and Isolation

### Execution Modes

| Mode | Boundary | Current behavior | Use |
| --- | --- | --- | --- |
| Host | None | Runs the backend as the worker's operating-system user in a per-run workspace. | Trusted development only. |
| Docker | Container | Runs the backend in the configured agent image and removes the container and temporary work directory after the run on a best-effort basis. | Baseline isolation on owned workers. |
| Kata | Lightweight VM through the Docker Kata runtime | Uses the Docker path with `kata-runtime`; requires Linux KVM and a working Kata installation. | Stronger workload isolation where supported. |

All backends enforce a maximum run duration through the runtime/session loop. The code carries default CPU and memory values, but current Docker launch paths do not constitute a complete configurable CPU/memory policy layer. Validate and harden the host, Docker daemon, network policy, mounts, image, and Kata runtime for your threat model.

### Credentials

- GitHub App installation tokens are repository-scoped and short-lived. The worker uses a credential bridge for Git/`gh` operations rather than leaving the token in the ordinary agent environment.
- Model-provider credentials are encrypted at rest with AES-256-GCM using `MODEL_PROVIDER_SECRET_KEY`.
- The server decrypts the credentials needed for a job and sends them to the authenticated worker. Use HTTPS between server and workers; plaintext HTTP is suitable only for isolated local development.
- Docker and Kata jobs receive required provider values inside their execution environment. Host jobs share the worker user's security context.
- A worker-side credential broker/vault that avoids handing reusable plaintext credentials to agent processes is planned, not implemented.

### Authentication and Tenancy

Single-user mode uses an instance password. Multiuser mode uses GitHub OAuth, teams, memberships, and invite links. These application-level controls do not by themselves make a deployment a hardened hostile multi-tenant environment. Use separate infrastructure or additional policy enforcement when tenants do not trust one another.

---

## Integrations

### Task-Tracker Backends

| Backend | Status |
| --- | --- |
| Kaneo | Active |
| Linear | Planned |
| Jira | Planned |
| GitHub Issues | Planned |

The current provider interface covers project/workspace discovery, task polling, board reads, task creation and updates, status movement, comments, and labels. A new tracker still requires a server adapter and configuration/UI support; it is not enabled by configuration alone.

### Agent Backends

| Backend | Status |
| --- | --- |
| OpenCode | Active |
| OMP RPC (`omp`) | Active |
| Additional coding-agent runtimes | Planned through the shared runtime/session interfaces |

Agent backend and model provider are separate choices. OpenCode and OMP are execution runtimes; Anthropic, OpenAI, and other entries supplied by the models.dev catalog are model-provider configurations used by those runtimes when compatible.

### Repository/VCS Backends

| Backend | Status |
| --- | --- |
| GitHub App | Active |
| GitLab | Planned |
| Bitbucket | Planned |

### Higher-Level Harnesses

OpenClaw, Hermes, and custom orchestrators are part of the target architecture, but Vulcanum does not yet ship dedicated adapters or an SDK for them. The current supported entry point is the tracker-driven workflow. A stable automation API and first-party harness adapters are roadmap items.

---

## Releases

Published artifacts are listed on the [GitHub Releases page](https://github.com/EzyGang/vulcanum/releases).

The current release workflow runs on a self-hosted runner and uploads two un-packaged binaries:

- `vulcanum` — worker setup/management CLI
- `vulcanum-server` — worker daemon

The control-plane server, dispatcher, and frontend are not included in those release assets. Windows/macOS binaries, installers, archives, containerized control-plane releases, and code signing are not currently produced by the release workflow. Build from source when the published worker artifacts do not match the target platform.

---

## Roadmap

Planned work is directional and not part of the current contract:

- Worker-side credential broker/vault and narrower secret exposure.
- Stable automation API plus adapters or SDKs for higher-level harnesses.
- Additional task-tracker adapters, including Linear, Jira, and GitHub Issues.
- A generalized VCS boundary beyond GitHub App repositories and webhooks.
- Additional agent runtime adapters through the shared runtime/session interfaces.
- Configurable and enforceable worker resource/network policies.
- Durable exported artifacts and session history beyond the current run/event records and worker-local message exports.
- Stronger tenant isolation, authorization, and policy controls.
- Repository readiness checks for branch protection, required CI, and review policy.

---

## Repository Layout

| Package | Path | Technology | Responsibility |
| --- | --- | --- | --- |
| CLI | `cli/` | Rust | Provision, register, run, and remove workers. |
| Worker daemon | `worker-server/` | Rust, SQLite | Poll, recover, isolate, execute, stream events, and submit results. |
| Control plane | `server/` | Rust, Actix Web, PostgreSQL, Redis | API, auth, providers, polling, dispatch coordination, run lifecycle, and GitHub webhooks. |
| Shared | `shared/` | Rust | API contracts, worker config/state, runtime traits, validation, and telemetry. |
| Frontend | `frontend/` | TypeScript, Preact, Signals, Tailwind CSS | Task board and control-plane UI. |
| Agent image | `docker/agent/` | Docker | Container image containing OpenCode, OMP, GitHub CLI, Kaneo CLI, and common development tools. |

JavaScript packages use pnpm workspaces and Turborepo. Rust crates are members of the root Cargo workspace.

---

## Development

### Common Commands

Run commands from the repository root:

| Command | Purpose |
| --- | --- |
| `pnpm install` | Install workspace dependencies. |
| `pnpm run build` | Build the Rust and frontend workspace through Turborepo. |
| `pnpm run format` | Format workspace source files. |
| `pnpm run validate` | Run Rust Clippy plus frontend linting and type-checking. |
| `pnpm run test` | Run workspace tests. Backend tests require a migrated PostgreSQL database. |
| `pnpm run dev` | Start development tasks, including the frontend dev server. |
| `pnpm migrate-server-up` | Apply server PostgreSQL migrations. |
| `pnpm migrate-server-down` | Revert the latest server migration. |
| `pnpm prep-queries` | Regenerate SQLx offline query metadata after query changes. |
| `cargo run -p vulcanum-server --bin vulcanum-web` | Start the control-plane API and poller. |
| `cargo run -p vulcanum-server --bin vulcanum-dispatcher` | Start the dispatcher. |
| `cargo run -p vulcanum-worker-server --bin vulcanum-server` | Start the worker daemon from source. |
| `cargo run -p vulcanum-cli --bin vulcanum -- worker setup` | Run interactive worker setup from source. |

The main CI workflow runs `pnpm run validate` and `pnpm run test` on self-hosted runners with `SQLX_OFFLINE=true` for compilation. Before review, run:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

Run `pnpm prep-queries` after changing backend SQL queries.

---

## Contributing

Contributions should preserve the separation between task trackers, control-plane lifecycle logic, isolation providers, and agent runtimes. Before a larger change, describe:

- **Problem:** the operational or user-facing gap.
- **Boundary:** whether the change belongs to a tracker adapter, control-plane service, VCS integration, worker, isolation provider, agent runtime, or UI.
- **Tradeoffs:** security, compatibility, persistence, migration, or operational costs.
- **Validation:** the focused tests and end-to-end scenario that prove the behavior.

Read [AGENTS.md](AGENTS.md) and the module-specific `AGENTS.md` before changing code. Keep the server's HTTP → service → repository layering intact, add business-logic tests for state transitions and errors, and include any validation command that could not run plus its exact blocker in the pull request.

---

## License

Vulcanum is licensed under `AGPL-3.0-or-later`. See [LICENSE](LICENSE).
