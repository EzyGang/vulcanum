#!/bin/bash
set -euo pipefail

kaneo login --instance "$KANEO_INSTANCE" "$KANEO_API_KEY"
kaneo set --project "$KANEO_PROJECT_ID" --workspace "$KANEO_WORKSPACE_ID" --global
kaneo task status "$KANEO_TASK_ID" in-progress

exec "$@"