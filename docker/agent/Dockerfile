FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    curl \
    git \
    less \
    openssh-client \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.opencode/bin:/root/.local/bin:$PATH"

RUN curl -fsSL https://opencode.ai/install | bash -s -- --no-modify-path \
    && curl -fsSL https://raw.githubusercontent.com/EzyGang/kaneo-cli/main/install.sh | bash \
    && kaneo install-skill --scope global --agent opencode \
    && test -x /root/.opencode/bin/opencode || (echo "ERROR: opencode binary not found at /root/.opencode/bin/opencode" && exit 1)

COPY docker/agent/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
WORKDIR /workdir
