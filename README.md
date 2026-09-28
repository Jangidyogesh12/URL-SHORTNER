# Link Shortener

A full-stack, per-user URL shortener built with a **Next.js** frontend and a **Rust (Axum)** backend. It uses **PostgreSQL** as the authoritative store, **Redis** as a short-URL cache, **Nginx** as a single-origin reverse proxy and load balancer, and shared Rust-generated TypeScript contracts.

The API runs as three replicas (`api1`, `api2`, `api3`) behind Nginx round-robin load balancing for local load-distribution testing.

Authenticated users can:

- Register and sign in.
- Create short links for long URLs.
- Browse, search, filter, edit, delete, copy, and open their links.
- Give each edited link a fixed lifetime—or leave it without expiry.
- Share public links in the form `/{short_code}`.

Both package managers live side by side in one repo:

- **npm workspaces** manage the JavaScript/TypeScript side (`apps/web`, `packages/*`).
- **Cargo workspace** manages the Rust side (`apps/api`, `crates/*`).

---

## Architecture

```text
                          ┌────────────────────────────────────────┐
           port 80        │                  nginx                 │
   browser ──────────────▶│  /                  -> web:3000        │
                          │  /api/*             -> api:8080        │
                          │  /<8-char-code>     -> web:3000        │
                          └───────────────────┬────────────────────┘
                                              │ backend network
                          ┌───────────────────┴────────────────────┐
                          │                                        │
                     ┌────▼────┐                              ┌────▼────┐
                     │   web   │                              │   api   │
                     │ Next.js │                              │  Axum   │
                     │  :3000  │                              │  :8080  │
                     └────┬────┘                              └───┬─────┘ 
                          │ fallback redirect lookups             │
                          └──────────────────────────────────────▶│
                                                                  │
                     ┌────────────────────────────────────────────▼───────┐
                     │                          db                        │
                     │                       Postgres                     │
                     │                         :5432                      │
                     └────────────────────────────────────────────────────┘

                     ┌─────────────────────────────────────────┐
                     │                 redis                   │
                     │           short-URL cache               │
                     │                 :6379                   │
                     └─────────────────────────────────────────┘
```

Only Nginx port `80` is exposed for application traffic. Postgres and Redis are also published to the host so local tools, Cargo commands, and migration commands can reach them. Because Nginx serves the UI, API, and short links from one origin, the browser does not make cross-origin requests.

\* Nginx still forwards `/health`, but the current API does not implement a health endpoint.

---

## What each component does

| Path | Component | Responsibility |
| --- | --- | --- |
| `apps/web` | **Next.js frontend** | Authentication UI, URL dashboard, typed API client, JWT session handling, short-code redirect fallback, and missing/expired-link pages. |
| `apps/api` | **Rust API (Axum)** | Registration, authentication, user-scoped URL management, public redirects, JWT middleware, Postgres access, and Redis caching. |
| `crates/shared` | **Rust shared library** | Canonical request/response DTOs. Derives `serde` for JSON and `ts-rs` for TypeScript, and hosts the export binary. API DTO modules re-export these types. |
| `packages/shared-types` | **TypeScript package** | Generated DTO bindings plus a manually maintained success/error envelope matching the backend response shape. |
| `apps/api/migrations` | **Database migrations** | Versioned sqlx migrations owning the Postgres schema. |
| `nginx.conf` | **Reverse proxy** | Routes `/` to Next.js, `/api/*` to Rust, and eight-character short codes to Next.js so expired and missing links can show friendly pages. |
| `docker-compose.yaml` | **Orchestration** | Wires `nginx`, `web`, `api`, `db`, and `redis` together with networks, ports, environment variables, health checks, and volumes. |
| `Cargo.toml` | **Cargo workspace root** | Declares Rust members (`apps/api`, `crates/shared`) and shared dependency versions. |
| `package.json` | **npm workspace root** | Declares JS members (`apps/web`, `packages/*`) and top-level convenience scripts. |

---

## Shared types pipeline

Rust DTOs are the source of truth. Generated DTO bindings are checked in so Docker and frontend builds do not need to run Rust first.

```text
crates/shared/src/lib.rs          canonical DTOs
        │
        │  npm run generate-types   (cargo run -p shared --bin export)
        ▼
packages/shared-types/src/generated/*.ts
        │
        │  plus manually maintained response envelope in
        │  packages/shared-types/src/api.ts
        ▼
apps/web  ─────────────────────────────────────▶  apps/api
```

DTO examples include:

- `UserLoginDto`
- `UserRegisterDto`
- `UserReadDto`
- `TokenReadDto`
- `TokenClaimsDto`
- `UrlCreateDto`
- `UrlQueryDto`
- `UrlEditDto`
- `UrlReadDto`

The frontend also imports:

```ts
interface ApiSuccessResponse<T> {
  data: T;
}

interface ApiErrorResponse {
  message: string | null;
  code: number;
}
```

`i64` fields are exported as TypeScript `number`, UUIDs as `string`, and Rust `Option<T>` as `T | null`. After changing any DTO in `crates/shared`, re-run `npm run generate-types`.

