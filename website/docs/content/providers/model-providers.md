# Model providers

A model-provider connection supplies models and credentials to agent jobs. The provider and model catalog comes from models.dev.

## Connect with an API key

1. Select the correct team.
2. Open **Settings > Model providers**.
3. Select **Add model provider**.
4. Select a provider.
5. Enter a display name.
6. Enter the credential fields shown from the catalog.
7. Save the connection.

The required credential field names depend on the selected provider. For example, an Anthropic connection can request `ANTHROPIC_API_KEY`.

The table shows credential field names. It does not show credential values.

## Connect OpenAI with ChatGPT

OpenAI also supports a ChatGPT Pro/Plus device login.

1. Select **OpenAI**.
2. Set **Auth Method** to **ChatGPT Pro/Plus**.
3. Save the form to start the device flow.
4. Open the displayed verification URL.
5. Enter the displayed user code.
6. Approve access.
7. Wait for Vulcanum to confirm the connection.

The tokens are encrypted and are not shown in the UI after connection.

The CLI provides the same flow:

```bash
vulcanum settings model-providers connect-openai [--name <NAME>] [--no-browser]
```

`--no-browser` prints the verification handoff without opening a browser.

## Configure credentials with the CLI

Add an API-key provider:

```bash
vulcanum settings model-providers add <PROVIDER_KEY> [--name <NAME>]
```

For non-interactive use, send a non-empty JSON object:

```bash
printf '%s' '{"ANTHROPIC_API_KEY":"value"}' | \
  vulcanum settings model-providers add anthropic \
  --name "Production Anthropic" \
  --auth api-key \
  --credentials-stdin
```

Field names and values must be non-empty strings.

Use `--auth none` for a provider that needs no credentials:

```bash
vulcanum settings model-providers add <PROVIDER_KEY> --auth none
```

## Select models

A connection does not select a model.

Open **Settings > Model selection**. Select the provider and model for the implementation primary and small-model pairs. You can also set optional review pairs.

The agent backend is selected separately. OpenCode and OMP RPC can receive different provider-specific runtime configuration from the same team settings.

## Update a connection

The web UI can change the display name and credentials for an existing connection. The provider key cannot be changed during edit.

With the CLI:

```bash
vulcanum settings model-providers update <UUID> \
  [--name <NAME>] \
  [--auth <api-key|none>] \
  [--credentials-stdin | --prompt-credentials]
```

Credentials remain unchanged unless you select a credential mode. To replace credentials on a `none` or device-OAuth connection, set `--auth api-key`.

## Delete a connection

Delete a connection in the web UI, or run:

```bash
vulcanum settings model-providers remove <UUID>
```

Deletion does not clear team model selections. Clear or replace each model pair that refers to the connection.

## Encryption and delivery

The control plane encrypts model credentials with AES-256-GCM. `MODEL_PROVIDER_SECRET_KEY` must decode to 32 bytes and must not change while encrypted credentials exist.

For each job, the server decrypts only the selected provider configuration and sends it to the authenticated worker. The job runtime receives the provider-specific environment or auth data.
