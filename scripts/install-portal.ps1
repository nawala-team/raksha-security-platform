#Requires -RunAsAdministrator
# Raksha Security Platform - Portal Installer (Windows)
param(
    [string]$Version = "latest",
    [string]$InstallDir = "C:\Program Files\Raksha",
    [string]$ConfigDir = "C:\ProgramData\Raksha",
    [string]$DataDir = "C:\ProgramData\Raksha\data",
    [string]$LogDir = "C:\ProgramData\Raksha\logs"
)

$ErrorActionPreference = "Stop"
$DownloadBase = "https://github.com/raksha-security/raksha-platform/releases/download"

function Write-Step($msg) { Write-Host "[*] $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "[+] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "[-] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  Raksha Security Platform - Portal Installer (Windows)" -ForegroundColor White
Write-Host "  =====================================================" -ForegroundColor White
Write-Host ""

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { Write-Err "32-bit OS not supported" }
Write-Step "Detected: windows-$arch"

# Create directories
Write-Step "Creating directories..."
@($InstallDir, "$InstallDir\bin", $ConfigDir, $DataDir, $LogDir) | ForEach-Object {
    if (-not (Test-Path $_)) { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
}
Write-Ok "Directories created"

# Download portal binary
$url = "$DownloadBase/v$Version/raksha-portal-windows-$arch.zip"
$tmp = "$env:TEMP\raksha-portal.zip"
Write-Step "Downloading Raksha Portal v$Version..."
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
} catch {
    Write-Err "Download failed: $_"
}
Expand-Archive -Path $tmp -DestinationPath "$InstallDir\bin" -Force
Remove-Item $tmp -Force
Write-Ok "Portal binary installed"

# Generate config
$configFile = "$ConfigDir\portal.toml"
if (-not (Test-Path $configFile)) {
    Write-Step "Generating config..."
    $jwtSecret = -join ((1..64) | ForEach-Object { [char](Get-Random -Minimum 33 -Maximum 126) })
    $config = @"
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://raksha:${POSTGRES_PASSWORD:-CHANGE_ME}@localhost:5432/raksha"
max_connections = 20

[redis]
url = "redis://localhost:6379"

[auth]
jwt_secret = "$jwtSecret"
token_expiry_hours = 24

[logging]
level = "info"
format = "json"
file = "C:/ProgramData/Raksha/logs/portal.log"
"@
    Set-Content -Path $configFile -Value $config -Encoding UTF8
    Write-Ok "Config generated at $configFile"
} else {
    Write-Warn "Config already exists, skipping"
}

# Install Windows Service
Write-Step "Installing Windows service..."
$svcName = "RakshaPortal"
$binPath = "$InstallDir\bin\raksha-portal.exe --config $configFile"
if (Get-Service -Name $svcName -ErrorAction SilentlyContinue) {
    Write-Warn "Service already exists, updating..."
    Stop-Service $svcName -Force -ErrorAction SilentlyContinue
    sc.exe delete $svcName | Out-Null
    Start-Sleep -Seconds 2
}
New-Service -Name $svcName `
    -BinaryPathName $binPath `
    -DisplayName "Raksha Security Portal" `
    -Description "Raksha Security Platform API Server" `
    -StartupType Automatic | Out-Null
Write-Ok "Windows service installed"

# Add to PATH
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($machinePath -notlike "*$InstallDir\bin*") {
    [Environment]::SetEnvironmentVariable("Path", "$machinePath;$InstallDir\bin", "Machine")
    Write-Ok "Added to system PATH"
}

# Firewall rule
if (-not (Get-NetFirewallRule -DisplayName "Raksha Portal" -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -DisplayName "Raksha Portal" -Direction Inbound `
        -Protocol TCP -LocalPort 8080 -Action Allow | Out-Null
    Write-Ok "Firewall rule added (port 8080)"
}

Write-Host ""
Write-Ok "Raksha Portal installed successfully!"
Write-Ok "Binary:  $InstallDir\bin\raksha-portal.exe"
Write-Ok "Config:  $configFile"
Write-Ok "Logs:    $LogDir"
Write-Ok "Start:   Start-Service RakshaPortal"
Write-Ok "Status:  Get-Service RakshaPortal"
