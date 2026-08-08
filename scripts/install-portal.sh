#!/usr/bin/env bash
# Raksha Security Platform - Portal Installer (Linux/macOS)
set -euo pipefail

RAKSHA_VERSION="${RAKSHA_VERSION:-latest}"
RAKSHA_USER="raksha"
RAKSHA_HOME="/opt/raksha"
RAKSHA_CONFIG="/etc/raksha"
RAKSHA_DATA="/var/lib/raksha"
RAKSHA_LOG="/var/log/raksha"
DOWNLOAD_BASE="https://github.com/raksha-security/raksha-platform/releases/download"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        log_error "This script must be run as root. Use: sudo $0"
    fi
}

detect_platform() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"
    case "$OS" in
        linux)  PLATFORM="linux" ;;
        darwin) PLATFORM="darwin" ;;
        *)      log_error "Unsupported OS: $OS" ;;
    esac
    case "$ARCH" in
        x86_64|amd64)   ARCH="amd64" ;;
        aarch64|arm64)  ARCH="arm64" ;;
        *)              log_error "Unsupported architecture: $ARCH" ;;
    esac
    log_info "Detected platform: ${PLATFORM}-${ARCH}"
}

check_dependencies() {
    for dep in curl tar; do
        command -v "$dep" &>/dev/null || log_error "Required: $dep"
    done
}

install_postgres() {
    if command -v psql &>/dev/null; then log_ok "PostgreSQL found"; return; fi
    log_info "Installing PostgreSQL..."
    if [ "$PLATFORM" = "linux" ]; then
        if command -v apt-get &>/dev/null; then
            apt-get update -qq && apt-get install -y -qq postgresql postgresql-client
        elif command -v dnf &>/dev/null; then
            dnf install -y postgresql-server postgresql
        elif command -v yum &>/dev/null; then
            yum install -y postgresql-server postgresql
        fi
    elif [ "$PLATFORM" = "darwin" ]; then
        command -v brew &>/dev/null && brew install postgresql@16
    fi
}

install_redis() {
    if command -v redis-server &>/dev/null; then log_ok "Redis found"; return; fi
    log_info "Installing Redis..."
    if [ "$PLATFORM" = "linux" ]; then
        if command -v apt-get &>/dev/null; then apt-get install -y -qq redis-server
        elif command -v dnf &>/dev/null; then dnf install -y redis
        elif command -v yum &>/dev/null; then yum install -y redis
        fi
    elif [ "$PLATFORM" = "darwin" ]; then
        command -v brew &>/dev/null && brew install redis
    fi
}

create_user() {
    if id "$RAKSHA_USER" &>/dev/null; then log_ok "User exists"; return; fi
    log_info "Creating user: $RAKSHA_USER"
    if [ "$PLATFORM" = "linux" ]; then
        useradd --system --no-create-home --shell /sbin/nologin "$RAKSHA_USER"
    elif [ "$PLATFORM" = "darwin" ]; then
        dscl . -create /Users/"$RAKSHA_USER"
        dscl . -create /Users/"$RAKSHA_USER" UserShell /usr/bin/false
    fi
}

create_directories() {
    mkdir -p "$RAKSHA_HOME/bin" "$RAKSHA_CONFIG" "$RAKSHA_DATA" "$RAKSHA_LOG"
    chown -R "$RAKSHA_USER":"$RAKSHA_USER" "$RAKSHA_HOME" "$RAKSHA_DATA" "$RAKSHA_LOG"
    chmod 750 "$RAKSHA_CONFIG"
}

download_portal() {
    local url="${DOWNLOAD_BASE}/v${RAKSHA_VERSION}/raksha-portal-${PLATFORM}-${ARCH}.tar.gz"
    local tmp="/tmp/raksha-portal.tar.gz"
    log_info "Downloading Raksha Portal v${RAKSHA_VERSION}..."
    curl -fsSL "$url" -o "$tmp" || log_error "Download failed"
    tar -xzf "$tmp" -C "$RAKSHA_HOME/bin/"
    chmod +x "$RAKSHA_HOME/bin/raksha-portal"
    rm -f "$tmp"
    log_ok "Portal binary installed"
}

