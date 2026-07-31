# Providers

Providers connect Vulcanum to external systems. Provider settings belong to a team.

## Provider types

| Type | Current support | Purpose |
| --- | --- | --- |
| Task tracker | Kaneo | Projects, columns, tasks, labels, and comments |
| Model provider | Provider catalog from models.dev | Model discovery and runtime credentials |
| Source control | GitHub App | Repository access, pull requests, webhooks, and user authorization |

Agent runtimes are worker-side providers. See [Workers](../workers/index.md) for OpenCode and OMP RPC.

## Configuration order

1. Add a [task tracker](task-trackers.md).
2. Connect the [GitHub App](github-app.md).
3. Add one or more [model providers](model-providers.md).
4. Select team models in **Settings > Model selection**.
5. Add a provider project.
6. Select project repositories.

## Credential handling

- Task-tracker API keys are stored by the control plane.
- Model-provider credentials are encrypted in PostgreSQL with `MODEL_PROVIDER_SECRET_KEY`.
- GitHub App private keys and OAuth secrets come from the control-plane environment.
- GitHub repository access uses short-lived installation tokens.
- The server decrypts the model credentials that a job needs and sends them to the authenticated worker.

Use HTTPS for all remote control-plane and provider traffic. Restrict access to the control-plane environment and database.
