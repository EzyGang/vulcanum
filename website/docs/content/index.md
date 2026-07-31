# Vulcanum documentation

Vulcanum dispatches work from a task tracker to agent workers. It keeps task state, run state, provider settings, and worker capacity in one control plane.

Use this documentation to deploy and operate Vulcanum.

## Start here

- [Architecture](architecture.md): Learn how the control plane, workers, and providers connect.
- [Deployment](deployment/index.md): Run the control plane and its data services.
- [Web UI](web-ui/index.md): Configure teams, projects, models, and integrations.
- [Providers](providers/index.md): Connect Kaneo, model providers, and GitHub.
- [Workers](workers/index.md): Install, register, and operate worker hosts.
- [CLI](cli/index.md): Log in and manage Vulcanum from a terminal.

## Current integration limits

Vulcanum currently supports these integrations:

| Domain | Supported integration |
| --- | --- |
| Task tracker | Kaneo |
| Source control and pull requests | GitHub App |
| Agent runtime | OpenCode and OMP RPC |
| Worker isolation | Host, Docker, and Kata Containers |

Do not configure a planned integration as if it is available. The current release does not include other task trackers, source-control systems, or agent runtimes.
