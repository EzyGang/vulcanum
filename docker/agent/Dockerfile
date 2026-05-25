FROM alpine:3.21

RUN apk add --no-cache \
    bash \
    curl \
    git \
    openssh-client

RUN curl -fsSL https://opencode.ai/install | sh \
    && curl -fsSL https://raw.githubusercontent.com/EzyGang/kaneo-cli/main/install.sh | bash \
    && export PATH="$HOME/.local/bin:$PATH" \
    && kaneo install-skill --scope global --agent opencode

ENV PATH="/root/.local/bin:$PATH"

COPY docker/agent/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]

WORKDIR /workdir
