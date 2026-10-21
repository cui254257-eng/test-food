$ErrorActionPreference = 'Stop'

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$dataDir = Join-Path $projectRoot 'data\mysql-shop'
$serverBin = if ($env:MYSQL_SERVER_BIN) {
  $env:MYSQL_SERVER_BIN
} else {
  'C:\Program Files\MySQL\MySQL Server 9.7\bin'
}
$serverExe = Join-Path $serverBin 'mysqld.exe'
$clientExe = Join-Path $serverBin 'mysql.exe'

if (-not (Test-Path -LiteralPath $serverExe)) {
  throw "MySQL Server not found at $serverExe. Set MYSQL_SERVER_BIN to its bin directory."
}

$listener = Get-NetTCPConnection -State Listen -LocalPort 3307 -ErrorAction SilentlyContinue | Select-Object -First 1
if ($listener) {
  $process = Get-CimInstance Win32_Process -Filter "ProcessId=$($listener.OwningProcess)"
  if ($process.Name -ne 'mysqld.exe' -or $process.CommandLine -notlike "*$(Join-Path $projectRoot 'data\mysql-shop')*") {
    throw 'Port 3307 is already used by another process or MySQL instance.'
  }
  Write-Output 'MySQL shop is already listening on 127.0.0.1:3307.'
  exit 0
}

$initialize = -not (Test-Path -LiteralPath $dataDir)
if ($initialize) {
  New-Item -ItemType Directory -Path $dataDir | Out-Null
  & $serverExe --no-defaults --initialize-insecure "--basedir=$((Split-Path $serverBin -Parent))" "--datadir=$dataDir" --console
  if ($LASTEXITCODE -ne 0) { throw "MySQL initialization failed with code $LASTEXITCODE." }
} elseif (-not (Test-Path -LiteralPath (Join-Path $dataDir 'ibdata1'))) {
  throw "The MySQL data directory exists but is not initialized: $dataDir"
}

$baseDir = Split-Path $serverBin -Parent
$arguments = "--no-defaults --console --mysqlx=0 --bind-address=127.0.0.1 --port=3307 --basedir=`"$baseDir`" --datadir=`"$dataDir`" --log-error=`"$(Join-Path $dataDir 'shop.err')`""
Start-Process -FilePath $serverExe -ArgumentList $arguments -WindowStyle Hidden | Out-Null

$deadline = (Get-Date).AddSeconds(45)
do {
  Start-Sleep -Milliseconds 500
  $listener = Get-NetTCPConnection -State Listen -LocalPort 3307 -ErrorAction SilentlyContinue | Select-Object -First 1
} until ($listener -or (Get-Date) -gt $deadline)
if (-not $listener) { throw "MySQL did not start. Check $(Join-Path $dataDir 'shop.err')." }

if ($initialize) {
  & $clientExe --protocol=TCP --host=127.0.0.1 --port=3307 --user=root --execute="ALTER USER 'root'@'localhost' IDENTIFIED BY 'root'; CREATE DATABASE IF NOT EXISTS shop CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;"
} else {
  $env:MYSQL_PWD = 'root'
  try {
    & $clientExe --protocol=TCP --host=127.0.0.1 --port=3307 --user=root --execute="CREATE DATABASE IF NOT EXISTS shop CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;"
  } finally {
    Remove-Item Env:\MYSQL_PWD -ErrorAction SilentlyContinue
  }
}
if ($LASTEXITCODE -ne 0) { throw "Could not create the shop database (mysql exited with code $LASTEXITCODE)." }

Write-Output 'MySQL shop is ready at 127.0.0.1:3307/shop.'
