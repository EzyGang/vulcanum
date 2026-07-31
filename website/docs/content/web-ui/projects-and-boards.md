# Projects and task boards

A Vulcanum project links one provider project to automation settings and zero or more GitHub repositories.

## Add a project

Before you add a project, connect a task tracker. Then use the board picker in the application navigation to add a provider project.

A new project starts with automation disabled. This prevents work from starting before you configure its workflow and repositories.

You can also add a project with the [CLI](../cli/reference.md#add-a-project).

## Map workflow columns

Each automated project uses four provider columns:

| Role | Use |
| --- | --- |
| Pickup | A task in this column can create an implementation run |
| In progress | The worker acknowledged the task and work is active |
| In review | Implementation is complete and pull requests can be reviewed |
| Done | All linked pull requests are closed or merged |

Set the mappings with the CLI:

```bash
vulcanum projects columns set <PROJECT_ID> \
  --pickup <COLUMN> \
  --in-progress <COLUMN> \
  --in-review <COLUMN> \
  --done <COLUMN>
```

`COLUMN` can be the provider column name, slug, or ID. Vulcanum validates the value and stores the canonical slug.

## Configure board settings

Open a board. Select the settings button.

### Repositories

Select all repositories that jobs for this project need. The selected repositories are passed to workers for cloning.

Automation can be enabled with no repository selected, but the UI shows a warning. A code job cannot clone or update a repository that is not selected.

### Project overrides

A project can override:

- implementation prompt template;
- `Agents.md` instructions;
- maximum in-progress tasks.

Leave a field empty to use the team default. Select **Reset to team default** to clear a prompt override.

### Review automation

A project can inherit, enable, or disable review automation. It can also override:

- review follow-up passes, from 1 to 10;
- review prompt template.

Leave an override empty to use the team default.

## Enable automation

Use the automation control on the board. Enable automation only when:

- pickup, in-progress, in-review, and done columns are mapped;
- all required repositories are selected;
- model providers and team models are configured;
- a compatible worker is available.

Automation and workflow mappings are independent. Disabling automation keeps the mappings.

## Use the board

The board is a proxy for the connected task tracker. You can:

- view tasks by provider column;
- create a task;
- open task details;
- edit task content;
- move a task to another column;
- view labels and run state;
- inspect project and task token usage.

Vulcanum keeps one managed automation label active on each automated task. Manual provider labels remain unchanged.

## Automation result

A successful implementation run adds its result to the source task and moves the task to review. If review automation is enabled, Vulcanum creates separate review runs for the submitted pull requests.

A failed or blocked run does not advance the task. Fix the cause before you start new work.
