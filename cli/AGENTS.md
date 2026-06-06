# AGENTS.md - vulcanum-cli

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

CLI for the Vulcanum orchestrator.

## Source Layout

| Path | Role |
| --- | --- |
| `src/main.rs` | CLI argument parsing, top-level subcommand dispatch |
| `src/console.rs` | Terminal output helpers (spinner step, info, warn) |
| `src/commands/mod.rs` | Command namespace registration |
| `src/commands/setup/mod.rs` | Worker setup orchestration flow |
| `src/commands/setup/backends/` | Isolation provider implementations (docker, kata, agent_image) |
| `src/commands/setup/connect.rs` | Worker registration and connection to an instance |
| `src/commands/setup/host.rs` | Host environment probes (KVM, sudo, binaries, paths) |
| `src/commands/setup/docker_daemon.rs` | Docker daemon.json I/O helpers |
| `src/commands/setup/systemd.rs` | systemd unit creation, systemctl wrapper, service lifecycle |
| `src/commands/setup/prompts.rs` | Interactive prompts and backend selection logic |

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-cli --bin vulcanum

# Or from this directory
cd cli
cargo run --bin vulcanum
```
