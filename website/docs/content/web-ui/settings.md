# Settings

Open **Settings** and select a category from the settings index. All values apply to the selected team.

## Agent defaults

Use **Team > Agent defaults** to configure shared project behavior.

| Field | Effect |
| --- | --- |
| **Prompt Template** | Base implementation prompt for the team |
| **Agents.md** | Repository instructions that are added to jobs |
| **Project In-progress Limit** | Maximum number of automated tasks that one project can have in progress |
| **Enable PR Review Automation** | Starts review work for implementation pull requests |
| **Review Follow-up Passes** | Maximum number of review follow-up passes |
| **Review Prompt Template** | Base prompt for pull-request reviews |

Select **Reset to system default** to remove a team prompt override. Save the form after you change a value.

A project can override the prompt, `Agents.md`, in-progress limit, and review settings. An empty project override inherits the team value.

## Model selection

Use **Team > Model selection** to select runtime models.

### Implementation runtime

Select:

1. **Agent Backend**: OpenCode or OMP RPC.
2. **Primary provider** and **Primary model**.
3. **Small model provider** and **Small model** when the backend uses a small model.

Only connected model providers are available. The model list comes from the selected provider catalog.

### Review runtime

Review model pairs are optional. If a review pair is empty, Vulcanum uses the corresponding implementation pair.

Select **Clear override** to restore inheritance for a review pair.

## Task trackers

Use **Integrations > Task trackers** to add, edit, or delete Kaneo connections.

The form requires:

- provider type;
- display name;
- instance URL;
- API key.

Deleting a connection is permanent. Projects that use the connection cannot access their provider after deletion.

See [Task trackers](../providers/task-trackers.md).

## Model providers

Use **Integrations > Model providers** to connect credentials for agent models.

1. Select a provider from the catalog.
2. Set a display name.
3. Enter the credential fields shown for that provider.
4. Save the connection.

For OpenAI, select **API Key** or **ChatGPT Pro/Plus**. The ChatGPT option starts a device login. Open the verification URL and enter the displayed user code. Vulcanum stores the resulting tokens encrypted and does not show them again.

See [Model providers](../providers/model-providers.md).

## GitHub App

Use **Integrations > GitHub app** to:

- connect one or more GitHub App installations;
- refresh installation status;
- disconnect an installation;
- link or unlink the GitHub identity used to authorize review commands.

The control-plane environment must contain the GitHub App settings before this panel can complete a connection.

See [GitHub App](../providers/github-app.md).

## Save and inheritance rules

- Team settings apply to all team projects unless a project has an override.
- Project settings do not change the team defaults.
- Review model settings inherit from implementation model settings when the review pair is empty.
- Project prompt and review fields inherit from the team when their override is empty.
- Integration settings are team-scoped. Another team must create its own connections.
