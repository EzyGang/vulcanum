FROM alpine:3.21

RUN apk add --no-cache \
    bash \
    curl \
    git \
    openssh-client

RUN curl -fsSL https://opencode.ai/install.sh | sh

WORKDIR /workdir
