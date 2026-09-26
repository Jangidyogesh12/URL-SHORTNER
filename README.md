# Rust + Next.js Monorepo

A full-stack monorepo wiring a **Next.js** frontend to a **Rust (Axum)** backend, backed by **PostgreSQL**, fronted by **Nginx**, and glued together with shared, auto-generated TypeScript types.

Both package managers live side by side in one repo:

- **npm workspaces** manage the JavaScript/TypeScript side (`apps/web`, `packages/*`).
- **Cargo workspace** manages the Rust side (`apps/api`, `crates/*`).

---

## Architecture

```
                         ┌─────────────────────────────┐
          port 80        │            nginx            │
  browser ──────────────▶│  /        -> web:3000       │
                         │  /api/*   -> api:8080       │
                         │  /health  -> api:8080       │
                         └──────────────┬──────────────┘
                                        │  backend network
                         ┌──────────────┴──────────────┐
                         │                             │
                    ┌────▼────┐                   ┌────▼────┐
                    │   web   │                   │   api   │
                    │ Next.js │                   │  Axum   │
                    │  :3000  │                   │  :8080  │
                    └─────────┘                   └────┬────┘
                                                       │
                                                  ┌────▼────┐
                                                  │   db    │
                                                  │ Postgres│
                                                  │  :5432  │
                                                  └─────────┘
```

Only Nginx (port 80) is exposed for application traffic; Postgres is additionally published on `5432` so local tooling (`sqlx`, `psql`, `cargo run`) can reach it. The browser talks to one origin (`http://localhost`), so there is **no CORS configuration** needed in the Rust API.

---

## What each component does

| Path | Component | Responsibility |
| --- | --- | --- |
| `apps/web` | **Next.js frontend** | UI, routing, React server/client components. Fetches the API over relative `/api/*` URLs and consumes shared TS types. Built as a `standalone` output for a small Docker image. |
| `apps/api` | **Rust API (Axum)** | HTTP backend. Exposes `/health`, `/api/hello`, `/api/users`. Owns business logic and talks to Postgres via `sqlx`. |
| `crates/shared` | **Rust shared library** | Single source of truth for data transfer objects (`HelloResponse`, `HealthResponse`, `User`). Derives `serde` (JSON) **and** `ts-rs` (TypeScript). Also hosts the `export` binary that emits the TS types. |
| `packages/shared-types` | **Generated TS package** | Holds the TypeScript types emitted from `crates/shared`. Consumed directly by the frontend (`import type { User } from "shared-types"`). |
| `apps/api/migrations` | **Database migrations** | Versioned sqlx migrations that own the Postgres schema. Applied with `npm run migrate:up`, reverted with `npm run migrate:down`. |
| `nginx.conf` | **Reverse proxy** | Routes `/` to the Next.js app, `/api/*` and `/health` to the Rust API, so everything is served from one origin on port 80. |
| `docker-compose.yaml` | **Orchestration** | Wires `nginx`, `web`, `api`, and `db` together with networks, ports, env vars, healthchecks, and volumes. |
| `Cargo.toml` | **Cargo workspace root** | Declares Rust members (`apps/api`, `crates/shared`) and shared dependency versions. |
| `package.json` | **npm workspace root** | Declares JS members (`apps/web`, `packages/*`) and top-level convenience scripts. |

---

## Shared types pipeline

Rust structs are the source of truth. The TypeScript types are **generated**, never hand-written.

```
crates/shared/src/lib.rs          #[derive(Serialize, TS)]
        │
        │  npm run generate-types   (cargo run -p shared --bin export)
        ▼
packages/shared-types/src/generated/*.ts
        │
        │  import type { User } from "shared-types"
        ▼
apps/web  ─────────────────────────────────────▶  apps/api
```

`i64` fields are exported as TypeScript `number` (configured in `.cargo/config.toml`). After changing any DTO in `crates/shared`, re-run `npm run generate-types`.

---

## Project structure

