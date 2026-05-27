# MVP Implementation Tasks

## P0 — Core Infrastructure (nothing works without these)

- [ ] **Server background poller**: `tokio::spawn` + `tokio::time::interval`, polls Kaneo per enabled project, filters against DB, inserts work_runs with `ON CONFLICT DO NOTHING`
- [ ] **Kaneo API client**: fetch tasks by project + column slug, PATCH task status, POST comment. Configurable base URL + API key (set by user per-project)
- [ ] **Work runs DB schema + migrations**: `project_configs`, `workers`, `work_runs` tables with unique constraint. SQLx migrations in `server/migrations/`
- [ ] **Project config CRUD**: `GET/POST/PUT /projects`, `POST /projects/:id/columns` (fetch Kaneo columns for mapping UI). Service + repository layers
- [ ] **Worker registration flow**: `POST /workers/codes` (generate short-lived code), `POST /workers/connect` (exchange code for token pair), `POST /workers/refresh` (refresh access token). In-memory code store + DB token hashing
- [ ] **HTTP polling endpoints**: `GET /poll?worker_id=X` (in-memory boolean cache flag, 200/204), `GET /jobs/:id` (full work spec), `POST /jobs/:id/ack`, `POST /jobs/:id/result`. Service + route layers

## P1 — Worker + Agent Execution

- [ ] **Worker daemon binary**: `vulcanum` with subcommands (`connect`, `daemon`, future: `setup-worker`, `tui`). Tokio-based async runtime
- [ ] **Poll loop in worker**: `GET /poll` every 15s, backoff on unreachable. Token refresh before expiry. Worker status updates (idle/busy)
- [ ] **Kata Containers isolation**: run container via Docker with `--runtime=kata-runtime`, mount tmpfs workdir as volume, network egress-only, CPU/memory limits. Container lifecycle management (run, wait, collect, destroy)
- [ ] **OpenCode harness adapter**: spawn OpenCode inside Kata container with rendered prompt + secrets in env. Parse output (exit code, PR URL, token usage). Timeout enforcement
- [ ] **Prompt template rendering**: interpolate `{task_title}`, `{task_body}`, `{repo_url}`, `{branch}` into per-project template. Default template shipped in code
- [ ] **Result submission + Kaneo sync**: worker POSTs `/jobs/:id/result` → main app PATCHes Kaneo status to target column, posts comment with PR link. Handle failure case (status back to pickup)

## P2 — Reliability + Observability

- [ ] **In-memory boolean cache for /poll**: per-worker flag, flipped when new work_run created for that worker. Reset on ack. Avoids DB hit on every 15s poll
- [ ] **Worker heartbeat / liveness**: `last_seen` timestamp updated on each poll. Stale threshold → worker marked `disconnected` → no new work dispatched. Work run timeout: if running > max, mark `stalled` → re-queue
- [ ] **CLI `vulcanum connect` command**: takes `--instance` and `--code`, registers with main app, stores token pair locally, daemonizes (systemd user service)
- [ ] **Duplicate prevention**: `ON CONFLICT DO NOTHING` on insert, unique constraint on `(external_task_ref, status) WHERE status IN ('pending', 'dispatched', 'running')`
- [ ] **Token revocation**: `DELETE /workers/:id` removes worker from DB → refresh token invalid → worker can't refresh → disconnected. Worker list endpoint for UI

## P3 — Developer Experience

- [ ] **Worker setup script**: `vulcanum setup-worker` installs Docker + kata-runtime, pulls container image, installs OpenCode, configures systemd. For Ubuntu 22.04+ initially
- [ ] **Container image build**: Dockerfile producing image with OpenCode, git, curl, SSH. Build script or pre-built image push
- [ ] **Basic API UI**: minimal page for generating worker codes, listing workers, toggling projects on/off. Can be static HTML served by actix-web
- [ ] **Logging**: `tracing` crate, structured JSON in production, token usage metrics per run

## Not in MVP (explicitly deferred)

- Multi-user auth gating (code exists, just not enforced on API)
- WebSockets / SSE — HTTP polling is sufficient
- Claude Code / Codex CLI harnesses — OpenCode only
- Agent-vault proxy — secrets flow through main app over HTTPS
- Vulcanum API SKILL.md — CLI + API is enough for now
- TUI — `vulcanum connect` command + API is the interface
- macOS support — Kata Containers requires Linux KVM
- Verifier agent (second OpenCode run) — human reviews PRs
- CI status polling in main app — it's a metadata broker, not a CI watcher
- Linear/Jira integrations — Kaneo only