---

## Project structure

```text
.
├── apps/
│   ├── api/                    # Rust Axum backend
│   │   ├── src/main.rs         # server bootstrap and shutdown
│   │   ├── src/routes/         # auth, registration, URL, redirect, and root routes
│   │   ├── src/handler/        # request handlers
│   │   ├── src/service/        # token, user, and URL business logic
│   │   ├── src/repository/     # Postgres and Redis access
│   │   ├── src/state/          # route state and dependency wiring
│   │   ├── src/middleware/     # JWT authentication middleware
│   │   ├── src/entity/         # database row models
│   │   ├── src/dto/            # re-exports of canonical shared DTOs
│   │   ├── src/config/         # parameters, Postgres, and Redis setup
│   │   ├── src/utils/          # success/error response helpers
│   │   ├── migrations/         # sqlx migrations (schema source of truth)
│   │   ├── Cargo.toml
│   │   └── Dockerfile
│   └── web/                    # Next.js frontend
│       ├── src/app/            # home, not-found, missing-link, and expired-link pages
│       ├── src/components/     # auth and URL-dashboard UI
│       ├── src/lib/            # typed API client and session handling
│       ├── src/proxy.ts        # short-code redirect fallback
│       ├── next.config.ts      # standalone output, rewrites, shared-types
│       ├── package.json
│       └── Dockerfile
├── crates/
│   └── shared/                 # DTOs + ts-rs export binary
├── packages/
│   └── shared-types/           # generated DTOs plus response-envelope types
├── nginx.conf                  # reverse-proxy config
├── docker-compose.yaml         # service orchestration
├── Cargo.toml                  # Cargo workspace root
├── package.json                # npm workspace root
└── .env                        # local environment variables
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
npm run migrate:info   # inspect applied and pending migrations
npm run migrate:up     # apply pending migrations
```

Then open:

- `http://localhost/` — Next.js app
- `http://localhost/api/auth` — login API
- `http://localhost/api/register` — registration API
- `http://localhost/AbC123Xy` — example short-link route

Stop with `npm run docker:down`.

Targeted code changes do **not** require rebuilding every service. For example:

```bash
docker compose up -d --build api web
```

Plain `docker compose up -d` reuses the existing images, so source edits will not appear until the affected images are rebuilt. Restart Nginx after changing `nginx.conf`.

### Option B — Local development

Run Postgres and Redis through Docker, then run each app locally:

```bash
# terminal 1 — Postgres and Redis
docker compose up -d db redis

# terminal 2 — Rust API (http://localhost:8080)
npm run dev:api

# terminal 3 — Next.js (http://localhost:3000)
npm run dev:web
```

In local development:

- Next.js rewrites `/api/*` to `http://localhost:8080`.
- The Next.js short-code proxy also defaults to `http://localhost:8080`.
- The production web container instead uses `API_URL=http://api:8080`.
- Copy `.env.example` to `.env` and set a real `JWT_SECRET` before running locally.

---

## Root commands