```
.
├── apps/
│   ├── api/                    # Rust Axum backend
│   │   ├── src/main.rs         # server bootstrap, router, state
│   │   ├── src/routes.rs       # /health, /api/hello, /api/users handlers
│   │   ├── migrations/         # sqlx migrations (schema source of truth)
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   └── web/                    # Next.js frontend
│       ├── src/app/            # App Router pages/layout
│       ├── src/lib/api.ts      # typed API client
│       ├── next.config.ts      # standalone output, rewrites, shared-types
│       ├── package.json
│       └── Dockerfile
├── crates/
│   └── shared/                 # DTOs + ts-rs export binary
├── packages/
│   └── shared-types/           # generated TypeScript types
├── nginx.conf                  # reverse proxy config
├── docker-compose.yaml         # service orchestration
├── Cargo.toml                  # Cargo workspace root
├── package.json                # npm workspace root
└── .env                        # environment variables
```

---

## Prerequisites

- Node.js 20+ and npm
- Rust (stable) + Cargo
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres,rustls`)
- Docker + Docker Compose

---

## Running it

### Option A — Docker (full stack)

```bash
npm run docker:up      # docker compose up --build
npm run migrate:up     # apply migrations (once you add them)
```

Then open:

- http://localhost/            — Next.js app
- http://localhost/api/hello   — Rust endpoint
- http://localhost/api/users   — rows from Postgres (empty until data exists)
- http://localhost/health      — API health check

Stop with `npm run docker:down`.

### Option B — Local development

Run the database (and optionally Nginx) via Docker, then run each app with hot reload:

```bash
# terminal 1 — Postgres
docker compose up db

# terminal 2 — Rust API (http://localhost:8080)
npm run dev:api

# terminal 3 — Next.js (http://localhost:3000)
npm run dev:web
```

In dev, Next.js rewrites `/api/*` to `http://localhost:8080` (see `next.config.ts`), so no Nginx is required.

---

## Root commands

| Command | Description |
| --- | --- |
| `npm run dev:web` | Start the Next.js dev server |
| `npm run dev:api` | Run the Rust API with Cargo |
| `npm run generate-types` | Regenerate TypeScript types from Rust DTOs |
| `npm run migrate:up` | Apply all pending sqlx migrations |
| `npm run migrate:down` | Revert the most recent migration |
| `npm run migrate:info` | List applied / pending migrations |
| `npm run db:reset` | Wipe the database to zero via raw `psql` (drops the `public` schema) |
| `npm run build` | Generate types, then build web and api |
| `npm run check` | `cargo check --workspace` |
| `npm run docker:up` | Build and start the full stack |
| `npm run docker:down` | Stop the stack |

Individual workspaces can be targeted too, e.g. `npm run build --workspace=web`.

---

## Migrations

The Postgres schema is owned by the Rust API and versioned with **sqlx migrations** in `apps/api/migrations/`. Each migration is a reversible pair (`*.up.sql` / `*.down.sql`), tracked in the `_sqlx_migrations` table.

```bash
npm run migrate:up              # apply all pending migrations
npm run migrate:down            # revert the most recent migration
npm run migrate:info            # list applied / pending migrations
npm run migrate:add -- <name>   # create a new reversible migration
```

Migrations are **not** applied automatically when the API starts — run them explicitly with `npm run migrate:up`.

To reset the database back to zero, use the raw-Postgres reset instead of reverting migrations. It drops and recreates the `public` schema, removing all tables and data (including `_sqlx_migrations`):

```bash
npm run db:reset
```

---

## API endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/health` | Liveness check — `{ "status": "ok" }` |
| `GET` | `/api/hello` | Demo response from Rust (`HelloResponse`) |
| `GET` | `/api/users` | Reads users from Postgres (`User[]`) |

---

## Environment variables

Defined in `.env` (see `.env.example`):

| Variable | Purpose |
| --- | --- |
| `POSTGRES_USER` | Postgres user |
| `POSTGRES_PASSWORD` | Postgres password |
| `POSTGRES_DB` | Postgres database name |
| `DATABASE_URL` | Connection string used by the Rust API for local runs and by the `sqlx` CLI (host view: `localhost`). The `api` container gets its own `db:5432` URL from `docker-compose.yaml`. |
| `API_PORT` | Port the Rust API listens on (default `8080`) |

---

## Notes

- The Postgres schema lives in `apps/api/migrations/` and is **not** applied automatically; run `npm run migrate:up` after starting the database.
- Generated files in `packages/shared-types/src/generated/` are committed so the frontend and Docker builds work without running Rust first. Regenerate them with `npm run generate-types` after DTO changes.
- The Next.js standalone Docker image expects those generated files to exist at build time.
- Because Nginx serves everything from a single origin, the browser never makes cross-origin requests.
