#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Raksha Security Platform — local / bare-metal startup
# Brings up PostgreSQL, Redis, the portal API (Rust) and the web dashboard
# (Next.js) in one command. Works on Linux, macOS and Termux/Android.
#
#   ./start-local.sh            # start everything (default)
#   ./start-local.sh status     # show what is currently running
#   ./start-local.sh stop       # stop everything
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PGDATA="${PGDATA:-$HOME/postgresql-data}"
PORTAL_PORT="${PORTAL_PORT:-8080}"
WEB_PORT="${WEB_PORT:-3000}"
REDIS_PORT="${REDIS_PORT:-6379}"

LOG_DIR="${LOG_DIR:-$HOME/raksha-logs}"
mkdir -p "$LOG_DIR"

# Load only the connection-related vars from .env (the file is not shell-safe
# because some values contain spaces), falling back to sane defaults.
DATABASE_URL="$(
  grep -E '^RAKSHA__DATABASE__URL=' "$ROOT/.env" 2>/dev/null | head -1 \
    | cut -d= -f2- \
    | tr -d '\r'
)"
DATABASE_URL="${DATABASE_URL:-postgres://raksha:raksha_dev@localhost:5432/raksha_platform}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"

pg_running() { [ -f "$PGDATA/postmaster.pid" ] && kill -0 "$(head -1 "$PGDATA/postmaster.pid")" 2>/dev/null; }

start_db() {
  if pg_running; then
    echo "[db] PostgreSQL already running"
  else
    echo "[db] starting PostgreSQL on 127.0.0.1:${POSTGRES_PORT:-5432}"
    pg_ctl -D "$PGDATA" -l "$LOG_DIR/postgres.log" -o "-p ${POSTGRES_PORT:-5432}" start
  fi
}

start_redis() {
  if redis-cli -h 127.0.0.1 -p "$REDIS_PORT" ping >/dev/null 2>&1; then
    echo "[redis] already running on :$REDIS_PORT"
  else
    echo "[redis] starting on 127.0.0.1:$REDIS_PORT"
    redis-server --port "$REDIS_PORT" --save '' >"$LOG_DIR/redis.log" 2>&1 &
  fi
}

start_portal() {
  if [ ! -x "$ROOT/target/debug/raksha-portal" ]; then
    echo "[portal] binary not found — building (with DATABASE_URL for sqlx macros)..."
    (cd "$ROOT" && DATABASE_URL="$DATABASE_URL" cargo build -p raksha-portal)
  fi
  if curl -sf -m 2 "http://127.0.0.1:${PORTAL_PORT}/api/v1/health" >/dev/null 2>&1; then
    echo "[portal] already running on :$PORTAL_PORT"
  else
    echo "[portal] starting API on 0.0.0.0:$PORTAL_PORT"
    (cd "$ROOT" && RUST_LOG=info setsid ./target/debug/raksha-portal \
      >"$LOG_DIR/portal.log" 2>&1 </dev/null &)
  fi
}

start_web() {
  if [ ! -d "$ROOT/apps/web/.next/BUILD_ID" ] && [ ! -f "$ROOT/apps/web/.next/BUILD_ID" ]; then
    echo "[web] .next build missing — building..."
    (cd "$ROOT/apps/web" && PORTAL_API_URL="http://localhost:${PORTAL_PORT}" npm run build)
  else
    echo "[web] using existing .next build"
  fi
  if curl -sf -m 2 "http://127.0.0.1:${WEB_PORT}/login" >/dev/null 2>&1; then
    echo "[web] already running on :$WEB_PORT"
  else
    echo "[web] starting dashboard on 0.0.0.0:$WEB_PORT"
    (cd "$ROOT/apps/web" && PORT="$WEB_PORT" PORTAL_API_URL="http://localhost:${PORTAL_PORT}" \
      setsid npm run start >"$LOG_DIR/web.log" 2>&1 </dev/null &)
  fi
}

status() {
  echo "── Raksha stack status ─────────────────────────────"
  psql -h 127.0.0.1 -p "${POSTGRES_PORT:-5432}" -d raksha_platform -U raksha \
    -tAc "SELECT 'postgres: up, '||count(*)||' tables' FROM pg_tables WHERE schemaname='public'" 2>/dev/null \
    || echo "postgres: DOWN"
  redis-cli -h 127.0.0.1 -p "$REDIS_PORT" ping 2>/dev/null | sed 's/^/redis: /' || echo "redis: DOWN"
  curl -sf -m 3 "http://127.0.0.1:${PORTAL_PORT}/api/v1/health" >/dev/null 2>&1 && echo "portal: up (:$PORTAL_PORT)" || echo "portal: DOWN"
  curl -sf -m 3 -o /dev/null "http://127.0.0.1:${WEB_PORT}/login" 2>/dev/null && echo "web: up (:$WEB_PORT)" || echo "web: DOWN"
}

stop() {
  pkill -f raksha-portal 2>/dev/null || true
  pkill -f 'next-server' 2>/dev/null || true
  redis-cli -h 127.0.0.1 -p "$REDIS_PORT" shutdown nosave 2>/dev/null || true
  pg_ctl -D "$PGDATA" stop -m fast 2>/dev/null || true
  echo "All services stopped."
}

case "${1:-start}" in
  start) start_db; start_redis; start_portal; start_web; echo; status ;;
  status) status ;;
  stop) stop ;;
  *) echo "usage: $0 [start|status|stop]"; exit 1 ;;
esac
