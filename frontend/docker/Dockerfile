FROM node:22-alpine AS builder
WORKDIR /app

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY pnpm-workspace.yaml pnpm-lock.yaml package.json ./
COPY frontend/ frontend/

WORKDIR /app/frontend
RUN pnpm install --frozen-lockfile
RUN pnpm build

FROM nginx:alpine AS runtime
COPY frontend/docker/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/frontend/dist /app
EXPOSE 80
