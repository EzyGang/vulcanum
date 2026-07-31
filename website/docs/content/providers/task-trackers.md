# Task trackers

Vulcanum currently supports Kaneo as its task-tracker provider.

## Before you start

Get these values from the Kaneo instance:

- the instance URL;
- an API key that can access the required workspaces, projects, tasks, columns, labels, and comments.

Use a service account when your Kaneo deployment supports one. Give the key only the access that Vulcanum needs.

## Connect in the web UI

1. Select the correct team.
2. Open **Settings > Task trackers**.
3. Select **Add task tracker**.
4. Select **Kaneo**.
5. Enter a clear connection name.
6. Enter the Kaneo instance URL.
7. Enter the API key.
8. Select **Create Provider**.

All fields are required.

Kaneo settings are stored per connection. Do not set global `KANEO_INSTANCE` or `KANEO_API_KEY` environment variables.

## Connect with the CLI

```bash
vulcanum settings task-trackers add \
  --name <NAME> \
  --instance-url <URL>
```

The CLI prompts for the API key with hidden input. For non-interactive use, pass one JSON object through standard input:

```bash
printf '%s' '{"api_key":"value"}' | \
  vulcanum settings task-trackers add \
  --name <NAME> \
  --instance-url <URL> \
  --credentials-stdin
```

The CLI reads credentials only after it authenticates and resolves the team.

## Add provider projects

After the connection succeeds, use the board picker or the CLI to add a provider project. A Vulcanum project stores:

- the task-tracker connection ID;
- the external workspace and project IDs;
- workflow column mappings;
- automation state;
- selected GitHub repositories;
- project overrides.

A new project starts with automation disabled.

## Edit a connection

In the web UI, select the edit action on the provider row. Save the new name, URL, or API key.

With the CLI:

```bash
vulcanum settings task-trackers update <UUID> \
  [--name <NAME>] \
  [--instance-url <URL>] \
  [--credentials-stdin | --prompt-credentials]
```

The CLI keeps credentials unchanged unless you select a credential input mode.

## Delete a connection

Use the delete action in the web UI, or run:

```bash
vulcanum settings task-trackers remove <UUID>
```

Deletion is permanent. Projects that refer to the deleted connection can no longer read or update the provider. Remove or replace those projects before you delete an active connection.

## Automation behavior

For an enabled project, the control plane polls Kaneo at `POLL_PERIOD_SECS`. A task in the mapped pickup column can create a pending implementation run. Vulcanum moves the task through the configured in-progress, in-review, and done columns as work completes.

Vulcanum can also create tasks, edit tasks, move tasks, manage labels, and add run results as comments.
