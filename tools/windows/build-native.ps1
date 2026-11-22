# Build gdl_play natively on Windows (this fork, 2026-11-22; docs/toolchain.md
# "Native Windows build"): Rust's x86_64-pc-windows-gnu host (rustup) with
# MSYS2's MINGW64 GCC, the target upstream cross-builds. No Visual Studio.
#
#   powershell -ExecutionPolicy Bypass -File tools\windows\build-native.ps1 [-Tests] [-Msys C:\msys64]
#
# Checks the HD upscaler weights first (git-ignored; include_bytes! needs
# them; docs/remaster/HANDOFF.md section 5). Builds target\release\gdl_play.exe;
# -Tests also runs the gdl_core tests and the gdl_client library tests.
param(
    [switch] $Tests,
    [string] $Msys = 'C:\msys64'
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$weights = Join-Path $root 'crates\gdl_client\assets\hd\realesr-general-x4v3-dn05.bin'
$want = 'e6626f1f5a94c46423826e44e60a6efe4b92d918aee97877c246fe7c2259ef4b'
if (-not (Test-Path $weights)) { throw "missing $weights (docs/remaster/HANDOFF.md section 5)" }
$got = (Get-FileHash $weights -Algorithm SHA256).Hash.ToLower()
if ($got -ne $want) { throw "weights SHA-256 is $got, want $want" }
$gcc = Join-Path $Msys 'mingw64\bin'
if (-not (Test-Path (Join-Path $gcc 'gcc.exe'))) { throw "no MINGW64 gcc in $gcc (MSYS2: pacman -S mingw-w64-x86_64-gcc)" }
# This process only: the MINGW64 tools (gcc for blake3's assembly, the
# linker) and rustup's cargo. rust-toolchain.toml selects 1.93.1.
$env:PATH = "$gcc;$env:USERPROFILE\.cargo\bin;$env:PATH"
Set-Location $root
function Invoke-Cargo([string[]] $CargoArgs) {
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo $($CargoArgs -join ' ') failed ($LASTEXITCODE)" }
}
Invoke-Cargo @('build', '--release', '--locked', '-p', 'gdl_client', '--bin', 'gdl_play')
$exe = Join-Path $root 'target\release\gdl_play.exe'
"built: $exe ($((Get-Item $exe).Length) bytes)"
if ($Tests) {
    Invoke-Cargo @('test', '--locked', '-p', 'gdl_core')
    Invoke-Cargo @('test', '--locked', '-p', 'gdl_client', '--lib')
}