| Command | Description |
| --- | --- |
| `npm run dev:web` | Start the Next.js dev server |
| `npm run dev:api` | Run the Rust API with Cargo |
| `npm run types` | Alias for regenerating shared TypeScript types |
| `npm run generate-types` | Regenerate TypeScript types from Rust DTOs |
| `npm run migrate:up` | Apply all pending sqlx migrations |
| `npm run migrate:down` | Revert the most recent migration |
| `npm run migrate:info` | List applied / pending migrations |
| `npm run migrate:add -- <name>` | Create a new reversible migration |
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
npm run migrate:info            # list applied / pending migrations
npm run migrate:up              # apply all pending migrations
npm run migrate:down            # revert the most recent migration
npm run migrate:add -- <name>   # create a new reversible migration
```

Migrations are **not** applied automatically when the API starts—run them explicitly with `npm run migrate:up`.

Current schema behavior:

- `users` has unique names, emails, and phone numbers.
- Registration phone numbers are optional and stored as `NULL` when omitted.
- `urls.short_code` is globally unique.
- `urls.user_id` is optional and set to `NULL` when its owning user is deleted.
- `urls.expires_at` is nullable; `NULL` means the short link never expires.
- New links default to a 30-day expiry.
- Do not edit an already-applied migration; create a new migration instead. Editing applied files can cause checksum mismatches.

To reset the database back to zero, use the raw-Postgres reset instead of reverting migrations. It drops and recreates the `public` schema, removing all tables and data (including `_sqlx_migrations`):

```bash
npm run db:reset
```

---

## Authentication and sessions

1. The frontend offers sign-in and registration tabs.
2. Registration returns the new user profile, after which the frontend immediately signs that user in.
3. Login returns a JWT valid for 30 minutes.
4. Protected API routes require:
   ```text
   Authorization: Bearer <token>
   ```
5. Backend JWT middleware validates the token and loads the corresponding user.
6. The frontend stores the token in browser local storage, derives identity and expiry from its claims, signs out automatically when it expires, and returns to authentication after a `401`.

---

## API endpoints

All successful JSON responses use:

```json
{
  "data": {}
}
```

API errors use:

```json
{
  "message": "Url not found",
  "code": 404
}
```

| Method | Path | Authentication | Description |
| --- | --- | --- | --- |
| `POST` | `/api/register` | No | Creates a user and returns `UserReadDto`. Phone may be omitted or `null`. |
| `POST` | `/api/auth` | No | Accepts email/password and returns `TokenReadDto`. |
| `GET` | `/api/urls` | Yes | Returns the authenticated user’s URLs, newest first. |
| `GET` | `/api/url?long_url=...` | Yes | Returns the authenticated user’s newest mapping for one long URL. |
| `POST` | `/api/create_url` | Yes | Creates a short URL. Returns `201` for a new mapping or `200` with the existing active mapping. |
| `PUT` | `/api/edit_url` | Yes | Changes the destination and lifetime owned by one short code. |
| `DELETE` | `/api/delete_url?long_url=...` | Yes | Deletes the authenticated user’s mapping and returns `204`. |
| `GET` | `/{short_code}` | No | Public redirect to the original URL. |

The edit request must include both fields:

```json
{
  "short_code": "AbC123Xy",
  "new_url": "https://example.com/new-destination",
  "expires_in_minutes": 120
}
```

Supported edit lifetimes are:

| Selection | `expires_in_minutes` | Meaning |
| --- | --- | --- |
| 30 minutes | `30` | Expiry is calculated 30 minutes from edit time. |
| 2 hours | `120` | Expiry is calculated 2 hours from edit time. |
| 1 week | `10080` | Expiry is calculated 1 week from edit time. |
| 1 year | `525600` | Expiry is calculated 365 days from edit time. |
| No expiry | `null` | Stored expiry is `NULL`; the link never expires. |

Other numeric lifetimes are rejected with `400 Invalid link lifetime`. Editing the same row is allowed; moving a destination onto a different live row owned by the same user returns `409`.

---

## Redirects and expiry

- Short codes are eight Base62 characters.
- Codes are derived from the owning user and long URL, so different users receive different codes for the same destination.
- Redis and Postgres expiry checks use the stored `expires_at`.
- Direct API behavior is:
  - valid code: `307` with a `Location` header;
  - unknown or malformed code: `404`;
  - expired code: `410`.
- Production domain behavior is:
  - valid code: browser is redirected to the original website;
  - unknown code: browser shows `/missing`;
  - expired code: browser shows `/expired`.

The Next.js proxy is the fallback used when a short-code request reaches the web service directly. Nginx routes production eight-character short codes to the web service.

---

## Redis cache

Postgres remains authoritative. Redis accelerates repeated lookups and preserves expiry metadata.

Each cached short URL is stored as a JSON string under two keys:

```text
url:user:{user_id}:{long_url}
url:code:{short_code}
```

The cached JSON includes `id`, `long_url`, `short_code`, creation/update timestamps, and `expires_at`.

Cache behavior:

- Reads check Redis before Postgres.
- Expiring entries use a Redis TTL equal to their remaining lifetime.
- Unlimited entries are stored persistently, with `"expires_at": null`.
- Expired cache entries are treated as misses or dead links.
- Edits refresh both keys; deletions remove both keys.
- Redis failures are logged and fall back to Postgres rather than failing the request.
- `GET /api/urls` always reads the complete user listing from Postgres.

---

## Environment variables

Defined in `.env` (see `.env.example`):

| Variable | Purpose |
| --- | --- |
| `POSTGRES_USER` | Postgres user |
| `POSTGRES_PASSWORD` | Postgres password |
| `POSTGRES_DB` | Postgres database name |
| `DATABASE_URL` | Host-view connection string used by local API runs and the `sqlx` CLI (`localhost`). The API container gets its own `db:5432` URL from `docker-compose.yaml`. |
| `REDIS_URL` | Host-view Redis connection used locally (`127.0.0.1`). The API container uses `redis:6379`. |
| `APP_URL` | Network interface bound by the Rust API. |
| `APP_PORT` | Port bound by the Rust API. |
| `JWT_SECRET` | Secret used to sign authentication tokens. |
| `API_URL` | Server-side API address used by the web service’s short-code fallback. Compose sets this to `http://api:8080`. |
| `NODE_ENV` | Runtime mode for the web container. |
| `RUST_LOG` | API logging level in Compose. |

---

## Notes

- The Postgres schema lives in `apps/api/migrations/` and is **not** applied automatically; run `npm run migrate:up` after starting the database.
- Generated files in `packages/shared-types/src/generated/` are checked in so the frontend and Docker builds work without running Rust first. Regenerate them with `npm run generate-types` after DTO changes.
- The Next.js standalone Docker image expects those generated files to exist at build time.
- Postgres data persists in the `db_data` volume. Redis has no Compose volume and is treated as ephemeral cache state.
- Rebuilding services can leave old untagged images. That is normal Docker behavior; remove only dangling images if disk space becomes an issue, and never prune volumes unless persistent database data may be deleted.
- Because Nginx serves everything from a single origin, the browser never makes cross-origin requests.
