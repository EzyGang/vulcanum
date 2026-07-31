# Control-plane configuration

Set the same base environment for `vulcanum-web` and `vulcanum-dispatcher`. Both processes load the full application configuration.

## Required variables

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | PostgreSQL connection URL |
| `JWT_SECRET` | Secret used to sign application and worker tokens |
| `INSTANCE_PASSWORD` | Password for single-user login; the process also requires this value in multi-user mode |
| `MODEL_PROVIDER_SECRET_KEY` | Base64-encoded 32-byte key used to encrypt model-provider credentials |

Generate and store strong random values. Do not commit them.

`MODEL_PROVIDER_SECRET_KEY` must decode to exactly 32 bytes. Keep the same key for the life of the stored credentials. If you change it, Vulcanum cannot decrypt the existing model-provider credentials.

## General variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `MAX_CONNS` | `32` | Maximum PostgreSQL connections for the web process |
| `POLL_PERIOD_SECS` | `30` | Interval between task-tracker poll cycles |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `IS_SINGLE_USER` | `true` | `true` uses the instance password; `false` uses GitHub OAuth and teams |
| `STALE_WORKER_THRESHOLD_SECS` | `120` | Time after which a worker is stale for dispatch |
| `UNHEALTHY_THRESHOLD` | `3` | Consecutive failure threshold used for worker health |
| `STALLED_RUNNING_THRESHOLD_SECS` | `1800` | Time after which the dispatcher treats a running assignment as stalled |
| `DISPATCH_INTERVAL_SECS` | `15` | Dispatcher cycle interval; used only by `vulcanum-dispatcher` |

Use integer values for intervals and thresholds.

## GitHub App variables

These variables are optional when GitHub functions are not in use. Configure the complete set when you use repositories, pull requests, or GitHub login.

| Variable | Purpose |
| --- | --- |
| `GITHUB_APP_ID` | Numeric GitHub App ID |
| `GITHUB_APP_PRIVATE_KEY` | Base64 encoding of the complete private-key PEM file |
| `GITHUB_APP_SLUG` | Public slug of the GitHub App |
| `GITHUB_WEBHOOK_SECRET` | Shared secret that verifies GitHub webhook signatures |
| `GITHUB_CLIENT_ID` | OAuth client ID for GitHub user authorization |
| `GITHUB_CLIENT_SECRET` | OAuth client secret |
| `GITHUB_OAUTH_REDIRECT_URL` | Public callback URL for GitHub OAuth |

The redirect URL must exactly match a callback URL in the GitHub App or OAuth App.

For a public deployment, use these endpoint paths:

```text
https://<public-host>/api/v1/github/callback
https://<public-host>/api/v1/github/webhook
```

For App permissions and events, see [GitHub App](../providers/github-app.md).

## Example environment

```bash
DATABASE_URL=postgres://vulcanum:replace-me@postgres:5432/vulcanum
REDIS_URL=redis://redis:6379
JWT_SECRET=replace-with-a-long-random-secret
INSTANCE_PASSWORD=replace-with-a-login-password
IS_SINGLE_USER=true
MODEL_PROVIDER_SECRET_KEY=replace-with-a-base64-encoded-32-byte-key
```

Add the GitHub variables only when you configure that integration.

## Process commands

From a source checkout, start the web process:

```bash
cargo run -p vulcanum-server --bin vulcanum-web
```

Start the dispatcher in another process:

```bash
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

The API binds to `0.0.0.0:8000`. The application does not provide TLS. Terminate TLS before traffic reaches the API.

## Authentication modes

### Single-user mode

Set:

```bash
IS_SINGLE_USER=true
INSTANCE_PASSWORD=<password>
```

Users log in with the instance password. Vulcanum uses the default team for the instance.

### Multi-user mode

Set:

```bash
IS_SINGLE_USER=false
```

Also set the three GitHub OAuth variables. Users log in with GitHub. Team membership controls access to team data.

Application authentication does not isolate untrusted worker jobs. Select a worker isolation mode separately.
