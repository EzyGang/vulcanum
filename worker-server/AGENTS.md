# AGENTS.md - vulcanum-worker-server

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Host Machine Server for the Vulcanum orchestrator.

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-worker-server --bin vulcanum-server

# Or from this directory
cd worker-server
cargo run --bin vulcanum-server
```
