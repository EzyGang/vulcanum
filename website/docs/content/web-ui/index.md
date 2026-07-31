# Web UI

The web UI is the main control interface. It is scoped to the selected team.

## Pages

| Page | Purpose |
| --- | --- |
| Dashboard | Select a provider project, view its task board, create tasks, and configure project automation |
| Runs | Inspect implementation and pull-request-review runs |
| Workers | Generate registration codes and manage registered workers |
| Teams | Create teams, inspect members, and create invite links |
| Settings | Configure team defaults and external integrations |

## Select a team

Use the team selector in the application navigation. The selected team controls the projects, runs, workers, and settings that the UI shows.

A multi-user account can belong to more than one team. Access depends on the team membership and role. A single-user instance uses its default team.

## Recommended configuration order

1. Open **Settings > Task trackers** and add Kaneo.
2. Open **Settings > GitHub app** and connect an installation.
3. Open **Settings > Model providers** and connect at least one provider.
4. Open **Settings > Model selection** and select implementation models.
5. Open **Settings > Agent defaults** and review the prompts and limits.
6. Use the board picker to add a provider project.
7. Map the workflow columns with the CLI.
8. Open the board settings and select repositories.
9. Enable automation on the board.
10. Open **Workers** and register a worker.

The board can read and edit provider tasks before automation is enabled. Enable automation only after you map the workflow columns and select the repositories that workers need.

## Authentication

Single-user deployments use the instance password. Multi-user deployments use GitHub OAuth.

The browser session calls only the control-plane API. Provider credentials are sent to the server. They are not saved in browser-local application state.
