# 🔱 Raksha Installation Guide

> Detailed setup instructions for all platforms

## Table of Contents

- [System Requirements](#system-requirements)
- [Docker Installation](#docker-installation)
- [Native Installation](#native-installation)
- [Configuration](#configuration)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum

| Resource | Requirement |
|----------|------------|
| CPU | 2 cores |
| RAM | 4 GB |
| Disk | 20 GB |
| OS | Linux (x86_64), macOS, Windows (WSL2) |

### Recommended (Production)

| Resource | Requirement |
|----------|------------|
| CPU | 8+ cores |
| RAM | 16+ GB |
| Disk | 100+ GB SSD |
| OS | Ubuntu 22.04 LTS / RHEL 9 |
| Network | 1 Gbps |

## Docker Installation

### Prerequisites

- Docker Engine 24+ or Docker Desktop
- Docker Compose v2

### Quick Start

```bash
# Single container (all-in-one with embedded SQLite)
docker run -d \
  --name raksha \
  -p 8080:8080 \
  -p 9090:9090 \
  -v raksha-data:/var/lib/raksha \
  nawala/raksha:latest
```

### Production (Docker Compose)

```bash
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform

# Copy and configure environment
cp .env.example .env
# Edit .env with your settings

# Start all services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f raksha-api
```

The `docker-compose.yml` starts:
- Raksha API server (port 8080)
- Raksha dashboard (port 3000)
- PostgreSQL 16 (port 5432)
- Redis 7 (port 6379)
- Kafka + Zookeeper (port 9092)

## Native Installation

### Linux (Ubuntu/Debian)

```bash
# 1. Install system dependencies
sudo apt update && sudo apt install -y \
  build-essential pkg-config libssl-dev \
  postgresql-16 redis-server \
  curl git

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable

# 3. Install Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# 4. Clone and build
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform
npm install
cargo build --release

# 5. Setup database
sudo -u postgres createuser raksha
sudo -u postgres createdb -O raksha raksha
cargo run --bin raksha-migrate

# 6. Configure
cp .env.example .env
# Edit .env with your database credentials

# 7. Start
./target/release/raksha-server
```

### macOS

```bash
# 1. Install Homebrew dependencies
brew install postgresql@16 redis openssl pkg-config

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Install Node.js
brew install node@20

# 4. Start services
brew services start postgresql@16
brew services start redis

# 5. Clone and build
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform
npm install
cargo build --release

# 6. Setup database
createuser raksha
createdb -O raksha raksha
cargo run --bin raksha-migrate

# 7. Configure and start
cp .env.example .env
./target/release/raksha-server
```

### Windows (WSL2)

```powershell
# Enable WSL2 and install Ubuntu 22.04
wsl --install -d Ubuntu-22.04
```

Then follow the Linux (Ubuntu/Debian) instructions inside WSL2.

## Configuration

After installation, configure Raksha by editing `raksha.toml`:

```bash
cp raksha.example.toml raksha.toml
```

Key settings to customize:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://raksha:your_password@localhost:5432/raksha"

[security]
jwt_secret = "generate-a-secure-random-string"
```

Generate a secure JWT secret:

```bash
openssl rand -base64 64
```

## Verification

After starting Raksha, verify the installation:

```bash
# Check API health
curl http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","version":"0.1.0","modules":["server","network","database","compliance","ml","audit"]}

# Check dashboard
open http://localhost:8080
```

## Troubleshooting

### Port already in use

```bash
# Find process using port 8080
lsof -i :8080
# or on Linux
ss -tlnp | grep 8080
```

### Database connection refused

```bash
# Verify PostgreSQL is running
systemctl status postgresql
# Check connection
pg_isready -h localhost -p 5432
```

### Permission denied errors

```bash
# Ensure correct ownership
sudo chown -R $USER:$USER /var/lib/raksha
```

### Build failures

```bash
# Update Rust toolchain
rustup update stable

# Clear build cache
cargo clean
cargo build --release
```

---

*Part of the Nawala Ecosystem* 🔱
