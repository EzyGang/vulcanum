FROM alpine:3.21

RUN apk add --no-cache \
    bash \
    curl \
    git \
    openssh-client

ENV PATH="/root/.local/bin:$HOME/.local/bin:$PATH"

RUN curl -fsSL https://opencode.ai/install | sh
RUN curl -fsSL https://raw.githubusercontent.com/EzyGang/kaneo-cli/main/install.sh | bash
RUN kaneo install-skill --scope global --agent opencode


COPY docker/agent/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]

WORKDIR /workdir
