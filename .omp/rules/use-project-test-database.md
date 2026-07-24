---
name: use-project-test-database
description: "Use the test database configured in .env instead of creating or deleting local databases"
condition: "cargo\\s+sqlx\\s+database\\s+(?:create|drop)"
scope: "tool"
---

Use `DATABASE_URL` from the project `.env` exactly as configured. Do not synthesize another database URL or create, drop, start, or provision a local PostgreSQL database unless the user explicitly requests database lifecycle operations.