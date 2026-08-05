# GitHub App

The GitHub App provides:

- repository discovery;
- short-lived clone credentials;
- pull-request tracking;
- pull-request close and merge events;
- review and implementation commands in pull-request comments;
- GitHub user authorization.

## Create the GitHub App

Configure these URLs for a public control-plane host:

```text
Callback URL: https://<host>/api/v1/github/callback
Webhook URL:  https://<host>/api/v1/github/webhook
```

Enable **Request user authorization (OAuth) during installation**. The optional Setup URL can use the callback URL.

Subscribe to these webhook events:

- **Pull request**;
- **Issue comment**.

Grant these repository permissions:

| Permission | Access | Reason |
| --- | --- | --- |
| Contents | Read and write | Clone repositories and let jobs update code |
| Pull requests | Read and write | Track and work with pull requests |
| Issues | Read and write | Receive pull-request comments and post command responses |
| Workflows | Read and write | Let jobs update repository workflows |

GitHub models pull-request timeline comments as issue comments. The App needs Issues access for mention commands.

## Configure the control plane

Set:

```bash
GITHUB_APP_ID=<numeric-app-id>
GITHUB_APP_PRIVATE_KEY=<base64-encoded-complete-pem>
GITHUB_APP_SLUG=<app-slug>
GITHUB_WEBHOOK_SECRET=<webhook-secret>
GITHUB_CLIENT_ID=<oauth-client-id>
GITHUB_CLIENT_SECRET=<oauth-client-secret>
GITHUB_OAUTH_REDIRECT_URL=https://<host>/api/v1/github/callback
```

The OAuth credentials can belong to the same GitHub App or to a separate OAuth App. The redirect URL must exactly match a configured callback URL.

Restart `vulcanum-web` after you change these variables.

## Connect an installation

1. Select the correct team.
2. Open **Settings > GitHub app**.
3. Select **Connect**.
4. Select the GitHub account and repositories.
5. Complete the installation in GitHub.
6. Return to Vulcanum and refresh the status if necessary.

You can connect another account from the same panel. Disconnect only the installation that you no longer need.

The CLI can start a connection:

```bash
vulcanum settings github connect [--no-browser]
```

The command reports that the browser flow started. The callback stores the installation.

## Link a review identity

In single-user mode, use **PR review identity** in the GitHub settings panel. Link the GitHub account that can authorize pull-request mention commands.

In multi-user mode, each commenter must sign in with GitHub and belong to the team that owns the project.

## Pull-request mention commands

An authorized team member can add a new comment to an open pull request:

```text
@app-slug review [project:<project-config-uuid>]
@app-slug implement [project:<project-config-uuid>] [ticket:<external-task-ref>] <request>
```

Commands and App mentions are case-insensitive. Vulcanum ignores edited comments, deleted comments, comments that are not on pull requests, and comments from the App itself.

### Select a project

If one repository belongs to more than one eligible Vulcanum project, put the project selector immediately after the command:

```text
@vulcanum-app review project:0d915a91-f314-4c1e-a2b6-dae140ca16d2
```

If the selector is missing or invalid, Vulcanum replies with exact project-specific commands.

### Select a ticket

An implementation request can reuse a ticket that is already mapped to the pull request. If more than one ticket matches after project selection, add the ticket selector after the optional project selector:

```text
@vulcanum-app implement project:<PROJECT_UUID> ticket:<TASK_REF> handle the retry case
```

Vulcanum does not guess when more than one project or ticket matches.

The implementation request is required and can use more than one line. Vulcanum preserves the request as run context. It rejects a follow-up when the selected ticket already has an active implementation run.

## Credential path

The control plane gets a short-lived GitHub installation token for a job. The worker exposes the token through Git and `gh` credential helpers. It does not put the direct token in the ordinary agent environment.
