# Build logo.ico and version.rc to rc file with gcc toolchain
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = "./res"
$logoPath = Join-Path -Path $scriptDir -ChildPath "logo.ico"
$rcPath = Join-Path -Path $scriptDir -ChildPath "version.rc"
$outPath = Join-Path -Path $scriptDir -ChildPath "resources.res"

foreach ($path in @($logoPath, $rcPath)) {
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Missing: $path"
    }
}

$windres = Get-Command -Name 'windres.exe' -ErrorAction Stop

$arguments = @(
    '--input', $rcPath,
    '--output', $outPath,
    '--include-dir', $scriptDir
)

$process = Start-Process -FilePath $windres.Source -ArgumentList $arguments -NoNewWindow -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "windres exits with: $($process.ExitCode)"
}

Write-Host "Success: $outPath"