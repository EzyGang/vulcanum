#!/bin/bash
set -euo pipefail

kaneo login --instance "$PROVIDER_INSTANCE_URL" "$PROVIDER_API_KEY"
kaneo set --project "$EXTERNAL_PROJECT_ID" --workspace "$EXTERNAL_WORKSPACE_ID" --global
kaneo task status "$EXTERNAL_TASK_ID" in-progress

[ -f /workdir/setup.sh ] && bash /workdir/setup.sh || true

exec "$@"