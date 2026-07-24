#Requires -RunAsAdministrator
# Raksha Security Platform - Agent Installer (Windows)
param(
    [string]$Version = "latest",
    [string]$PortalUrl = "http://localhost:8080",
    [string]$AgentKey = "",
    [string]$InstallDir = "C:\Program Files\Raksha\Agent",
    [string]$ConfigDir = "C:\ProgramData\Raksha\Agent",
    [string]$LogDir = "C:\ProgramData\Raksha\Agent\logs"
)

$ErrorActionPreference = "Stop"
$DownloadBase = "https://github.com/raksha-security/raksha-platform/releases/download"

function Write-Step($msg) { Write-Host "[*] $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "[+] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "[-] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  Raksha Security Platform - Agent Installer (Windows)" -ForegroundColor White
Write-Host "  ====================================================" -ForegroundColor White
Write-Host ""

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { Write-Err "32-bit not supported" }
Write-Step "Detected: windows-$arch"

# Create directories
Write-Step "Creating directories..."
@($InstallDir, "$InstallDir\bin", $ConfigDir, $LogDir) | ForEach-Object {
    if (-not (Test-Path $_)) { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
}
Write-Ok "Directories created"

# Download agent
$url = "$DownloadBase/v$Version/raksha-agent-windows-$arch.zip"
$tmp = "$env:TEMP\raksha-agent.zip"
Write-Step "Downloading agent v$Version..."
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
} catch {
    Write-Err "Download failed: $_"
}
Expand-Archive -Path $tmp -DestinationPath "$InstallDir\bin" -Force
Remove-Item $tmp -Force
Write-Ok "Agent binary installed"

# Generate config
$configFile = "$ConfigDir\agent.toml"
if (-not (Test-Path $configFile)) {
    Write-Step "Generating config..."
    $agentId = [guid]::NewGuid().ToString()
    $hostname = $env:COMPUTERNAME
    $config = @"
[agent]
id = "$agentId"
hostname = "$hostname"

[portal]
url = "$PortalUrl"
api_key = "$AgentKey"
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
file = "C:/ProgramData/Raksha/Agent/logs/agent.log"
"@
    Set-Content -Path $configFile -Value $config -Encoding UTF8
    Write-Ok "Config generated at $configFile"
} else {
    Write-Warn "Config already exists, skipping"
}

# Install Windows Service
Write-Step "Installing Windows service..."
$svcName = "RakshaAgent"
$binPath = "$InstallDir\bin\raksha-agent.exe --config $configFile"
if (Get-Service -Name $svcName -ErrorAction SilentlyContinue) {
    Write-Warn "Service exists, updating..."
    Stop-Service $svcName -Force -ErrorAction SilentlyContinue
    sc.exe delete $svcName | Out-Null
    Start-Sleep -Seconds 2
}
New-Service -Name $svcName `
    -BinaryPathName $binPath `
    -DisplayName "Raksha Security Agent" `
    -Description "Raksha host monitoring and security agent" `
    -StartupType Automatic | Out-Null

# Set service recovery options (restart on failure)
sc.exe failure $svcName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null
Write-Ok "Windows service installed with auto-recovery"

# Add firewall rule for outbound (agent -> portal)
if (-not (Get-NetFirewallRule -DisplayName "Raksha Agent Outbound" -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -DisplayName "Raksha Agent Outbound" -Direction Outbound `
        -Program "$InstallDir\bin\raksha-agent.exe" -Action Allow | Out-Null
    Write-Ok "Firewall rule added"
}

# Add to PATH
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($machinePath -notlike "*$InstallDir\bin*") {
    [Environment]::SetEnvironmentVariable("Path", "$machinePath;$InstallDir\bin", "Machine")
    Write-Ok "Added to system PATH"
}

Write-Host ""
Write-Ok "Raksha Agent installed successfully!"
Write-Ok "Binary:  $InstallDir\bin\raksha-agent.exe"
Write-Ok "Config:  $configFile"
Write-Ok "Start:   Start-Service RakshaAgent"
Write-Ok "Status:  Get-Service RakshaAgent"
Write-Ok "Logs:    $LogDir\agent.log"
