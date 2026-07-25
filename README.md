# Vulcanum

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/EzyGang/vulcanum/ci.yml?style=for-the-badge" alt="CI">
  <img src="https://img.shields.io/github/v/release/EzyGang/vulcanum?style=for-the-badge" alt="GitHub release">
  <img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later">
</p>

Vulcanum is being built as an agnostic orchestration layer between task trackers, implementation agents, and higher-level harnesses.

It gives those systems one place to hand work off, run it, follow what happened, and return the result. The tracker does not need to know how a coding agent works. The coding agent does not need tracker-specific logic. A higher-level agent should be able to use the same Vulcanum CLI regardless of which tracker or execution backend sits behind it.

Execution also does not have to live on one machine. A worker can run on an idle PC or Mac at home while other workers run on a VPS or another machine. Vulcanum treats them as one pool and sends compatible work wherever capacity is available.

The point is not to build another task tracker, another coding agent, or just another local agent runner. Vulcanum is the layer that lets those parts change independently and still work together.

> [!WARNING]
> Vulcanum is still pre-1.0. It is under active development, migrations and interfaces can change, and it should be run on infrastructure you control.

## Contents

- [Installation](#installation)
- [CLI Reference](docs/cli-reference.md)
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

## Installation

The release installer downloads the `vulcanum` CLI and the `vulcanum-server` worker daemon on Linux or macOS:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh | sh
```

The installer:

- supports x86_64 and ARM64 Linux and macOS;
- downloads the latest release archive and its SHA-256 checksum;
- verifies the archive before installing either binary;
- installs to `~/.local/bin` by default and, when needed, prints commands to update `PATH` for the current shell and persist it in Bash, Zsh, or Fish.

It requires `tar`, `awk`, `sed`, either `curl` or `wget`, and either `sha256sum` or `shasum`. To inspect the script before running it:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  -o install.sh \
  https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh
less install.sh
sh install.sh
```

Set `VULCANUM_VERSION` to install a specific release or `VULCANUM_INSTALL_DIR` to choose another destination:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh |
  VULCANUM_VERSION=0.1.0 VULCANUM_INSTALL_DIR="$HOME/bin" sh
```

This installs the worker-side binaries only. The control-plane server, dispatcher, and frontend must still be deployed separately or run from source as described under [Running Locally](#running-locally).

### Install the agent skills

After installing the CLI, install both Vulcanum agent skills into a supported coding agent:

```bash
vulcanum skills install
```

The command uses the first available package runner—`pnpm dlx`, `npx`, `bunx`, or `yarn dlx`—to install `vulcanum-cli` and `vulcanum-ticket-template` from this repository. At least one supported JavaScript package runner must be available.

Install only one skill when needed:

```bash
vulcanum skills install cli
vulcanum skills install ticket-template
```

To place a skill manually or pass it directly to an agent, print its complete `SKILL.md` to standard output:

```bash
vulcanum skills install ticket-template --stdout > ./SKILL.md
```

See the [CLI reference](docs/cli-reference.md#agent-skills) for canonical skill names and command behavior.

---

## What Vulcanum Is For

The main problem Vulcanum tries to solve is coupling.

Without a middle layer, the code that picks up a task usually knows about the tracker, the agent runner, the machine that will execute it, the repository host, and the way results should be reported. Adding a second tracker or agent backend means rebuilding a large part of that glue.

Vulcanum puts one boundary in the middle of three sides:

1. **Task trackers**, where people plan and follow the work.
2. **Execution backends**, where implementation and review agents actually run.
3. **Higher-level harnesses**, such as OpenClaw, Hermes, or a personal agent that wants to delegate software work.

That separation is the main idea. Task trackers connect through provider interfaces. OpenCode, OMP, and future implementation agents connect through worker runtime interfaces. Higher-level harnesses will use a shared `SKILL.md` and the Vulcanum CLI rather than getting a custom integration for every harness.

The distributed worker setup follows from that design. Workers can be on the same network or spread across different machines. You might keep one on a home machine that would otherwise sit idle, run another on a Mac, and add VPS workers for always-on capacity. They all connect to the same control plane and advertise what they can run.

Worker registration, dispatch, concurrency, and crash recovery are necessary parts of making this reliable, but they are not the main selling point by themselves. The useful part is having one neutral layer that joins trackers, execution, and higher-level automation without making any one of them the center of the stack.

## Where It Fits

### Task-tracker side

This is the working entry point today.

You configure a project, choose the column Vulcanum should watch, connect one or more GitHub repositories, and choose the execution settings. Moving a task into the pickup column starts the workflow. Vulcanum keeps the task updated as implementation, review, and pull request state change.

Kaneo is the current tracker backend. The server-side provider boundary is intended to support other trackers without changing the worker or agent runtime.

### Execution side

Workers connect to the control plane from whichever machines you want to use. Each worker reports its available capacity and execution mode. Vulcanum can then use local machines and remote hosts as one worker pool instead of tying the workflow to a single computer.

OpenCode and OMP are the current agent backends. Host, Docker, and Kata are the current execution modes.

### Higher-level harness side

OpenClaw, Hermes, or another personal agent should not need a dedicated Vulcanum adapter.

The planned connection is a shareable `SKILL.md` plus an installed Vulcanum CLI. The skill explains how to use Vulcanum and the CLI provides the commands. Anything that can install the skill and call the CLI can use the same interface.

The CLI supports worker lifecycle management plus authenticated worker and settings inspection. Higher-level run-control commands are still planned.

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
- **OpenCode and OMP.** Team defaults select the agent backend for each run, and every worker can execute either backend.
- **GitHub repositories and pull requests.** A GitHub App provides repository access, short-lived clone credentials, pull request tracking, and close/merge webhooks.
- **Host, Docker, and Kata execution.** Each worker is configured with one execution mode.
- **Concurrent workers.** The dispatcher respects each worker's available job slots.
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

This is the CLI. It provisions, registers, starts, and removes workers, and exposes authenticated worker and settings inspection commands. See the [CLI reference](docs/cli-reference.md) for command syntax, authentication, and team-selection behavior.

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

<details>
<summary>Show local development setup</summary>

#### Prerequisites

For the control plane:

- Node.js 22 or newer
- pnpm 11
- Rust stable with Cargo, rustfmt, and Clippy
- PostgreSQL 15 or newer
- Redis

For host workers, both OpenCode and OMP must be installed. Docker execution needs Docker. Kata execution needs Linux, KVM, Docker, and Kata Containers.

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

Repository cloning, pull request tracking, and review triggers use a GitHub App. For local
development, configure:

- Callback URL: `http://localhost:8000/api/v1/github/callback`
- Enable **Request user authorization (OAuth) during installation**
- The optional Setup URL may use the same callback endpoint
- Webhook URL: `http://localhost:8000/api/v1/github/webhook`
- Webhook events: **Pull request** and **Issue comment**
- **Contents** permission: read and write
- **Pull requests** permission: read and write
- **Issues** permission: read and write

The callback accepts both GitHub response shapes. OAuth responses are correlated through a
single-use state nonce; installation responses are verified against the GitHub App API before
Vulcanum stores them.

To review any open pull request in a repository connected to an enabled project, an authorized
team member can comment `@app-slug review`. If the repository belongs to multiple review-enabled
projects, Vulcanum replies with project-specific commands; use
`@app-slug review project:<project-config-uuid>` to select one. GitHub models pull request timeline
comments as issue comments, so the app needs **Issues: read** to receive commands and **Issues:
write** to post project-selection guidance.

Add the app values to `.env`:

```bash
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY=base64-encoded-private-key-pem
GITHUB_APP_SLUG=vulcanum-app
GITHUB_WEBHOOK_SECRET=replace-with-the-app-webhook-secret
```

`GITHUB_APP_PRIVATE_KEY` is the base64 encoding of the complete PEM file.

GitHub user authorization links PR commenters to Vulcanum. The credentials may belong to the same
GitHub App or to a separate OAuth App:

```bash
GITHUB_CLIENT_ID=your-client-id
GITHUB_CLIENT_SECRET=your-client-secret
GITHUB_OAUTH_REDIRECT_URL=http://localhost:8000/api/v1/github/callback
```
`GITHUB_OAUTH_REDIRECT_URL` is the public callback that GitHub returns the temporary authorization
code to. It must exactly match a callback URL configured on the GitHub App or OAuth App.

Restart `vulcanum-web`, then connect the app from the GitHub section in Settings. In single-user
mode, select **Link account** there and authorize the GitHub account allowed to trigger reviews. In
multiuser mode, set `IS_SINGLE_USER=false`; each commenter must sign in through GitHub and belong
to the team that owns the project.

Model providers, model selection, team defaults, tracker providers, repositories, workflow columns, prompts, and review settings are configured in the UI.

</details>

### Registering a Worker

<details>
<summary>Show worker registration commands</summary>

Generate a registration code from the Workers page, then run setup on the worker host.

Docker workers include both supported agent runtimes. The backend for each run is selected from the team's defaults:

```bash
vulcanum worker setup \
  --instance http://<control-plane-host>:8000 \
  --code <registration-code> \
  --isolation docker
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

</details>

### Repository Layout

| Path | What is in it |
| --- | --- |
| `cli/` | Rust CLI for worker provisioning, registration, service setup, and management. |
| `worker-server/` | Rust worker daemon, execution environments, agent runtimes, recovery, SQLite journal, and local session storage. |
| `server/` | Actix Web control plane, dispatcher, PostgreSQL repositories, business services, provider integrations, and GitHub webhooks. |
| `shared/` | Shared API types, client, worker configuration, runtime traits, validation, paths, and telemetry. |
| `frontend/` | Preact control UI. |
| `docker/agent/` | Worker image containing OpenCode, OMP, GitHub CLI, Kaneo CLI, and common development tools. |

The TypeScript packages use pnpm workspaces and Turborepo. The Rust crates are members of the root Cargo workspace.

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

Published archives are listed on the [GitHub Releases page](https://github.com/EzyGang/vulcanum/releases).

Each release includes `vulcanum` and `vulcanum-server` in checksum-protected archives for:

- x86_64 and ARM64 Linux;
- x86_64 and ARM64 macOS.

Use the [installation script](#installation) to select the archive for the current platform automatically. The release workflow does not publish the control-plane server, dispatcher, or frontend.

### Contributing

Read the root [AGENTS.md](AGENTS.md) and the module-specific `AGENTS.md` before changing code.

Keep tracker behavior, control-plane business logic, worker execution, isolation, and agent runtimes in their existing layers. Add tests for application behavior and state transitions rather than framework plumbing. Include the commands you ran in the pull request, along with the exact blocker for anything that could not run.

### License

Vulcanum is licensed under `AGPL-3.0-or-later`. See [LICENSE](LICENSE).
