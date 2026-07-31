# Set up a worker

The setup command supports Linux and macOS. It installs a systemd service on Linux and a launchd service on macOS. The CLI does not provide worker service setup for Windows.

## 1. Install the worker binaries

Run on the worker host:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh | sh
```

The installer supports x86_64 and ARM64 Linux and macOS. It downloads the release archive and SHA-256 checksum, verifies the archive, and installs both binaries to `~/.local/bin` by default.

The host needs:

- `tar`;
- `awk`;
- `sed`;
- `curl` or `wget`;
- `sha256sum` or `shasum`.

To install a specific version or path:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/EzyGang/vulcanum/main/install.sh | \
  VULCANUM_VERSION=0.1.0 VULCANUM_INSTALL_DIR="$HOME/bin" sh
```

Keep `vulcanum` and `vulcanum-server` in the same directory.

## 2. Select an isolation mode

| CLI value | Stored value | Requirement | Security boundary |
| --- | --- | --- | --- |
| `none` | `host` | OpenCode and OMP installed on the host | None |
| `docker` | `docker` | Docker | Container |
| `kata` | `kata` | Linux, KVM, Docker, and Kata Containers | Lightweight virtual machine |

Non-interactive setup defaults to Docker when both `--instance` and `--code` are present.

Setup requires `sudo`. It uses non-interactive `sudo` after authorization to install dependencies and configure the worker service.

## 3. Generate a registration code

1. Sign in to the web UI.
2. Select the target team.
3. Open **Workers**.
4. Select **Generate code**.
5. Copy the recommended setup command.

The code expires after 10 minutes and can be used one time.

## 4. Run setup

Docker:

```bash
vulcanum worker setup \
  --instance https://<control-plane-host> \
  --code <registration-code> \
  --isolation docker
```

Kata:

```bash
vulcanum worker setup \
  --instance https://<control-plane-host> \
  --code <registration-code> \
  --isolation kata
```

Host:

```bash
vulcanum worker setup \
  --instance https://<control-plane-host> \
  --code <registration-code> \
  --isolation none
```

If you omit required values, setup prompts for them.

Setup performs these actions:

1. Checks administrator access.
2. Installs or checks the selected runtime dependencies.
3. Pulls the agent image for Docker or Kata.
4. Configures systemd or launchd.
5. Validates the environment.
6. Writes `~/.vulcanum/config.json`.
7. Registers the worker.
8. Writes `~/.vulcanum/worker.json`.
9. Enables and starts the worker service.

Use `--force` to register again when local worker state exists:

```bash
vulcanum worker setup --force
```

Without `--force`, setup verifies the existing connection. It registers again only if verification fails.

## 5. Verify the worker

Open **Workers** in the web UI. Confirm:

- the worker name matches the host name;
- status is `idle` when no job is active;
- last-seen time updates;
- capacity is correct.

On Linux, inspect logs:

```bash
journalctl -u vulcanum-worker
```

On macOS, the launchd setup writes logs to:

```text
/tmp/vulcanum-worker.log
/tmp/vulcanum-worker.err
```

## Remove a worker

Run on the worker host:

```bash
vulcanum worker self-delete
```

The command tries to unregister the worker, stop and remove its service, delete local worker state, and remove worker-owned runtime data.
