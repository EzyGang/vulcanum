# Workers

A worker is a machine that runs agent jobs. The `vulcanum-server` worker daemon polls the control plane and executes jobs with OpenCode or OMP RPC.

## Worker responsibilities

For each job, the worker:

1. Gets and acknowledges the assignment.
2. Records the job in a local SQLite journal.
3. Creates a separate workspace.
4. Prepares the configured isolation environment.
5. Clones the project repositories.
6. Starts the selected agent runtime.
7. Runs implementation or review turns.
8. Reports events and token usage.
9. Submits the final result.
10. Saves the agent message history.
11. Cleans the environment.

## Supported combinations

The team setting selects the agent runtime for each run:

- OpenCode;
- OMP RPC.

The worker setting selects one isolation mode for the worker:

- host;
- Docker;
- Kata Containers.

The runtime and isolation settings are independent. A Docker worker image includes both supported runtimes.

## Capacity

During setup, the CLI calculates worker capacity from CPU and memory:

- one job for each two CPU threads;
- one job for each 4 GiB of RAM;
- the lower result is used;
- capacity is limited to a minimum of 1 and a maximum of 3.

The server returns the accepted maximum during registration. The daemon uses that value to limit concurrent jobs. If all slots are in use, new job IDs wait in a local queue.

## Local files

| Path | Content |
| --- | --- |
| `~/.vulcanum/config.json` | Worker configuration |
| `~/.vulcanum/worker.json` | Worker ID, instance URL, tokens, and accepted capacity |
| `~/.vulcanum/worker.db` | SQLite recovery journal |
| `~/.vulcanum/sessions/<work-run-id>/` | Exported agent message history |
| `~/.vulcanum/app.json` | Separate CLI application login state |

Protect the `.vulcanum` directory. `worker.json` contains worker access and refresh tokens.

## Manage workers in the web UI

Open **Workers** to:

- generate a 10-minute, one-time registration code;
- copy the recommended setup command;
- see status, last-seen time, and active capacity;
- rename a worker;
- disable or re-enable dispatch to a worker;
- delete a worker registration.

Deleting a server-side registration does not remove the service or local files from the worker host. Use `vulcanum worker self-delete` on the host for local removal.
