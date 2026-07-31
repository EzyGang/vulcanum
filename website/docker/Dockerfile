FROM ghcr.io/astral-sh/uv:0.8.22-python3.13-bookworm-slim AS documentation

WORKDIR /workspace/website/docs

COPY website/docs/pyproject.toml website/docs/uv.lock ./
RUN uv sync --frozen

COPY website/docs/content ./content
COPY website/docs/zensical.toml ./
RUN uv run zensical build

FROM node:22-alpine AS landing

WORKDIR /workspace

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY website/landing/package.json ./website/landing/package.json
RUN corepack enable && pnpm install --frozen-lockfile --filter @repo/landing...

COPY frontend/public/logo.svg ./frontend/public/logo.svg
COPY website/landing ./website/landing
RUN pnpm --filter @repo/landing build

FROM nginx:1.29.3-alpine

COPY website/docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=landing /workspace/website/landing/dist /usr/share/nginx/html
COPY --from=documentation /workspace/website/docs/site /usr/share/nginx/html/docs

EXPOSE 80
