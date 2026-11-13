# Isolated PostgreSQL logical restore drill; requires PostgreSQL binaries on PATH.
# Retains its temporary directory as evidence; never connects to an existing cluster.
param([ValidateRange(1024,65535)][int]$Port = 55484)
$ErrorActionPreference = 'Stop'
$drillRoot = Join-Path ([IO.Path]::GetTempPath()) ('toinoi-sys14-' + [guid]::NewGuid().ToString('N'))
$dataPath = Join-Path $drillRoot 'data'
$dumpPath = Join-Path $drillRoot 'probe.dump'
$logPath = Join-Path $drillRoot 'postgres.log'
$started = $false
New-Item -ItemType Directory -Path $drillRoot | Out-Null
function Invoke-Pg([string]$Binary, [string[]]$Arguments) {
    $result = & $Binary @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) { throw "$Binary failed: $result" }
    return $result
}
try {
    Invoke-Pg 'initdb' @('-D', $dataPath, '-U', 'drill', '--auth=trust', '--encoding=UTF8', '--no-locale') | Out-Null
    # Trust authentication is confined to this short-lived loopback cluster of synthetic data.
    # File redirection avoids inherited PowerShell capture pipes keeping pg_ctl open on Windows.
    $ctlArgs = @('-D', ('"' + $dataPath + '"'), '-l', ('"' + $logPath + '"'),
        '-o', ('"-h 127.0.0.1 -p ' + $Port + '"'), '-w', 'start')
    $ctl = Start-Process -FilePath (Get-Command pg_ctl).Source -ArgumentList $ctlArgs -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $drillRoot 'start.out') -RedirectStandardError (Join-Path $drillRoot 'start.err')
    # Wait only for pg_ctl, not its long-lived PostgreSQL descendant tree.
    $ctl.WaitForExit()
    if ($ctl.ExitCode -ne 0) { throw "pg_ctl start failed; inspect $drillRoot" }
    $started = $true
    $conn = @('-h', '127.0.0.1', '-p', "$Port", '-U', 'drill')
    Invoke-Pg 'createdb' ($conn + @('-T', 'template0', 'source_drill')) | Out-Null
    $sql = "CREATE TABLE recovery_probe(id integer PRIMARY KEY, amount integer NOT NULL, committed_at timestamptz NOT NULL DEFAULT clock_timestamp()); INSERT INTO recovery_probe(id,amount) VALUES (1,35000);"
    Invoke-Pg 'psql' ($conn + @('-X', '-v', 'ON_ERROR_STOP=1', '-d', 'source_drill', '-c', $sql)) | Out-Null
    Invoke-Pg 'pg_dump' ($conn + @('-Fc', '-f', $dumpPath, 'source_drill')) | Out-Null
    Invoke-Pg 'psql' ($conn + @('-X', '-v', 'ON_ERROR_STOP=1', '-d', 'source_drill', '-c', 'INSERT INTO recovery_probe(id,amount) VALUES (2,42000);')) | Out-Null
    $sourceCount = Invoke-Pg 'psql' ($conn + @('-X', '-At', '-d', 'source_drill', '-c', 'SELECT count(*) FROM recovery_probe;'))
    if ("$sourceCount".Trim() -ne '2') { throw 'Source baseline should contain two committed markers' }
    # Simulated accidental deletion, only in the freshly-created source database.
    Invoke-Pg 'psql' ($conn + @('-X', '-v', 'ON_ERROR_STOP=1', '-d', 'source_drill', '-c', 'DELETE FROM recovery_probe;')) | Out-Null
    $timer = [Diagnostics.Stopwatch]::StartNew()
    Invoke-Pg 'createdb' ($conn + @('-T', 'template0', 'restored_drill')) | Out-Null
    Invoke-Pg 'pg_restore' ($conn + @('--exit-on-error', '--single-transaction', '-d', 'restored_drill', $dumpPath)) | Out-Null
    $actual = Invoke-Pg 'psql' ($conn + @('-X', '-At', '-v', 'ON_ERROR_STOP=1', '-d', 'restored_drill', '-c', "SELECT id::text || ':' || amount::text FROM recovery_probe ORDER BY id;"))
    if ("$actual".Trim() -ne '1:35000') { throw "Unexpected restored markers: $actual" }
    Invoke-Pg 'psql' ($conn + @('-X', '-v', 'ON_ERROR_STOP=1', '-d', 'restored_drill', '-c', 'INSERT INTO recovery_probe(id,amount) VALUES (3,15000);')) | Out-Null
    $count = Invoke-Pg 'psql' ($conn + @('-X', '-At', '-d', 'restored_drill', '-c', 'SELECT count(*) FROM recovery_probe;'))
    if ("$count".Trim() -ne '2') { throw 'Post-restore write check failed' }
    $timer.Stop()
    $report = [ordered]@{
        postgres = "$(Invoke-Pg 'pg_dump' @('--version'))"
        recoveredMarker = 1; missingPostBackupMarker = 2; newWriteMarker = 3
        restoreAndValidationMs = $timer.ElapsedMilliseconds
        applicationRtoMeasured = $false; rpoTimeMeasured = $false
        evidenceDirectory = $drillRoot
    }
    $report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $drillRoot 'result.json') -Encoding utf8
    $report | ConvertTo-Json
} finally {
    if ($started) { Invoke-Pg 'pg_ctl' @('-D', $dataPath, '-m', 'fast', '-w', 'stop') | Out-Null }
}
