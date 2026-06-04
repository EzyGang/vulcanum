FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    curl \
    git \
    less \
    openssh-client \
    ca-certificates \
    gnupg \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    make \
    python3 \
    python3-pip \
    postgresql-client \
    redis-tools \
    sqlite3 \
    && curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
       gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
       > /etc/apt/sources.list.d/github-cli.list \
    && curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get update && apt-get install -y --no-install-recommends \
       gh \
       nodejs \
    && corepack enable \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.opencode/bin:/root/.local/bin:$PATH"

RUN curl -fsSL https://opencode.ai/install | bash -s -- --no-modify-path \
    && curl -fsSL https://raw.githubusercontent.com/EzyGang/kaneo-cli/main/install.sh | bash \
    && curl https://mise.run | sh \
    && kaneo install-skill --scope global --agent opencode \
    && test -x /root/.opencode/bin/opencode || (echo "ERROR: opencode binary not found at /root/.opencode/bin/opencode" && exit 1)

COPY docker/agent/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /workdir

RUN mkdir -p /workdir/home \
    && cd /workdir/home \
    && kaneo install-skill --scope local --agent opencode

ENTRYPOINT ["/entrypoint.sh"]