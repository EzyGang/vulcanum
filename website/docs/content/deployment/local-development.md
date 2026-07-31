# Local development

## Prerequisites

Install:

- Node.js 22 or newer;
- pnpm 11;
- stable Rust with Cargo, rustfmt, and Clippy;
- PostgreSQL 15 or newer;
- Redis.

A host worker also needs OpenCode and OMP installed. A Docker worker needs Docker. A Kata worker needs Linux, KVM, Docker, and Kata Containers.

## Install dependencies

Run from the repository root:

```bash
pnpm install
```

## Configure the control plane

Create `.env` in the repository root:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/vulcanum
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=replace-with-a-long-random-secret
INSTANCE_PASSWORD=replace-with-a-login-password
IS_SINGLE_USER=true
MODEL_PROVIDER_SECRET_KEY=replace-with-a-base64-encoded-32-byte-key
```

## Start the services

Apply migrations:

```bash
pnpm migrate-server-up
```

Start the API and task poller:

```bash
cargo run -p vulcanum-server --bin vulcanum-web
```

Start the dispatcher in a second terminal:

```bash
cargo run -p vulcanum-server --bin vulcanum-dispatcher
```

Start the frontend in a third terminal:

```bash
pnpm run dev --filter=@repo/frontend
```

Open `http://localhost:5173`. The API listens on `http://localhost:8000`.

## Configure local GitHub callbacks

Use these GitHub App URLs:

```text
Callback URL: http://localhost:8000/api/v1/github/callback
Webhook URL:  http://localhost:8000/api/v1/github/webhook
```

Add the GitHub environment variables from [Control-plane configuration](control-plane.md#github-app-variables). Restart `vulcanum-web` after you change `.env`.

GitHub must reach the webhook URL. A loopback URL works only for browser callbacks on the same machine. Use a secure tunnel or a reachable development host for webhook delivery.

## Configure the application

Use the web UI in this order:

1. Add a [task-tracker provider](../providers/task-trackers.md).
2. Connect the [GitHub App](../providers/github-app.md).
3. Add a [model provider](../providers/model-providers.md).
4. Select the team models and agent backend in [Settings](../web-ui/settings.md).
5. Add a provider project to Vulcanum.
6. Map its workflow columns with the CLI.
7. Select one or more repositories.
8. Enable project automation.
9. [Register a worker](../workers/setup.md).

Kaneo credentials belong in task-tracker provider settings. Do not set global `KANEO_INSTANCE` or `KANEO_API_KEY` variables.

## Build the documentation

The documentation is a separate uv project under `website/docs/`.

From the repository root, install the locked environment:

```bash
cd website/docs
uv sync
```

Start a live preview:

```bash
uv run zensical serve
```

Build the static site:

```bash
uv run zensical build
```

Zensical writes the output to `website/docs/site/` relative to the repository root.
