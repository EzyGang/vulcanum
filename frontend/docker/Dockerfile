FROM node:22-alpine AS builder
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@latest --activate
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY index.html vite.config.ts tsconfig.json ./
COPY src/ ./src/
COPY public/ ./public/
RUN pnpm build

FROM nginx:alpine AS runtime
COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/dist /app
EXPOSE 80
