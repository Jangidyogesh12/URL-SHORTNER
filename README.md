# URL Shortener — Rust + Next.js Monorepo

> This is the **Rust + Next.js monorepo** implementation of the blog
> **["A URL Shortener Is Easy, Until You Scale It"](https://www.yogesharma.in/writing/url-shortener)**.
>
> Read it side-by-side with the blog: the blog reasons from constraints to
> architecture, this repo shows how those ideas translate into running code.

If you have read the blog, you already know the story:

`naive design → reverse proxy → cache → key generation → expiry`.
This repo implements exactly that path — no more, no less.

---

## What is implemented here

| Blog idea | What this repo does |
| --- | --- |
| **Reverse proxy / API gateway** | `nginx` is the single entrypoint. `:80` redirects to HTTPS, `:443` (Cloudflare Origin certs) routes `/api/*` to the Rust API, `/{8-char-code}` + `/` to Next.js. |
| **Horizontally scaled API** | 3 identical Axum replicas (`api1`, `api2`, `api3`) behind nginx round-robin (`api_upstream`). |
| **Cache in front of the DB** | Redis cache-aside. Reads check Redis first, miss falls through to Postgres and repopulates Redis. Redis failures log and fall back to Postgres. |
| **Write-through / invalidate on write** | Create warms both cache keys. Edit refreshes both keys (deletes the old `user → url` key). Delete removes both keys. |
| **Short-key generation** | `SHA256(user_id \| long_url [\| salt])` → Base62 (8 chars). Up to 5 attempts with a random salt on global `short_code` collision. Per-user: same user + same URL returns the existing active row (`200`), otherwise creates (`201`). |
| **Expiry with TTL** | `urls.expires_at` is source of truth. New links default to **30 days**. Edits accept only `30`, `120`, `10080`, `525600` minutes or `null` (never expires). Redis `SETEX` TTL = remaining lifetime; `null` expiry = persistent key. Expired rows return `410`; Next.js shows `/expired`. |
| **Public redirect** | `GET /{short_code}` → `307` + `Location`. Unknown / malformed → `404` (Next.js shows `/missing`). No auth required. |
| **User-scoped CRUD** | Register / login (JWT, 30 min, bcrypt passwords), then `create / get / list / edit / delete` — all scoped to the logged-in user. |
| **Single-origin frontend** | Next.js serves UI, dashboard, and the short-code fallback proxy (`src/proxy.ts`). Browser only talks to nginx, so no CORS. |
| **Typed contracts** | Rust DTOs in `crates/shared` are the source of truth; `ts-rs` generates `packages/shared-types/src/generated/*.ts`. |

> Scope note: this repo runs **one Postgres primary** (no read replicas, no sharding, no background expiry sweeper). Expiry is enforced lazily on read plus Redis TTL.

---

## Architecture

```text
                        ┌─────────────────────────────────────────────┐
                        │                    nginx                    │
                        │  :80  → 301 https://$host$request_uri       │
                        │  :443 ssl (Cloudflare Origin certs)         │
 browser ──────────────▶│    /api/*          → api_upstream (1/2/3)   │
         single origin  │    /XXXXXXXX (8×B62)→ web:3000              │
                        │    /                 → web:3000             │
                        └──────┬────────────────────────┬─────────────┘
                               │                        │
                  ┌────────────▼───────┐   ┌────────────▼────────────┐
                  │        web         │   │      api × 3            │
                  │      Next.js       │   │   Rust (Axum) :8080     │
                  │       :3000        │──▶│  auth + CRUD + redirect │
                  └────────────────────┘   └────────────┬────────────┘
                        short-code                       │
                        fallback                         ▼
                        proxy               ┌─────────────────────────┐
                                            │   Postgres :5432        │
                                            │   authoritative store   │
                                            └─────────────────────────┘
                                            ┌─────────────────────────┐
                                            │   Redis :6379           │
                                            │   ephemeral short-URL   │
                                            │   cache (no volume)     │
                                            └─────────────────────────┘
```

Only nginx ports `80`/`443` are application traffic. Postgres (`5432`) and
Redis (`6379`) are also published so local `cargo` / `sqlx` / GUI tools can
reach them.

---

## How it works (blog → code)

### 1. Gateway routing (`nginx.conf`)

- `location /api/` → `http://api_upstream` (`api1:8080`, `api2:8080`, `api3:8080`, round-robin).
- `location ~ "^/[0-9A-Za-z]{8}$"` → `web:3000` so unknown/expired codes render friendly pages instead of raw JSON.
- `location /` → `web:3000`.
- `:80` unconditionally returns `301 https://...`; `:443` serves TLS with `./certs/fullchain.pem` + `./certs/privkey.pem` (Cloudflare Origin certs for Full-Strict).

### 2. Short-code generation (`apps/api/src/service/url_service.rs`)

```text
digest = SHA256("{user_id}|{long_url}[|{random_salt}]")
code   = first 8 chars of digest bytes mapped through Base62
         "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
```

- Codes are **per-user**: different users get different codes for the same long URL.
- Global uniqueness enforced by `urls.short_code UNIQUE` + a pre-insert lookup; on collision a fresh `Uuid` salt is tried (max 5 attempts).
- Same user re-shortening an **active** URL gets the existing row back (`200`); only genuinely new rows return `201`.

### 3. Redis cache (`apps/api/src/repository/url_cache_repository.rs`)

Postgres is authoritative. Each URL is cached as JSON under **two keys**:

```text
url:user:{user_id}:{long_url}
url:code:{short_code}
```

- Read path: `cache → Postgres → repopulate cache`.
- `GET /api/urls` (full listing) always hits Postgres.
- TTL = `expires_at - now` (min 1s); `expires_at = NULL` = persistent key.
- Expired / malformed cache entries are treated as misses.
- Any Redis error is logged (`warn!`) and the request continues against Postgres.

### 4. Expiry & redirects

Direct API (`GET /{short_code}`):

| State | Response |
| --- | --- |
| Valid | `307 Temporary Redirect` + `Location: <original_url>` |
| Unknown / malformed (not 8 chars) | `404` |
| Expired (`expires_at <= now`) | `410` |

Production (through nginx → Next.js `src/proxy.ts`):

| State | Browser sees |
| --- | --- |
| Valid | Redirected to the original site (`307`) |
| Unknown | `/missing` page |
| Expired | `/expired` page |

Create defaults to `expires_at = now + 30 days`. Only edits can change the
lifetime, and only to `30 / 120 / 10080 / 525600` minutes or `null`.

### 5. Auth (JWT, 30 minutes)

1. `POST /api/register` → creates user (bcrypt-hashed password), frontend immediately logs in.
2. `POST /api/auth` → returns `{ token, iat, exp }` (`exp = iat + 30 min`).
3. Frontend stores the token in `localStorage`, attaches `Authorization: Bearer <token>`, auto-signs-out on expiry / `401`.
4. Axum middleware validates the JWT and loads the `User` for every `/api/url*` route.

### 6. Shared types pipeline

Rust is the source of truth — generated files are checked in so Docker / Vercel-style builds never need Cargo first:

```text
crates/shared/src/lib.rs        canonical DTOs (serde + ts-rs)
        │  npm run generate-types  (cargo run -p shared --bin export)
        ▼
packages/shared-types/src/generated/*.ts
        │  + hand-maintained envelope in packages/shared-types/src/api.ts
        │    { data: T } on success / { message, code } on error
        ▼
apps/web  ◀── typed fetch client (src/lib/api.ts) ──▶  apps/api
```

DTOs: `UserLoginDto`, `UserRegisterDto`, `UserReadDto`, `TokenReadDto`,
`TokenClaimsDto`, `UrlCreateDto`, `UrlQueryDto`, `UrlEditDto`, `UrlReadDto`.
`i64` → `number`, `Uuid` → `string`, `Option<T>` → `T | null`.

---

## Project structure

```text
.
├── apps/
│   ├── api/                  # Rust Axum backend (:8080)
│   │   ├── src/main.rs       # bootstrap, tracing, graceful shutdown
│   │   ├── src/routes/       # /api/* + /{short_code} wiring
│   │   ├── src/handler/      # thin HTTP adapters
│   │   ├── src/service/      # token / user / url business logic
│   │   ├── src/repository/   # Postgres + Redis access
│   │   ├── src/middleware/   # JWT auth
│   │   ├── src/entity/       # DB row models
│   │   ├── src/dto/          # re-exports of crates/shared
│   │   ├── src/config/       # env, Postgres pool, Redis manager
│   │   ├── src/utils/        # { data } / { message, code } helpers
│   │   ├── src/state/        # dependency wiring
│   │   ├── migrations/       # sqlx schema (source of truth)
│   │   └── Dockerfile
│   └── web/                  # Next.js frontend (:3000)
│       ├── src/app/          # home, /missing, /expired, not-found
│       ├── src/components/   # auth-card, auth-provider, url-dashboard
│       ├── src/lib/          # typed api.ts client + session.ts
│       ├── src/proxy.ts      # short-code fallback (410→/expired, 404→/missing)
│       └── Dockerfile        # standalone output
├── crates/shared/            # canonical DTOs + export binary
├── packages/shared-types/    # generated TS + ApiSuccess/Error envelope
├── nginx.conf                # gateway / TLS / upstream routing
├── docker-compose.yaml       # db, redis, api1-3, web, nginx
├── do.sh                     # prod-style run (prebuilt tarballs + certs check)
├── Cargo.toml                # Cargo workspace (apps/api, crates/shared)
└── package.json              # npm workspaces (apps/web, packages/*)
```

Both package managers live side by side: **npm workspaces** for TS,
**Cargo workspace** for Rust.

---

## API endpoints

Success envelope: `{ "data": ... }` · Error envelope: `{ "message": "...", "code": N }`

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| `POST` | `/api/register` | No | Create user → `UserReadDto`. `phone` optional / `null`. |
| `POST` | `/api/auth` | No | Email + password → `TokenReadDto` (30-min JWT). |
| `GET` | `/api/urls` | Yes | Own URLs, newest first (always from Postgres). |
| `GET` | `/api/url?long_url=...` | Yes | Own newest mapping for one long URL. |
| `POST` | `/api/create_url` | Yes | `{ long_url }` → `201` new / `200` existing active. Default expiry 30 days. |
| `PUT` | `/api/edit_url` | Yes | `{ short_code, new_url, expires_in_minutes }`. See lifetimes below. |
| `DELETE` | `/api/delete_url?long_url=...` | Yes | Delete own mapping → `204`. Clears both cache keys. |
| `GET` | `/{short_code}` | No | Public redirect (`307` / `404` / `410`). |

Edit lifetimes (`expires_in_minutes`):

| UI selection | Value | Meaning |
| --- | --- | --- |
| 30 minutes | `30` | `now + 30 min` |
| 2 hours | `120` | `now + 2 h` |
| 1 week | `10080` | `now + 7 d` |
| 1 year | `525600` | `now + 365 d` |
| No expiry | `null` | `expires_at = NULL`, never expires |

Any other number → `400 Invalid link lifetime`. Pointing `new_url` at a
different live row you own → `409`.

---

## Prerequisites

- Node.js 20+ and npm
- Rust (stable) + Cargo
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres,rustls`)
- Docker + Docker Compose
- Copy `.env.example` → `.env` and set a real `JWT_SECRET`

---

## Running it

### Option A — Docker (full stack)

```bash
npm run docker:up      # docker compose up --build
npm run migrate:info   # check pending migrations
npm run migrate:up     # apply schema (NOT automatic on boot)
```

Open `http://localhost/` (nginx → web / api). Stop with `npm run docker:down`.

Targeted rebuilds (source edits need a rebuild of the affected image):

```bash
docker compose up -d --build api1 api2 api3 web   # api/web changes
docker compose restart nginx                      # nginx.conf changes
```

### Option B — Local development

```bash
# terminal 1 — infra only
docker compose up -d db redis
npm run migrate:up

# terminal 2 — Rust API (http://localhost:8080)
npm run dev:api

# terminal 3 — Next.js (http://localhost:3000)
npm run dev:web
```

Locally, Next.js rewrites `/api/*` and the short-code proxy to
`http://localhost:8080`. In Compose, the web container uses `API_URL=http://api:8080`
(Docker DNS alias shared by `api1/2/3`).

### Production-style (`do.sh`)

`./do.sh --run` loads prebuilt `api.tar.gz` / `web.tar.gz`, requires
`certs/fullchain.pem` + `certs/privkey.pem` (Cloudflare Origin certs for
`yogesharma.space`), then starts `db → redis → api1/2/3 → web → nginx`.
`./do.sh --stop | --status | --logs [svc]` manages the stack.

---

## Root commands

| Command | Description |
| --- | --- |
| `npm run dev:web` / `npm run dev:api` | Dev servers |
| `npm run generate-types` (`npm run types`) | Regenerate TS from Rust DTOs (run after any DTO change) |
| `npm run build` | Types → web build → api release build |
| `npm run check` | `cargo check --workspace` |
| `npm run migrate:info / migrate:up / migrate:down` | Inspect / apply / revert sqlx migrations |
| `npm run migrate:add -- <name>` | New reversible migration pair |
| `npm run db:reset` | Drop + recreate `public` schema (wipes everything incl. `_sqlx_migrations`) |
| `npm run docker:up` / `npm run docker:down` | Full-stack up / down |

---

## Migrations & schema

Schema is owned by `apps/api/migrations/` (reversible `*.up.sql` / `*.down.sql`,
tracked in `_sqlx_migrations`, never auto-applied):

- `users`: unique `name`, `email`, `phone` (nullable); bcrypt password hash.
- `urls`: `id` (uuid PK), `user_id` nullable (`ON DELETE SET NULL`), `original_url` text, `short_code varchar(8)` globally unique + indexed, `created_at / updated_at`, `expires_at` nullable (`NULL` = never expires).

> Never edit an applied migration — add a new one. Editing causes sqlx checksum mismatches.

---

## Environment variables (`.env` ⊇ `.env.example`)

| Variable | Used by |
| --- | --- |
| `POSTGRES_USER / POSTGRES_PASSWORD / POSTGRES_DB` | Compose `db`, API container `DATABASE_URL` |
| `DATABASE_URL` | Local `cargo` runs + `sqlx` CLI (`localhost`); Compose overrides to `db:5432` |
| `REDIS_URL` | Local (`127.0.0.1`); Compose overrides to `redis:6379` |
| `APP_URL / APP_PORT` | Interface + port the Rust API binds (`0.0.0.0:8080`) |
| `JWT_SECRET` | Token signing (required, no default) |
| `API_URL` | Server-side API base for the web short-code fallback (`http://api:8080` in Compose) |
| `NODE_ENV / RUST_LOG` | Web mode / API log level in Compose |

---

## Notes

- Generated `packages/shared-types/src/generated/` files are checked in intentionally.
- Postgres data persists in the `db_data` volume; Redis is ephemeral (no volume).
- Rebuilds may leave dangling images — normal Docker behavior; never prune volumes unless you intend to delete DB data.
- Because nginx serves UI + API + short links from one origin, the browser never makes cross-origin requests.

---

*Companion to [yogesharma.in/writing/url-shortener](https://www.yogesharma.in/writing/url-shortener) — if something here is unclear, that post is the design rationale; if something there is abstract, this repo is the concrete answer.*