generate_config() {
    if [ -f "$RAKSHA_CONFIG/portal.toml" ]; then log_warn "Config exists, skipping"; return; fi
    local jwt_secret
    jwt_secret=$(openssl rand -hex 32 2>/dev/null || head -c 64 /dev/urandom | base64 | tr -d '\n')
    cat > "$RAKSHA_CONFIG/portal.toml" <<EOF
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://raksha:${POSTGRES_PASSWORD:-CHANGE_ME}@localhost:5432/raksha"
max_connections = 20

[redis]
url = "redis://localhost:6379"

[auth]
jwt_secret = "${jwt_secret}"
token_expiry_hours = 24

[logging]
level = "info"
format = "json"
file = "/var/log/raksha/portal.log"
EOF
    chown "$RAKSHA_USER":"$RAKSHA_USER" "$RAKSHA_CONFIG/portal.toml"
    chmod 640 "$RAKSHA_CONFIG/portal.toml"
    log_ok "Config generated"
}

setup_database() {
    log_info "Setting up database..."
    if sudo -u postgres psql -lqt 2>/dev/null | cut -d \| -f 1 | grep -qw raksha; then
        log_ok "Database exists"; return
    fi
    sudo -u postgres psql -c "CREATE USER raksha WITH PASSWORD '${POSTGRES_PASSWORD:-CHANGE_ME}';" 2>/dev/null || true
    sudo -u postgres psql -c "CREATE DATABASE raksha OWNER raksha;" 2>/dev/null || true
    log_ok "Database configured"
}

install_systemd_service() {
    [ "$PLATFORM" != "linux" ] && return
    [ ! -d /etc/systemd/system ] && return
    cat > /etc/systemd/system/raksha-portal.service <<EOF
[Unit]
Description=Raksha Security Platform Portal
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=${RAKSHA_USER}
Group=${RAKSHA_USER}
WorkingDirectory=${RAKSHA_HOME}
ExecStart=${RAKSHA_HOME}/bin/raksha-portal --config ${RAKSHA_CONFIG}/portal.toml
Restart=always
RestartSec=5
LimitNOFILE=65536
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable raksha-portal
    log_ok "Systemd service installed"
}

install_launchd_service() {
    [ "$PLATFORM" != "darwin" ] && return
    cat > /Library/LaunchDaemons/com.raksha.portal.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>Label</key><string>com.raksha.portal</string>
    <key>ProgramArguments</key><array>
        <string>${RAKSHA_HOME}/bin/raksha-portal</string>
        <string>--config</string>
        <string>${RAKSHA_CONFIG}/portal.toml</string>
    </array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
    <key>UserName</key><string>${RAKSHA_USER}</string>
</dict></plist>
EOF
    launchctl load /Library/LaunchDaemons/com.raksha.portal.plist
    log_ok "Launchd service installed"
}

main() {
    echo ""
    echo "  Raksha Security Platform - Portal Installer"
    echo "  ============================================"
    echo ""
    check_root
    detect_platform
    check_dependencies
    install_postgres
    install_redis
    create_user
    create_directories
    download_portal
    generate_config
    setup_database
    install_systemd_service
    install_launchd_service
    echo ""
    log_ok "Raksha Portal installed successfully!"
    log_ok "Binary:  $RAKSHA_HOME/bin/raksha-portal"
    log_ok "Config:  $RAKSHA_CONFIG/portal.toml"
    log_ok "Logs:    $RAKSHA_LOG/"
    [ "$PLATFORM" = "linux" ] && log_ok "Start: systemctl start raksha-portal"
    [ "$PLATFORM" = "darwin" ] && log_ok "Start: launchctl start com.raksha.portal"
}

main "$@"
