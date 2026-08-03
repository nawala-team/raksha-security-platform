# Raksha Security Platform — Installation Guide

Step-by-step instructions to install and run the Raksha Security Platform
locally (bare metal / Termux) or in production with Docker Compose.

## Architecture overview

The platform is a workspace monorepo:

| Component   | Path          | Tech                             | Port |
|-------------|---------------|----------------------------------|------|
| Portal API  | `apps/portal` | Rust (axum + sqlx + PostgreSQL)  | 8080 |
| Web UI      | `apps/web`    | Next.js 16 + React + Tailwind    | 3000 |
| Agent       | `apps/agent`  | Rust collector                   | –    |
| CLI         | `apps/cli`    | Rust                             | –    |
| Core libs   | `crates/*`    | Shared domain/business logic     | –    |

The Next.js dashboard proxies `/api/*` to the internal portal, so the portal
never needs to be exposed to browsers.

---

## Prerequisites

- **Rust** (stable) + Cargo
- **Node.js 18+** and **npm**
- **PostgreSQL 13+** (16/18 recommended)
- **Redis 6+**
- (Optional) Docker Engine 24+ & Docker Compose v2 for the production path

On Android/Termux you additionally need a WebAssembly SWC compiler for Next.js —
the repo already configures `experimental.useWasmBinary` in
`apps/web/next.config.js`.

---

## Option A — Local / bare-metal install (step by step)

### 1. Clone the repository

```sh
git clone https://github.com/dansiapa/raksha-security-platform.git
cd raksha-security-platform
```

### 2. Configure environment

```sh
cp .env.example .env
```

Edit `.env` and at minimum set:

```ini
RAKSHA__SERVER__HOST=0.0.0.0
RAKSHA__SERVER__PORT=8080

RAKSHA__DATABASE__URL=postgres://raksha:raksha_dev@localhost:5432/raksha_platform
RAKSHA__REDIS__URL=redis://localhost:6379

RAKSHA__JWT__SECRET=change-me-to-a-long-random-string
RAKSHA__APP__ENVIRONMENT=development
```

> In production generate the JWT secret with `openssl rand -base64 48` and set
> `RAKSHA__APP__ENVIRONMENT=production`.

### 3. Start PostgreSQL and Redis

Start PostgreSQL against the data directory (trust auth is pre-configured in
`pg_hba.conf`):

```sh
initdb -D ~/postgresql-data          # only the first time
pg_ctl -D ~/postgresql-data -l ~/pg.log start
```

Create the role and database (the first `psql` runs as the current OS user):

```sh
psql -d postgres -c "CREATE ROLE raksha SUPERUSER LOGIN PASSWORD 'raksha_dev';"
psql -d postgres -c "CREATE DATABASE raksha_platform OWNER raksha;"
```

Start Redis:

```sh
redis-server --port 6379 --save ''
```

### 4. Apply database migrations (creates all tables)

```sh
cd raksha-security-platform
for f in migrations/*.sql; do
  psql "postgres://raksha:raksha_dev@localhost:5432/raksha_platform" -f "$f"
done
```

This creates all **53 tables** (users, tenants, agents, alerts, incidents, GRC,
honeypots, dark web, backups, documents, FIM, network, etc.) plus seed data for
the default tenant and built-in roles.

### 5. Build the portal (Rust)

The sqlx query macros need a live database at compile time:

```sh
export DATABASE_URL=postgres://raksha:raksha_dev@localhost:5432/raksha_platform
cargo build -p raksha-portal
```

### 6. Run the portal API

```sh
cd raksha-security-platform
RUST_LOG=info ./target/debug/raksha-portal
```

Verify it is healthy:

```sh
curl http://127.0.0.1:8080/api/v1/health
# {"status":"healthy","version":"0.1.0",...}
```

### 7. Build the web dashboard (Next.js)

The portal URL is baked into the build (`next.config.js` rewrites), so export it
before building:

```sh
cd apps/web
PORTAL_API_URL=http://localhost:8080 npm install
PORTAL_API_URL=http://localhost:8080 npm run build
```

### 8. Run the web dashboard

```sh
PORT=3000 PORTAL_API_URL=http://localhost:8080 npm run start
```

Open **http://localhost:3000** and log in.

### 9. One-command management

The repository includes `start-local.sh` for starting / stopping / checking the
whole stack:

```sh
./start-local.sh          # start PostgreSQL + Redis + portal + web
./start-local.sh status   # show what is running
./start-local.sh stop     # stop everything
```

---

## Option B — Production (Docker Compose)

1. Install Docker Engine 24+ and Docker Compose v2 on the server.
2. Clone the repo and copy the environment template:
   ```sh
   cp .env.example .env
   ```
3. Set unique `POSTGRES_PASSWORD` and `JWT_SECRET`:
   ```sh
   openssl rand -base64 48
   ```
4. Set `RAKSHA__APP__ENVIRONMENT=production`.
5. Start and verify:
   ```sh
   docker compose up -d --build
   docker compose ps
   docker compose logs --tail=100 web portal
   ```
6. Put a TLS reverse proxy (Nginx / Caddy) in front of the web service and
   terminate HTTPS there. Do **not** expose the portal, database, Redis or
   OpenSearch ports to the internet.

See `docs/PRODUCTION-DEPLOYMENT.md` for full production guidance.

---

## Default accounts

For development, users can self-register on the login page or be created by an
admin. Seed accounts (created via the API in the running instance):

| Role | Email | Password |
|------|-------|----------|
| Super Admin | `superadmin@raksha.local` | `RakshaSuper!2026` |
| Tenant Admin | `tenantadmin@raksha.local` | `RakshaTenant!2026` |
| Analyst | `analyst@raksha.local` | `RakshaAnalyst!2026` |
| Operator | `operator@raksha.local` | `RakshaOperator!2026` |
| Viewer | `viewer@raksha.local` | `RakshaViewer!2026` |

> Passwords are hashed with argon2 and cannot be recovered. Use the
> Administration → Users page (or `scripts/seed_roles.sql` to reassign roles)
> to reset or create accounts.

---

## Troubleshooting

- **`error: set DATABASE_URL to use query macros online`** — the Rust build needs
  `DATABASE_URL` exported (step 5).
- **`pool timed out while waiting for an open connection`** — PostgreSQL (or
  Redis) is down. Run `./start-local.sh status` and restart.
- **Web shows `Internal Server Error` on `/api/*`** — the portal is not running
  and the Next.js proxy cannot reach it. Keep the portal up on port 8080.
- **`role "raksha" does not exist`** — create the role/database (step 3).
