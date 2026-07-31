# Isolation and recovery

## Host isolation

Host mode creates a separate work directory, but it does not create a security boundary. The agent process has the permissions of the worker operating-system user. It shares the host filesystem and network namespace.

Use host mode only for trusted work in a single-tenant environment. Install both OpenCode and OMP on the host.

## Docker isolation

Docker mode starts each job in the configured agent image. The worker gives the container a prepared workspace and starts the selected runtime. After the job, it removes the container and temporary work directory on a best-effort basis.

Use a controlled image. Keep the default image current, or set `image` in `~/.vulcanum/config.json` to an image that contains all required tools.

## Kata isolation

Kata mode uses the Docker execution path with `kata-runtime`. It adds a lightweight virtual-machine boundary when Kata, Docker, and KVM are configured correctly.

Kata setup is Linux-only. The worker host must give the service access to KVM.

## Runtime credentials

The control plane sends only the selected job configuration to the worker. The worker writes provider-specific runtime configuration inside the job environment.

GitHub access uses a short-lived installation token. The worker configures Git and `gh` credential helpers. The direct token is not part of the ordinary agent environment.

Model credentials are reusable secrets. Restrict access to the worker host, daemon service account, and job environment.

## SQLite journal

The daemon stores job state in `~/.vulcanum/worker.db`. A journal row contains the job and work directory, isolation and runtime data, session ID, turn state, result metadata, and timestamps.

The main state flow is:

```text
running -> completed | failed | lost -> submitted
```

The journal lets the daemon reconcile work after a process or host restart.

## Startup recovery

At startup, the daemon reads all journal rows in `running` state.

For each row, it uses this order:

1. If a complete finish artifact exists, submit it without rerunning completed agent work.
2. If review findings need another pass, resume the review flow.
3. For an OMP RPC job, start OMP-specific session recovery.
4. For OpenCode, check that the host process or container is alive.
5. Find the recorded session and inspect its status.
6. Reconnect to a live session when possible.
7. If the process, container, port, or session is missing, clean stale resources and submit the job as lost.

Recovery reserves a worker job slot. Recovered work and new work use the same concurrency limit.

Do not delete `worker.db` while a job can still be active. That removes the daemon's recovery record.

## Message history

Before cleanup, the daemon gets the full message history from the agent runtime. It writes raw JSON under:

```text
~/.vulcanum/sessions/<work-run-id>/<session-id>.json
```

The control plane stores ordered run events and usage, but it does not currently store this exported backend message history. Back up the worker session directory if you need long-term retention.

## Operational checks

After an unexpected restart:

1. Start the worker service.
2. Inspect the worker logs for recovery messages.
3. Confirm that the worker becomes visible in the web UI.
4. Inspect active and failed runs in **Runs**.
5. Check for stale `vulcanum-*` containers if cleanup reported an error.
6. Keep the journal and session files until reconciliation finishes.

Do not manually submit a replacement job while recovery is reconnecting to the original session. This can cause two agents to work on the same task.
