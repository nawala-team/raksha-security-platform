#!/bin/bash
# Raksha Agent Installer for Linux/macOS
# Usage: curl -fsSL https://portal/api/v1/agent/install | RAKSHA_TOKEN="rkat_xxx" RAKSHA_PORTAL="https://portal" bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
INSTALL_DIR="/opt/raksha-agent"
CONFIG_DIR="/etc/raksha"
LOG_DIR="/var/log/raksha"
SERVICE_NAME="raksha-agent"

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

check_prerequisites() {
    [ "$(id -u)" -ne 0 ] && error "Must be run as root (use sudo)"
    [ -z "${RAKSHA_TOKEN:-}" ] && error "RAKSHA_TOKEN is required"
    [ -z "${RAKSHA_PORTAL:-}" ] && error "RAKSHA_PORTAL is required"
    [[ ! "$RAKSHA_TOKEN" =~ ^rkat_ ]] && error "Invalid token format"
    info "Preflight checks passed"
}

detect_system() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    case "$ARCH" in x86_64|amd64) ARCH="x86_64";; aarch64|arm64) ARCH="aarch64";; *) error "Unsupported: $ARCH";; esac
    case "$OS" in linux) OS="linux";; darwin) OS="darwin";; *) error "Unsupported: $OS";; esac
    info "Detected: ${OS}/${ARCH}"
}

collect_fingerprint() {
    HOSTNAME=$(hostname); OS_VERSION=$(uname -r)
    CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
    TOTAL_MEM=$(free -b 2>/dev/null | awk '/^Mem:/{print $2}' || echo 0)
    MACHINE_ID=$(cat /etc/machine-id 2>/dev/null || hostname | sha256sum | cut -d' ' -f1)
    MAC_HASH=$(ip link show 2>/dev/null | grep -m1 'link/ether' | awk '{print $2}' | sha256sum | cut -d' ' -f1 || echo "unknown")
    info "Fingerprint collected"
}

download_agent() {
    mkdir -p "$INSTALL_DIR"
    URL="${RAKSHA_PORTAL}/api/v1/agent/download/${OS}/${ARCH}"
    FALLBACK="https://github.com/dansiapa/raksha-security-platform/releases/latest/download/raksha-agent-${OS}-${ARCH}"
    curl -fsSL -H "Authorization: Bearer ${RAKSHA_TOKEN}" -o "${INSTALL_DIR}/raksha-agent" "$URL" 2>/dev/null \
        || curl -fsSL -o "${INSTALL_DIR}/raksha-agent" "$FALLBACK" \
        || error "Failed to download agent"
    chmod +x "${INSTALL_DIR}/raksha-agent"
    info "Binary installed at ${INSTALL_DIR}/raksha-agent"
}

enroll_agent() {
    PAYLOAD="{\"token\":\"${RAKSHA_TOKEN}\",\"fingerprint\":{\"hostname\":\"${HOSTNAME}\",\"os\":\"${OS}\",\"os_version\":\"${OS_VERSION}\",\"arch\":\"${ARCH}\",\"machine_id\":\"${MACHINE_ID}\",\"cpu_cores\":${CPU_CORES},\"total_memory\":${TOTAL_MEM},\"mac_hash\":\"${MAC_HASH}\"}}"
    RESP=$(curl -sS -X POST -H "Content-Type: application/json" -d "$PAYLOAD" "${RAKSHA_PORTAL}/api/v1/agents/enroll")
    echo "$RESP" | grep -q '"error"' && error "Enrollment failed: $RESP"
    AGENT_ID=$(echo "$RESP" | grep -o '"agent_id":"[^"]*"' | cut -d'"' -f4)
    [ -z "$AGENT_ID" ] && error "Failed to parse enrollment response"
    mkdir -p "$CONFIG_DIR" "$LOG_DIR"
    cat > "${CONFIG_DIR}/agent.toml" <<EOF
[agent]
id = "${AGENT_ID}"
portal_url = "${RAKSHA_PORTAL}"
[security]
tls_verify = true
[reporting]
interval_secs = 30
heartbeat_secs = 10
[modules]
enabled = ["server", "network", "process"]
[logging]
level = "info"
file = "${LOG_DIR}/agent.log"
EOF
    chmod 600 "${CONFIG_DIR}/agent.toml"
    info "Agent enrolled: ${AGENT_ID}"
}

install_service() {
    cat > /etc/systemd/system/${SERVICE_NAME}.service <<EOF
[Unit]
Description=Raksha Security Agent
After=network-online.target
Wants=network-online.target
[Service]
Type=simple
ExecStart=${INSTALL_DIR}/raksha-agent --config ${CONFIG_DIR}/agent.toml
Restart=always
RestartSec=10
LimitNOFILE=65536
ProtectSystem=strict
ReadWritePaths=${LOG_DIR} ${CONFIG_DIR}
NoNewPrivileges=true
[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload && systemctl enable --now "$SERVICE_NAME"
    info "Service installed and started"
}

main() {
    echo -e "${BLUE}  🔱 Raksha Agent Installer${NC}"
    check_prerequisites
    detect_system
    collect_fingerprint
    download_agent
    enroll_agent
    install_service
    echo -e "\n${GREEN}✅ Raksha Agent installed!${NC}"
    echo "  Agent: ${AGENT_ID} | Portal: ${RAKSHA_PORTAL}"
    echo "  Config: ${CONFIG_DIR}/agent.toml | Logs: ${LOG_DIR}/agent.log"
    echo "  Status: systemctl status ${SERVICE_NAME}"
}

main "$@"
