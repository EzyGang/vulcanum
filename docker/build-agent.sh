#!/usr/bin/env bash
set -euo pipefail

IMAGE="ghcr.io/ezygang/vulcanum/agent:latest"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Building $IMAGE..."
docker build -t "$IMAGE" "$SCRIPT_DIR"

if [ "${1:-}" = "--push" ]; then
    echo "Pushing $IMAGE..."
    docker push "$IMAGE"
fi

echo "Done."
