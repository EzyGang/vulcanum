# Vulcanum

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/EzyGang/vulcanum/ci.yml?style=for-the-badge" alt="CI">
  <img src="https://img.shields.io/github/v/release/EzyGang/vulcanum?style=for-the-badge" alt="GitHub release">
  <img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later">
</p>

Vulcanum connects the parts around an implementation agent.

It watches work in a task tracker, turns that work into jobs, sends those jobs to available workers, and keeps the original task up to date while the agent works. If the agent opens a pull request, Vulcanum can run a separate review agent and wait for the pull request to be closed or merged before marking the task done.

The point is not to build another task tracker or another coding agent. Vulcanum is the layer between them.

> [!WARNING]
> Vulcanum is still pre-1.0. It is under active development, migrations and interfaces can change, and it should be run on infrastructure you control.

## Contents

- [What Vulcanum Is For](#what-vulcanum-is-for)
- [Where It Fits](#where-it-fits)
- [A Typical Run](#a-typical-run)
- [What Works Today](#what-works-today)
- [What Is Still Planned](#what-is-still-planned)
- [Technical Reference](#technical-reference)
  - [Architecture](#architecture)
  - [Processes and Storage](#processes-and-storage)
  - [Security and Execution](#security-and-execution)
  - [Running Locally](#running-locally)
  - [Registering a Worker](#registering-a-worker)
  - [Repository Layout](#repository-layout)
  - [Development](#development)
  - [Releases](#releases)
  - [Contributing](#contributing)
  - [License](#license)

---

## What Vulcanum Is For

A small agent setup is easy enough: read a task, run a coding agent, and hope it leaves the repository in a useful state.

The awkward parts show up later:

- Which worker should get the task?
- Is that worker already busy?
- Which repository or repositories belong to the task?
- Which agent backend and model should be used?
- What happens when a worker restarts halfway through a run?
- Where do live events, token usage, results, and pull requests go?
- Who updates the task tracker?
- Should a second agent review the pull request?
- When is the task actually done?

Vulcanum handles that middle layer. The tracker remains useful to the people planning the work. The coding agent remains focused on implementation. Higher-level systems can ask Vulcanum to manage execution instead of learning how every worker and backend works.

It is meant to be independent of the AI model, coding agent, task tracker, and higher-level harness around it. The current set of integrations is smaller than that goal, but the boundaries are already separate in the code.

## Where It Fits

There are two ways we expect people to use Vulcanum.

### From a task tracker

This is the working path today.

You configure a project, choose the column Vulcanum should watch, connect one or more GitHub repositories, and choose the worker/agent settings. Moving a task into the pickup column is enough to start the workflow. Vulcanum moves the task through progress, review, and done as the run changes state.

### From a higher-level harness

Systems such as OpenClaw, Hermes, or a personal agent should not need a dedicated Vulcanum adapter.

The planned connection is a shareable `SKILL.md` plus an installed Vulcanum CLI. The skill explains how to use Vulcanum, and the CLI provides the actual commands. This keeps the connection agent-agnostic: anything that can install the skill and call the CLI can use the same interface.

The current CLI is mainly for setting up and managing workers. It still needs the higher-level control commands before this path is complete.

## A Typical Run

1. A task enters the configured pickup column.
2. Vulcanum creates a pending implementation run.
3. The dispatcher picks a compatible worker with free capacity.
4. The worker acknowledges the job and the task moves to progress.
5. The worker clones the configured repositories using a short-lived GitHub App token.
6. OpenCode or OMP works on the task in the worker's configured execution environment.
7. Events, usage, and status are sent back while the run is active.
8. The agent finishes with a summary, result status, and any pull request URLs.
9. Vulcanum adds the result to the original task and moves it to review.
10. If agent review is enabled, Vulcanum creates a separate review run for each pull request before moving the task to review.
11. Once every linked pull request is closed or merged, Vulcanum moves the task to done.

A failed or blocked run stays failed in Vulcanum and does not quietly advance the task. Runs can also be cancelled from the control plane.

## What Works Today

The current implementation supports:

- **Kaneo projects and task boards.** Vulcanum can read projects, watch configured columns, create and update tasks, move tasks, post comments, and manage labels.
- **OpenCode and OMP.** Workers select one of these agent backends during setup and advertise that capability to the server.
- **GitHub repositories and pull requests.** A GitHub App provides repository access, short-lived clone credentials, pull request tracking, and close/merge webhooks.
- **Host, Docker, and Kata execution.** Each worker is configured with one execution mode.
- **Concurrent workers.** The dispatcher respects worker capabilities and available job slots.
- **Crash recovery.** Workers keep an SQLite journal for in-flight jobs and reconcile it when they restart.
- **Multi-repository jobs.** A project can provide more than one repository to an implementation run.
- **Multi-turn runs.** The agent can continue for the configured number of turns and ends through an explicit `finish_run` contract.
- **Optional agent review.** Implementation pull requests can be handed to separate review runs.
- **Live run data.** The server stores ordered events, status, worker, model, duration, token counts, summaries, and pull request URLs.
- **Team and project settings.** Models, prompts, turn limits, review settings, concurrency limits, repositories, and workflow columns can be configured at the appropriate level.
- **Model-provider credentials.** Credentials are encrypted in PostgreSQL and turned into backend-specific job configuration when a run starts.
- **Single-user and team modes.** A deployment can use an instance password or GitHub OAuth with teams and invite links.
- **A web UI.** The frontend includes the task board, dashboard, runs, workers, teams, and settings for tracker providers, model providers, models, GitHub, and defaults.

A few boundaries are worth being clear about:

- Kaneo is the only task-tracker backend right now.
- GitHub is the only repository and pull request backend right now.
- OpenCode and OMP are the only implementation-agent backends right now.
- Higher-level harness control through `SKILL.md` and the CLI is planned, not finished.
- Run records and events are stored centrally. Exported backend message history is currently saved on the worker under `~/.vulcanum/sessions/`; there is no server-side artifact bundle yet.
- Model credentials are encrypted at rest, but the server must decrypt the credentials needed by a job and send them to the worker. The credential-vault work described below is not implemented yet.

## What Is Still Planned

The main pieces still ahead are:

- A shareable Vulcanum `SKILL.md` for higher-level harnesses.
- More CLI commands so an agent or harness can inspect work, start or manage runs, and use Vulcanum without going through the web UI.
- A credential vault/broker so jobs can use credentials without handing reusable plaintext secrets directly to the agent process.
- More task-tracker backends.
- More implementation-agent backends.
- Repository and pull request backends beyond GitHub.
- Central storage for exported session history and run artifacts.

These are planned directions, not features hidden behind configuration.

---

## Technical Reference

Everything below is implementation and setup detail. If you only wanted to understand what Vulcanum does, the sections above are the useful part.

### Architecture

```mermaid
flowchart LR
  tracker[Task-tracker backend] <--> tracker_adapter[Task-tracker adapter]
  tracker_adapter <--> server[Vulcanum control plane]

  user[User] --> ui[Web UI]
  ui --> server

  harness[OpenClaw / Hermes / another agent] -. planned .-> skill[Shared SKILL.md + Vulcanum CLI]
  skill -. planned CLI control commands .-> server

  server <--> postgres[(PostgreSQL)]
  server <--> redis[(Redis)]
  dispatcher[Dispatcher] <--> postgres
  dispatcher <--> redis

  worker[Worker daemon] -->|poll / ack / events / result| server
  worker <--> journal[(SQLite journal)]
  worker --> isolation[Host / Docker / Kata]
  isolation --> runtime[Agent runtime]
  runtime --> opencode[OpenCode]
  runtime --> omp[OMP RPC]

  server <--> github[GitHub App]
  github --> repos[Git repositories and pull requests]
  isolation --> repos
```

There are three extension boundaries that matter:

1. **Task-tracker backends** live on the server side and translate tracker projects, columns, tasks, labels, and comments into Vulcanum's internal types.
2. **Agent backends** live on the worker side and implement the shared runtime/session contract.
3. **Execution environments** prepare the workspace independently of the selected agent backend.

The planned higher-level harness path is deliberately different. It uses shared instructions and CLI commands rather than an adapter for each harness.

### Processes and Storage

#### `vulcanum-web`

The Actix Web control-plane process:

- exposes the `/api/v1` HTTP API;
- runs PostgreSQL migrations at startup;
- polls enabled task-tracker projects;
- handles authentication, teams, providers, workers, projects, jobs, and runs;
- processes GitHub callbacks and pull request webhooks;
- serves the services used by the frontend and workers.

It listens on port `8000`.

#### `vulcanum-dispatcher`

The dispatcher is a separate process. It finds pending work, looks for compatible workers with free capacity, reserves a worker slot, and signals the selected worker through Redis.

#### `vulcanum-server`

Despite the binary name, this is the worker daemon. It polls the control plane, prepares a workspace, starts OpenCode or OMP, reports events, submits the result, and cleans up the run environment.

#### `vulcanum`

This is the CLI. Today it provisions, registers, starts, and removes workers. The higher-level harness commands are planned additions to this binary.

#### Frontend

The Preact frontend is the control UI. It talks to `vulcanum-web`; it does not communicate directly with PostgreSQL, Redis, workers, Kaneo, or GitHub.

#### Storage

- **PostgreSQL** stores users, teams, provider configuration, project configuration, workers, work runs, events, usage, linked pull requests, and encrypted model credentials.
- **Redis** carries dispatch and cancellation signals and backs short-lived registration, OAuth, invite, and webhook work.
- **Worker SQLite** stores the local execution journal used for restart recovery.
- **Worker filesystem** stores exported backend message history under `~/.vulcanum/sessions/`.

### Security and Execution

#### Execution modes

| Mode | What it means |
| --- | --- |
| Host | Runs the agent as the worker's operating-system user. It uses a separate workspace, but it is not a security boundary. Use it only for trusted work. |
| Docker | Runs the agent in the configured container image. The worker removes the container and temporary work directory after the run on a best-effort basis. |
| Kata | Uses the Docker execution path with `kata-runtime`, giving the job a lightweight virtual-machine boundary when Kata and KVM are set up correctly. |

Every backend has a maximum run duration enforced by the runtime/session loop.

#### Credentials

Model-provider credentials are encrypted with AES-256-GCM using `MODEL_PROVIDER_SECRET_KEY`. The key must decode to exactly 32 bytes and must remain stable for the life of the stored credentials.

For a job, the server decrypts the required provider configuration and sends it to the authenticated worker. Use HTTPS between the server and remote workers. Plain HTTP is only appropriate for an isolated local setup.

GitHub repository access uses short-lived installation tokens. The worker places the token behind Git and `gh` credential helpers instead of leaving the direct token in the ordinary agent environment.

#### Authentication

Single-user mode uses `INSTANCE_PASSWORD`. Multiuser mode uses GitHub OAuth, teams, memberships, and expiring invite links.

These application controls do not turn a shared deployment into a safe environment for mutually hostile tenants. Host mode in particular gives jobs the worker user's access.

### Running Locally

#### Prerequisites

For the control plane:

- Node.js 22 or newer
- pnpm 11
- Rust stable with Cargo, rustfmt, and Clippy
- PostgreSQL 15 or newer
- Redis

For workers, you also need the selected agent backend. Docker execution needs Docker. Kata execution needs Linux, KVM, Docker, and Kata Containers.

The setup CLI configures systemd on Linux and launchd on macOS. Kata setup is Linux-only. Windows setup and release automation are not currently provided.

#### Install and configure

```bash
pnpm install
```

Create a root `.env` file:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/vulcanum
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=replace-with-a-long-random-secret
INSTANCE_PASSWORD=replace-with-a-login-password
IS_SINGLE_USER=true
MODEL_PROVIDER_SECRET_KEY=replace-with-a-base64-encoded-32-byte-key
```

Changing `MODEL_PROVIDER_SECRET_KEY` later prevents existing model-provider credentials from being decrypted.

Apply migrations and start the two server processes in separate terminals:

```bash
pnpm migrate-server-up
cargo run -p vulcanum-server --bin vulcanum-web
```

```bash
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

Start the frontend:

```bash
pnpm run dev --filter=@repo/frontend
```

The API listens on `http://localhost:8000`. The frontend development server listens on `http://localhost:5173` and uses port `8000` as its default API URL.

Kaneo credentials are configured in the task-tracker provider settings, not through global `KANEO_INSTANCE` or `KANEO_API_KEY` environment variables.

#### GitHub App

Repository cloning and pull request tracking use a GitHub App. For local development, configure:

- Callback URL: `http://localhost:8000/api/v1/github/callback`
- Webhook URL: `http://localhost:8000/api/v1/github/webhook`
- Webhook event: **Pull request**
- **Contents** permission: read and write
- **Pull requests** permission: read and write

Add the app values to `.env`:

```bash
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY=base64-encoded-private-key-pem
GITHUB_APP_SLUG=vulcanum-app
GITHUB_WEBHOOK_SECRET=replace-with-the-app-webhook-secret
```

`GITHUB_APP_PRIVATE_KEY` is the base64 encoding of the complete PEM file. Restart `vulcanum-web`, then connect the app from the GitHub section in Settings.

GitHub OAuth is separate from the GitHub App. It is only needed for multiuser login:

```bash
IS_SINGLE_USER=false
GITHUB_OAUTH_CLIENT_ID=your-client-id
GITHUB_OAUTH_CLIENT_SECRET=your-client-secret
GITHUB_OAUTH_REDIRECT_URL=http://localhost:8000/api/v1/auth/github/callback
```

Model providers, model selection, team defaults, tracker providers, repositories, workflow columns, prompts, and review settings are configured in the UI.

### Registering a Worker

Generate a registration code from the Workers page, then run setup on the worker host.

OpenCode with Docker:

```bash
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation docker \
  --agent-backend opencode
```

OMP with Docker:

```bash
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation docker \
  --agent-backend omp-rpc
```

Kata on Linux:

```bash
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation kata
```

Use `--isolation none` for host execution. Non-interactive setup defaults to Docker when `--instance` and `--code` are provided without an isolation option.

`vulcanum wrk` is an alias for `vulcanum worker`. To start an already installed daemon directly:

```bash
vulcanum worker daemon
```

### Repository Layout

| Path | What is in it |
| --- | --- |
| `cli/` | Rust CLI for worker provisioning, registration, service setup, and management. |
| `worker-server/` | Rust worker daemon, execution environments, agent runtimes, recovery, SQLite journal, and local session storage. |
| `server/` | Actix Web control plane, dispatcher, PostgreSQL repositories, business services, provider integrations, and GitHub webhooks. |
| `shared/` | Shared API types, client, worker configuration, runtime traits, validation, paths, and telemetry. |
| `frontend/` | Preact control UI. |
| `docker/agent/` | Worker image containing OpenCode, OMP, GitHub CLI, Kaneo CLI, and common development tools. |

The JavaScript packages use pnpm workspaces and Turborepo. The Rust crates are members of the root Cargo workspace.

### Development

Run these from the repository root:

| Command | Purpose |
| --- | --- |
| `pnpm install` | Install workspace dependencies. |
| `pnpm run build` | Build the Rust and frontend workspace. |
| `pnpm run format` | Format workspace source files. |
| `pnpm run validate` | Run Rust formatting/Clippy and frontend lint/type-checking. |
| `pnpm run test` | Run workspace tests. Backend tests require a migrated PostgreSQL database. |
| `pnpm run dev` | Start development tasks. |
| `pnpm migrate-server-up` | Apply PostgreSQL migrations. |
| `pnpm migrate-server-down` | Revert the latest migration. |
| `pnpm prep-queries` | Regenerate SQLx offline metadata after query changes. |
| `cargo run -p vulcanum-server --bin vulcanum-web` | Start the API and poller. |
| `cargo run -p vulcanum-server --bin vulcanum-dispatcher` | Start the dispatcher. |
| `cargo run -p vulcanum-worker-server --bin vulcanum-server` | Start the worker daemon from source. |
| `cargo run -p vulcanum-cli --bin vulcanum -- worker setup` | Run worker setup from source. |

Before opening a pull request:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

Run `pnpm prep-queries` after changing server SQL queries.

### Releases

Published files are listed on the [GitHub Releases page](https://github.com/EzyGang/vulcanum/releases).

The current release workflow uploads two un-packaged binaries from a self-hosted runner:

- `vulcanum`, the CLI;
- `vulcanum-server`, the worker daemon.

It does not currently publish the control-plane server, dispatcher, frontend, installers, signed packages, or a platform matrix. Build from source when the published worker binaries do not match the target system.

### Contributing

Read the root [AGENTS.md](AGENTS.md) and the module-specific `AGENTS.md` before changing code.

Keep tracker behavior, control-plane business logic, worker execution, isolation, and agent runtimes in their existing layers. Add tests for application behavior and state transitions rather than framework plumbing. Include the commands you ran in the pull request, along with the exact blocker for anything that could not run.

### License

Vulcanum is licensed under `AGPL-3.0-or-later`. See [LICENSE](LICENSE).
