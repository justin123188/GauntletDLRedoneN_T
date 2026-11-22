# TEST harness run on Windows (this fork, 2026-11-22; docs/toolchain.md
# "Native Windows build"): gdl_play with GDL_TEST_AUTOSTART (a new default
# character, no menu time) in one level, a scripted port 0 and
# GDL_EXIT_AFTER; -Out receives the simulation state's SHA-256 after every
# stepped update, one line each. Compare two builds or two settings by
# comparing their -Out files.
#
#   powershell -ExecutionPolicy Bypass -File tools\windows\sim-hashes.ps1 -Exe DIR\gdl_play.exe -Config INSTALL\data\config.json -Out FILE [-Baseline] [-Level G1] [-Updates 600] [-Script FILE] [-Keep]
#
# APPDATA points at a new scratch folder for the game only: the harness's
# character is saved there, never in the player's saves, and the settings
# are the defaults (-Baseline: GDL_BASELINE=1, the original presentation and
# camera). No audio. The disc data is only read. The game appends a few
# lines to remaster-log.txt beside -Exe.
#
# The scripted port has no analog sticks (PadStatus analog and has_stick are
# false): only the d-pad (entries 0-3) and the buttons count, so the free
# camera's right stick cannot be scripted yet.
param(
    [Parameter(Mandatory)] [string] $Exe,
    [Parameter(Mandatory)] [string] $Config,
    [Parameter(Mandatory)] [string] $Out,
    [switch] $Baseline,
    [string] $Level = 'G1',
    [int] $Updates = 600,
    [string] $Script = (Join-Path $PSScriptRoot 'g1-walk-dpad.json'),
    [int] $TimeoutSec = 240,
    [switch] $Keep
)
$ErrorActionPreference = 'Stop'
$Exe = (Resolve-Path $Exe).Path
$Config = (Resolve-Path $Config).Path
$Script = (Resolve-Path $Script).Path
$Out = [IO.Path]::GetFullPath($Out)
$scratch = Join-Path ([IO.Path]::GetTempPath()) ("gdl-sim-hashes-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Force $scratch | Out-Null
if (Test-Path $Out) { Remove-Item -Force $Out }
$vars = [ordered]@{
    APPDATA = $scratch; GDL_NO_AUDIO = '1'; GDL_TEST_AUTOSTART = '1'; GDL_DEBUG_LEVEL = $Level
    GDL_INPUT_SCRIPT = $Script; GDL_EXIT_AFTER = "$Updates"; GDL_STATE_OUT = $Out
    GDL_BASELINE = $(if ($Baseline) { '1' } else { $null })
}
$saved = @{}
foreach ($k in $vars.Keys) { $saved[$k] = [Environment]::GetEnvironmentVariable($k, 'Process') }
try {
    # The child inherits this process's environment; restored in finally.
    foreach ($k in $vars.Keys) { [Environment]::SetEnvironmentVariable($k, $vars[$k], 'Process') }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -ArgumentList "`"$Config`"" -PassThru -WindowStyle Minimized `
        -RedirectStandardOutput (Join-Path $scratch 'stdout.txt') -RedirectStandardError (Join-Path $scratch 'stderr.txt')
    $null = $p.Handle
    if (-not $p.WaitForExit($TimeoutSec * 1000)) { $p.Kill(); throw "timed out after $TimeoutSec s (see $scratch\stderr.txt)" }
    $n = if (Test-Path $Out) { @(Get-Content $Out).Count } else { 0 }
    if ($p.ExitCode -ne 0 -or $n -ne $Updates) { $Keep = $true; throw "exit $($p.ExitCode), $n of $Updates hashes (see $scratch\stderr.txt)" }
    "{0}: {1} updates in {2}{3}, {4:N1} s; last {5}" -f $Out, $n, $Level, $(if ($Baseline) { ' (baseline)' } else { '' }), $sw.Elapsed.TotalSeconds, (Get-Content $Out -Tail 1)
} finally {
    foreach ($k in $saved.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k], 'Process') }
    if ($Keep) { "scratch kept: $scratch" } else { Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue }
}
