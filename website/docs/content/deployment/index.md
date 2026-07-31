# Deployment

A complete Vulcanum deployment needs these services:

| Service | Required | Purpose |
| --- | --- | --- |
| `vulcanum-web` | Yes | HTTP API, task polling, authentication, and provider callbacks |
| `vulcanum-dispatcher` | Yes | Assignment of pending runs to workers |
| Frontend | Yes for the web UI | Static Preact application and API proxy |
| PostgreSQL 15 or newer | Yes | Durable control-plane data |
| Redis | Yes | Dispatch, cancellation, and short-lived coordination data |
| One or more workers | Yes to run jobs | Agent execution |

## Deployment sequence

1. Start PostgreSQL and Redis.
2. Set the [control-plane environment variables](control-plane.md).
3. Start `vulcanum-web`.
4. Start `vulcanum-dispatcher` with the same environment.
5. Start the frontend.
6. Put an HTTPS reverse proxy or load balancer in front of the frontend and API.
7. Configure the public GitHub callback and webhook URLs if you use GitHub.
8. Connect providers in the web UI.
9. [Register a worker](../workers/setup.md).

Both server processes run PostgreSQL migrations at startup. You can also apply migrations before you start the processes:

```bash
pnpm migrate-server-up
```

## Network paths

Allow these required paths:

- Browser to frontend: HTTP or HTTPS.
- Frontend to `vulcanum-web`: `/api/*` on API port `8000`.
- `vulcanum-web` and `vulcanum-dispatcher` to PostgreSQL.
- `vulcanum-web` and `vulcanum-dispatcher` to Redis.
- Workers to the public `/api/v1` API.
- `vulcanum-web` to configured provider APIs.
- Workers to GitHub and to any package or source hosts that jobs use.

Do not expose PostgreSQL or Redis to workers or browsers.

## Container images

The repository contains these Dockerfiles:

```text
server/docker/Dockerfile
frontend/docker/Dockerfile
docker/agent/Dockerfile
website/docker/Dockerfile
```

Build the control-plane and frontend images from the repository root:

```bash
docker build -f server/docker/Dockerfile -t vulcanum-control-plane .
docker build -f frontend/docker/Dockerfile -t vulcanum-frontend .
```

The control-plane image contains `vulcanum-web` and `vulcanum-dispatcher`. Its default entry point starts `vulcanum-web`. Override the command for the separate dispatcher container.

The frontend image listens on port `80`. Its Nginx configuration sends `/api` requests to `http://server:8000`. Name the API service `server`, or provide an equivalent proxy configuration.

The website image serves the public landing page at `/` and the Zensical documentation at `/docs/`. It does not start the Vulcanum server and does not need control-plane environment variables.

The repository does not include a production Compose or Kubernetes definition. Supply PostgreSQL, Redis, secrets, storage policy, health checks, and TLS in your deployment platform.

## Release scope

Published release archives contain the worker-side `vulcanum` and `vulcanum-server` binaries. Release tags also publish multi-platform control-plane, frontend, and website images:

```text
ghcr.io/ezygang/vulcanum/backend:<release-tag>
ghcr.io/ezygang/vulcanum/frontend:<release-tag>
ghcr.io/ezygang/vulcanum/website:<release-tag>
```

Use the exact release tag, such as `v0.1.2` or `v0.1.2-alpha.1`. The release workflow does not publish a mutable `latest` tag.
