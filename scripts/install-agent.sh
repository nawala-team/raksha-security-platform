#!/usr/bin/env bash
# Raksha Security Platform - Agent Installer (Universal Linux/macOS/FreeBSD)
set -euo pipefail

RAKSHA_VERSION="${RAKSHA_VERSION:-latest}"
RAKSHA_PORTAL_URL="${RAKSHA_PORTAL_URL:-http://localhost:8080}"
RAKSHA_AGENT_KEY="${RAKSHA_AGENT_KEY:-}"
INSTALL_DIR="/opt/raksha-agent"
CONFIG_DIR="/etc/raksha"
LOG_DIR="/var/log/raksha"
DOWNLOAD_BASE="https://github.com/raksha-security/raksha-platform/releases/download"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

detect_platform() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"
    case "$OS" in
        linux)   PLATFORM="linux" ;;
        darwin)  PLATFORM="darwin" ;;
        freebsd) PLATFORM="freebsd" ;;
        *)       log_error "Unsupported OS: $OS" ;;
    esac
    case "$ARCH" in
        x86_64|amd64)       ARCH="amd64" ;;
        aarch64|arm64)      ARCH="arm64" ;;
        armv7l|armv7)       ARCH="armv7" ;;
        *)                  log_error "Unsupported arch: $ARCH" ;;
    esac
    log_info "Platform: ${PLATFORM}-${ARCH}"
}

check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        log_error "Run as root: sudo $0"
    fi
}

download_agent() {
    local url="${DOWNLOAD_BASE}/v${RAKSHA_VERSION}/raksha-agent-${PLATFORM}-${ARCH}.tar.gz"
    local tmp="/tmp/raksha-agent.tar.gz"
    log_info "Downloading agent v${RAKSHA_VERSION} for ${PLATFORM}-${ARCH}..."
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "$tmp"
    elif command -v wget &>/dev/null; then
        wget -qO "$tmp" "$url"
    else
        log_error "curl or wget required"
    fi
    mkdir -p "$INSTALL_DIR/bin"
    tar -xzf "$tmp" -C "$INSTALL_DIR/bin/"
    chmod +x "$INSTALL_DIR/bin/raksha-agent"
    rm -f "$tmp"
    log_ok "Agent binary installed"
}

generate_config() {
    mkdir -p "$CONFIG_DIR" "$LOG_DIR"
    if [ -f "$CONFIG_DIR/agent.toml" ]; then
        log_warn "Agent config exists, skipping"
        return
    fi
    local agent_id
    agent_id=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || uuidgen 2>/dev/null || date +%s%N)
    cat > "$CONFIG_DIR/agent.toml" <<EOF
[agent]
id = "${agent_id}"
hostname = "$(hostname)"

[portal]
url = "${RAKSHA_PORTAL_URL}"
api_key = "${RAKSHA_AGENT_KEY}"
tls_verify = true

[collection]
interval_seconds = 30
cpu = true
memory = true
disk = true
network = true
processes = true

[logging]
level = "info"
file = "${LOG_DIR}/agent.log"
EOF
    chmod 640 "$CONFIG_DIR/agent.toml"
    log_ok "Config generated"
}

install_service() {
    case "$PLATFORM" in
        linux)
            if [ -d /etc/systemd/system ]; then
                cat > /etc/systemd/system/raksha-agent.service <<EOF
[Unit]
Description=Raksha Security Agent
After=network.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/bin/raksha-agent --config ${CONFIG_DIR}/agent.toml
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
                systemctl daemon-reload
                systemctl enable raksha-agent
                log_ok "Systemd service installed"
            elif [ -d /etc/init.d ]; then
                cat > /etc/init.d/raksha-agent <<EOF
#!/bin/sh
### BEGIN INIT INFO
# Provides:          raksha-agent
# Required-Start:    \$network
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Description:       Raksha Security Agent
### END INIT INFO
DAEMON=${INSTALL_DIR}/bin/raksha-agent
DAEMON_ARGS="--config ${CONFIG_DIR}/agent.toml"
case "\$1" in
    start) \$DAEMON \$DAEMON_ARGS & ;;
    stop) pkill -f raksha-agent ;;
    restart) \$0 stop; \$0 start ;;
esac
EOF
                chmod +x /etc/init.d/raksha-agent
                log_ok "Init.d service installed"
            fi
            ;;
        darwin)
            cat > /Library/LaunchDaemons/com.raksha.agent.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>Label</key><string>com.raksha.agent</string>
    <key>ProgramArguments</key><array>
        <string>${INSTALL_DIR}/bin/raksha-agent</string>
        <string>--config</string>
        <string>${CONFIG_DIR}/agent.toml</string>
    </array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
</dict></plist>
EOF
            launchctl load /Library/LaunchDaemons/com.raksha.agent.plist
            log_ok "Launchd service installed"
            ;;
        freebsd)
            cat > /usr/local/etc/rc.d/raksha_agent <<EOF
#!/bin/sh
# PROVIDE: raksha_agent
# REQUIRE: NETWORKING
. /etc/rc.subr
name="raksha_agent"
rcvar="raksha_agent_enable"
command="${INSTALL_DIR}/bin/raksha-agent"
command_args="--config ${CONFIG_DIR}/agent.toml"
run_rc_command "\$1"
EOF
            chmod +x /usr/local/etc/rc.d/raksha_agent
            sysrc raksha_agent_enable=YES
            log_ok "RC service installed"
            ;;
    esac
}

main() {
    echo ""
    echo "  Raksha Security Platform - Agent Installer"
    echo "  =========================================="
    echo ""
    check_root
    detect_platform
    download_agent
    generate_config
    install_service
    echo ""
    log_ok "Agent installed successfully!"
    log_ok "Binary:  $INSTALL_DIR/bin/raksha-agent"
    log_ok "Config:  $CONFIG_DIR/agent.toml"
    case "$PLATFORM" in
        linux)   log_ok "Start: systemctl start raksha-agent" ;;
        darwin)  log_ok "Start: launchctl start com.raksha.agent" ;;
        freebsd) log_ok "Start: service raksha_agent start" ;;
    esac
}

main "$@"
