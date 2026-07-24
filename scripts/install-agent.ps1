# Raksha Agent Installer for Windows
# Usage: $env:RAKSHA_TOKEN="rkat_xxx"; $env:RAKSHA_PORTAL="https://portal"; irm https://portal/api/v1/agent/install.ps1 | iex
#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"
$InstallDir = "C:\Program Files\Raksha Agent"
$ConfigDir = "C:\ProgramData\Raksha"
$LogDir = "C:\ProgramData\Raksha\logs"
$ServiceName = "RakshaAgent"

function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Green }
function Write-Err($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

# Preflight
if (-not $env:RAKSHA_TOKEN) { Write-Err "RAKSHA_TOKEN env var required" }
if (-not $env:RAKSHA_PORTAL) { Write-Err "RAKSHA_PORTAL env var required" }
if (-not $env:RAKSHA_TOKEN.StartsWith("rkat_")) { Write-Err "Invalid token format" }
Write-Info "Preflight checks passed"

# Detect
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-Info "Detected: windows/$Arch"

# Fingerprint
$Hostname = $env:COMPUTERNAME
$OsVersion = (Get-CimInstance Win32_OperatingSystem).Version
$CpuCores = (Get-CimInstance Win32_Processor).NumberOfCores
$TotalMem = (Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory
$MachineId = (Get-CimInstance Win32_ComputerSystemProduct).UUID
$MacRaw = (Get-NetAdapter | Where-Object Status -eq "Up" | Select-Object -First 1).MacAddress
$MacHash = [BitConverter]::ToString(
    [System.Security.Cryptography.SHA256]::Create().ComputeHash(
        [System.Text.Encoding]::UTF8.GetBytes($MacRaw)
    )
).Replace("-","").ToLower()
Write-Info "Fingerprint collected"

# Download
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$BinaryPath = Join-Path $InstallDir "raksha-agent.exe"
$DownloadUrl = "$env:RAKSHA_PORTAL/api/v1/agent/download/windows/$Arch"
$FallbackUrl = "https://github.com/dansiapa/raksha-security-platform/releases/latest/download/raksha-agent-windows-$Arch.exe"
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $BinaryPath -Headers @{Authorization="Bearer $env:RAKSHA_TOKEN"}
} catch {
    try { Invoke-WebRequest -Uri $FallbackUrl -OutFile $BinaryPath }
    catch { Write-Err "Failed to download agent binary" }
}
Write-Info "Binary installed at $BinaryPath"

# Enroll
$Body = @{
    token = $env:RAKSHA_TOKEN
    fingerprint = @{
        hostname = $Hostname
        os = "windows"
        os_version = $OsVersion
        arch = $Arch
        machine_id = $MachineId
        cpu_cores = [int]$CpuCores
        total_memory = [long]$TotalMem
        mac_hash = $MacHash
    }
} | ConvertTo-Json -Depth 3

$Response = Invoke-RestMethod -Method Post -Uri "$env:RAKSHA_PORTAL/api/v1/agents/enroll" `
    -ContentType "application/json" -Body $Body
if ($Response.error) { Write-Err "Enrollment failed: $($Response.message)" }
$AgentId = $Response.agent_id
Write-Info "Agent enrolled: $AgentId"

# Config
New-Item -ItemType Directory -Force -Path $ConfigDir,$LogDir | Out-Null
$Config = @"
[agent]
id = "$AgentId"
portal_url = "$env:RAKSHA_PORTAL"
[security]
tls_verify = true
[reporting]
interval_secs = 30
heartbeat_secs = 10
[modules]
enabled = ["server", "network", "process"]
[logging]
level = "info"
file = "$LogDir\\agent.log"
"@
Set-Content -Path "$ConfigDir\agent.toml" -Value $Config
Write-Info "Config saved to $ConfigDir\agent.toml"

# Install Windows Service
$svcParams = @{
    Name = $ServiceName
    BinaryPathName = "`"$BinaryPath`" --config `"$ConfigDir\agent.toml`""
    DisplayName = "Raksha Security Agent"
    Description = "Infrastructure security monitoring agent"
    StartupType = "Automatic"
}
New-Service @svcParams | Out-Null
Start-Service $ServiceName
Write-Info "Service installed and started"

Write-Host "`n✅ Raksha Agent installed!" -ForegroundColor Green
Write-Host "  Agent: $AgentId | Portal: $env:RAKSHA_PORTAL"
Write-Host "  Config: $ConfigDir\agent.toml"
Write-Host "  Status: Get-Service $ServiceName"
