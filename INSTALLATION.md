# Raksha Security Platform — Installation Guide

> **Current Version: 1.1.0** | [Changelog](CHANGELOG.md) | [Releases](https://github.com/dansiapa/raksha-security-platform/releases)

Step-by-step instructions to install and run the Raksha Security Platform
locally (bare metal / Termux) or in production with Docker Compose.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Prerequisites](#prerequisites)
- [Option A — Local / Bare-metal Install](#option-a--local--bare-metal-install-step-by-step)
- [Option B — Production (Docker Compose)](#option-b--production-docker-compose)
- [Option C — Kubernetes (Helm)](#option-c--kubernetes-helm)
- [Post-Installation](#post-installation)
- [Default Accounts](#default-accounts)
- [Environment Variables Reference](#environment-variables-reference)
- [Troubleshooting](#troubleshooting)

---

## Architecture Overview

The platform is a workspace monorepo:

| Component   | Path          | Tech                             | Port  |
|-------------|---------------|----------------------------------|-------|
| Portal API  | `apps/portal` | Rust (Axum + SQLx + PostgreSQL)  | 8080  |
| Web UI      | `apps/web`    | Next.js 14 + React + Tailwind    | 3000  |
| ML Engine   | `ml/`         | Python (FastAPI + scikit-learn)  | 8000  |
| Agent       | `apps/agent`  | Rust collector                   | –     |
| CLI         | `apps/cli`    | Rust                             | –     |
| Core libs   | `crates/*`    | Shared domain/business logic     | –     |

The Next.js dashboard proxies `/api/*` to the internal portal, so the portal
never needs to be exposed to browsers.

```
┌─────────────────────────────────────────────────────────────────┐
│                         Internet                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Reverse Proxy (Nginx/Caddy) + TLS                  │
│                        Port 443                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Web UI (Next.js)                             │
│                        Port 3000                                │
│              Proxies /api/* → Portal:8080                       │
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Portal API     │  │   ML Engine     │  │    Agents       │
│  (Rust/Axum)    │  │   (FastAPI)     │  │    (Rust)       │
│   Port 8080     │  │   Port 8000     │  │   Distributed   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
          │                   │
          └─────────┬─────────┘
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Data Layer                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │PostgreSQL│  │  Redis   │  │TimescaleDB│  │  OpenSearch  │    │
│  │  :5432   │  │  :6379   │  │   :5433   │  │    :9200     │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Prerequisites

### For Local Development

- **Rust** 1.75+ (stable) + Cargo
- **Node.js 20+** and **npm 10+**
- **PostgreSQL 15+** (16 recommended)
- **Redis 7+**

### For Production (Docker)

- **Docker Engine 24+**
- **Docker Compose v2.20+**
- **4 GB RAM minimum** (8 GB recommended)
- **20 GB disk space** minimum

### For Kubernetes

- **Kubernetes 1.28+**
- **Helm 3.12+**
- **kubectl** configured

> **Termux (Android)**: The repo configures `experimental.useWasmBinary` in
> `apps/web/next.config.js` for SWC compatibility.

---

## Option A — Local / Bare-metal Install (step by step)

### 1. Clone the repository

```sh
git clone https://github.com/dansiapa/raksha-security-platform.git
cd raksha-security-platform
```

### 2. Configure environment

```sh
cp .env.example .env
```

Edit `.env` and set minimum required values:

```ini
# Server
RAKSHA__SERVER__HOST=0.0.0.0
RAKSHA__SERVER__PORT=8080

# Database
RAKSHA__DATABASE__URL=postgres://raksha:raksha_dev@localhost:5432/raksha_platform

# Redis
RAKSHA__REDIS__URL=redis://localhost:6379

# JWT (generate with: openssl rand -base64 48)
RAKSHA__JWT__SECRET=your-secure-random-string-here

# Environment
RAKSHA__APP__ENVIRONMENT=development
```

### 3. Start PostgreSQL and Redis

**PostgreSQL:**

```sh
# Initialize (first time only)
initdb -D ~/postgresql-data

# Start
pg_ctl -D ~/postgresql-data -l ~/pg.log start

# Create role and database
psql -d postgres -c "CREATE ROLE raksha SUPERUSER LOGIN PASSWORD 'raksha_dev';"
psql -d postgres -c "CREATE DATABASE raksha_platform OWNER raksha;"
```

**Redis:**

```sh
redis-server --port 6379 --daemonize yes
```

### 4. Apply database migrations

```sh
cd raksha-security-platform
for f in migrations/*.sql; do
  psql "postgres://raksha:raksha_dev@localhost:5432/raksha_platform" -f "$f"
done
```

This creates all **53 tables** (users, tenants, agents, alerts, incidents, GRC,
honeypots, dark web, backups, documents, FIM, network, etc.) plus seed data.

### 5. Build the portal (Rust)

```sh
export DATABASE_URL=postgres://raksha:raksha_dev@localhost:5432/raksha_platform
cargo build --release -p raksha-portal
```

### 6. Run the portal API

```sh
RUST_LOG=info ./target/release/raksha-portal
```

Verify health:

```sh
curl http://127.0.0.1:8080/api/v1/health
# {"status":"healthy","version":"1.1.0",...}
```

### 7. Build the web dashboard

```sh
cd apps/web
npm install
PORTAL_API_URL=http://localhost:8080 npm run build
```

### 8. Run the web dashboard

```sh
PORT=3000 npm run start
```

Open **http://localhost:3000** and log in.

### 9. One-command management (optional)

```sh
./start-local.sh          # Start all services
./start-local.sh status   # Check status
./start-local.sh stop     # Stop all services
```

---

## Option B — Production (Docker Compose)

### 1. Server Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU      | 2 cores | 4+ cores    |
| RAM      | 4 GB    | 8+ GB       |
| Disk     | 20 GB   | 50+ GB SSD  |
| OS       | Ubuntu 22.04 / Debian 12 / RHEL 9 |

### 2. Install Docker

```sh
# Ubuntu/Debian
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Verify
docker --version   # Should be 24+
docker compose version   # Should be v2.20+
```

### 3. Clone and Configure

```sh
git clone https://github.com/dansiapa/raksha-security-platform.git
cd raksha-security-platform
cp .env.example .env
```

### 4. Generate Secure Secrets

```sh
# Generate secure passwords
POSTGRES_PWD=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 48)
GRAFANA_PWD=$(openssl rand -base64 16)

# Update .env file
sed -i "s/POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=${POSTGRES_PWD}/" .env
sed -i "s/JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" .env
sed -i "s/GRAFANA_ADMIN_PASSWORD=.*/GRAFANA_ADMIN_PASSWORD=${GRAFANA_PWD}/" .env

# Set production mode
sed -i "s/RAKSHA__APP__ENVIRONMENT=.*/RAKSHA__APP__ENVIRONMENT=production/" .env
```

### 5. Configure Firewall

```sh
# UFW (Ubuntu)
sudo ufw allow 22/tcp      # SSH
sudo ufw allow 80/tcp      # HTTP (redirect to HTTPS)
sudo ufw allow 443/tcp     # HTTPS
sudo ufw enable

# Do NOT expose these ports externally:
# - 8080 (Portal API)
# - 5432 (PostgreSQL)
# - 6379 (Redis)
# - 9200 (OpenSearch)
```

### 6. Start Services

**Basic (Portal + Web + Database):**

```sh
docker compose up -d --build
```

**With SIEM Integration:**

```sh
docker compose --profile siem up -d --build
```

**With Monitoring (Grafana + Prometheus):**

```sh
docker compose --profile monitoring up -d --build
```

**Full Stack (All Services):**

```sh
docker compose --profile siem --profile monitoring up -d --build
```

### 7. Verify Deployment

```sh
# Check all containers are running
docker compose ps

# Check logs
docker compose logs --tail=100 portal web

# Test health endpoints
curl -f http://localhost:8080/api/v1/health
curl -f http://localhost:3000
```

### 8. Setup Reverse Proxy (Nginx)

Create `/etc/nginx/sites-available/raksha`:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        proxy_read_timeout 86400;
    }
}
```

Enable and restart:

```sh
sudo ln -s /etc/nginx/sites-available/raksha /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### 9. SSL Certificate (Let's Encrypt)

```sh
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

### 10. Setup Auto-restart on Boot

```sh
sudo systemctl enable docker

# Create systemd service
cat << 'EOF' | sudo tee /etc/systemd/system/raksha.service
[Unit]
Description=Raksha Security Platform
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/raksha-security-platform
ExecStart=/usr/bin/docker compose up -d
ExecStop=/usr/bin/docker compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable raksha
```

---

## Option C — Kubernetes (Helm)

### 1. Add Helm Repository

```sh
helm repo add raksha https://dansiapa.github.io/raksha-security-platform/charts
helm repo update
```

### 2. Create Namespace and Secrets

```sh
kubectl create namespace raksha

kubectl create secret generic raksha-secrets \
  --namespace raksha \
  --from-literal=postgres-password=$(openssl rand -base64 32) \
  --from-literal=jwt-secret=$(openssl rand -base64 48) \
  --from-literal=redis-password=$(openssl rand -base64 16)
```

### 3. Install Chart

```sh
helm install raksha raksha/raksha-platform \
  --namespace raksha \
  --set global.environment=production \
  --set ingress.enabled=true \
  --set ingress.host=your-domain.com \
  --set ingress.tls.enabled=true
```

### 4. Verify

```sh
kubectl get pods -n raksha
kubectl get svc -n raksha
kubectl logs -n raksha deployment/raksha-portal
```

See `deploy/k8s/helm/` for full chart documentation.

---

## Post-Installation

### 1. Create Admin Account

After first deployment, register an admin account via the web UI at `/register`
or use the CLI:

```sh
# Docker
docker compose exec portal raksha-cli user create \
  --email admin@example.com \
  --password "SecurePassword123!" \
  --role super_admin

# Local
./target/release/raksha-cli user create \
  --email admin@example.com \
  --password "SecurePassword123!" \
  --role super_admin
```

### 2. Enroll Agents

1. Go to **Settings → Agent Enrollment** in the web UI
2. Generate an enrollment token
3. Install agent on target servers:

```sh
curl -sSL https://your-domain.com/install-agent.sh | \
  ENROLLMENT_TOKEN="your-token" \
  PORTAL_URL="https://your-domain.com" \
  bash
```

### 3. Configure Notifications

Set up alert channels in **Settings → Notifications**:
- Email (SMTP)
- Telegram Bot
- Slack Webhook
- Custom Webhooks

---

## Default Accounts

For development/testing, seed accounts are available:

| Role         | Email                       | Password            |
|--------------|-----------------------------|---------------------|
| Super Admin  | `superadmin@raksha.local`   | `RakshaSuper!2026`  |
| Tenant Admin | `tenantadmin@raksha.local`  | `RakshaTenant!2026` |
| Analyst      | `analyst@raksha.local`      | `RakshaAnalyst!2026`|
| Operator     | `operator@raksha.local`     | `RakshaOperator!2026`|
| Viewer       | `viewer@raksha.local`       | `RakshaViewer!2026` |

> ⚠️ **Change these passwords immediately in production!**

---

## Environment Variables Reference

### Core Settings

| Variable | Description | Default |
|----------|-------------|---------|1
| `RAKSHA__SERVER__HOST` | API bind address | `0.0.0.0` |
| `RAKSHA__SERVER__PORT` | API port | `8080` |
| `RAKSHA__DATABASE__URL` | PostgreSQL connection string | – |
| `RAKSHA__REDIS__URL` | Redis connection string | – |
| `RAKSHA__JWT__SECRET` | JWT signing key (min 32 chars) | – |
| `RAKSHA__APP__ENVIRONMENT` | `development` or `production` | `development` |

### Docker Compose Ports

| Variable | Description | Default |
|----------|-------------|---------|1
| `PORTAL_PORT` | Portal API | `8080` |
| `WEB_PORT` | Web UI | `3000` |
| `ML_PORT` | ML Engine | `8000` |
| `POSTGRES_PORT` | PostgreSQL | `5432` |
| `REDIS_PORT` | Redis | `6379` |
| `GRAFANA_PORT` | Grafana | `3001` |

### SIEM Integration

| Variable | Description | Default |
|----------|-------------|---------|1
| `SIEM_ENABLED` | Enable SIEM forwarding | `false` |
| `SIEM_TARGET` | `splunk`, `elasticsearch`, `wazuh`, `graylog`, `syslog`, `loki` | `splunk` |
| `SIEM_HOST` | SIEM server address | – |
| `SIEM_PORT` | SIEM server port | – |
| `SIEM_TOKEN` | Authentication token | – |
| `SIEM_FORMAT` | `cef`, `leef`, `json`, `syslog` | `cef` |

---

## Troubleshooting

### Common Issues

| Problem | Solution |
|---------|----------|
| `error: set DATABASE_URL` | Export `DATABASE_URL` before Rust build |
| `pool timed out` | PostgreSQL/Redis not running. Check `docker compose ps` |
| `Internal Server Error on /api/*` | Portal not running. Check `docker compose logs portal` |
| `role "raksha" does not exist` | Create role/database (see step 3 local install) |
| `connection refused` | Check firewall rules and service ports |
| `certificate verify failed` | Ensure SSL certs are valid and not expired |

### View Logs

```sh
# All services
docker compose logs -f

# Specific service
docker compose logs -f portal
docker compose logs -f web

# Last 100 lines
docker compose logs --tail=100 portal
```

### Reset Database

```sh
# ⚠️ This deletes all data!
docker compose down -v
docker compose up -d --build
```

### Health Checks

```sh
# Portal API
curl http://localhost:8080/api/v1/health

# Web UI
curl http://localhost:3000

# PostgreSQL
docker compose exec postgres pg_isready

# Redis
docker compose exec redis redis-cli ping
```

---

## Updates

### Docker Compose

```sh
cd raksha-security-platform
git pull --ff-only
docker compose down
docker compose up -d --build
docker compose ps
```

### Kubernetes

```sh
helm repo update
helm upgrade raksha raksha/raksha-platform --namespace raksha
```

---

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issue Tracker](https://github.com/dansiapa/raksha-security-platform/issues)
- 📧 Contact: dummymailrangga@gmail.com

---

<p align="center">
  <sub>Part of the <a href="https://github.com/dansiapa/nawala-gateway-platform">Nawala Ecosystem</a></sub>
</p>
