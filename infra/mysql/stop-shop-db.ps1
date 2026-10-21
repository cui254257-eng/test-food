$ErrorActionPreference = 'Stop'

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$serverBin = if ($env:MYSQL_SERVER_BIN) {
  $env:MYSQL_SERVER_BIN
} else {
  'C:\Program Files\MySQL\MySQL Server 9.7\bin'
}
$adminExe = Join-Path $serverBin 'mysqladmin.exe'
$listener = Get-NetTCPConnection -State Listen -LocalPort 3307 -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $listener) {
  Write-Output 'MySQL shop is already stopped.'
  exit 0
}

$process = Get-CimInstance Win32_Process -Filter "ProcessId=$($listener.OwningProcess)"
$dataDir = Join-Path $projectRoot 'data\mysql-shop'
if ($process.Name -ne 'mysqld.exe' -or $process.CommandLine -notlike "*$dataDir*") {
  throw 'Port 3307 is not owned by this project MySQL instance; leaving it untouched.'
}

$env:MYSQL_PWD = 'root'
try {
  & $adminExe --protocol=TCP --host=127.0.0.1 --port=3307 --user=root shutdown
} finally {
  Remove-Item Env:\MYSQL_PWD -ErrorAction SilentlyContinue
}
if ($LASTEXITCODE -ne 0) { throw "Could not stop MySQL (mysqladmin exited with code $LASTEXITCODE)." }
Write-Output 'MySQL shop has stopped.'
