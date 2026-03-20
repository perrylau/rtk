param(
    [string]$ToolchainRoot = "D:\Rust\.rustup\toolchains\stable-x86_64-pc-windows-gnullvm",
    [string]$LlvmBin = "D:\llvm-mingw\llvm-mingw-20260311-ucrt-x86_64\bin",
    [string]$Target = "x86_64-pc-windows-gnullvm",
    [ValidateSet("release", "debug")]
    [string]$Profile = "release",
    [string]$OutputDir = ".\dist\windows-gnullvm",
    [string]$DeployTo = "",
    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-ImportedRuntimeDlls {
    param(
        [string]$ExePath,
        [string]$LlvmBinPath
    )

    $fallback = @("libunwind.dll")
    $llvmReadobj = Join-Path $LlvmBinPath "llvm-readobj.exe"
    if (-not (Test-Path $llvmReadobj)) {
        return $fallback
    }

    $imported = @()
    $output = & $llvmReadobj --coff-imports $ExePath 2>$null
    foreach ($line in $output) {
        if ($line -match '^\s*DLL Name:\s+(.+)$') {
            $dllName = $Matches[1].Trim()
            if (Test-Path (Join-Path $LlvmBinPath $dllName)) {
                $imported += $dllName
            }
        }
    }

    if ($imported.Count -eq 0) {
        return @($fallback)
    }

    return @($imported | Sort-Object -Unique)
}

$cargo = Join-Path $ToolchainRoot "bin\cargo.exe"
$rustc = Join-Path $ToolchainRoot "bin\rustc.exe"
$rustdoc = Join-Path $ToolchainRoot "bin\rustdoc.exe"
$linker = Join-Path $LlvmBin "x86_64-w64-mingw32-clang.exe"
$archiveTool = Join-Path $LlvmBin "llvm-ar.exe"

if (-not (Test-Path $cargo)) {
    throw "cargo not found: $cargo"
}
if (-not (Test-Path $rustc)) {
    throw "rustc not found: $rustc"
}
if (-not (Test-Path $rustdoc)) {
    throw "rustdoc not found: $rustdoc"
}
if (-not (Test-Path $linker)) {
    throw "linker not found: $linker"
}
if (-not (Test-Path $archiveTool)) {
    throw "archive tool not found: $archiveTool"
}

$outputRoot = (Resolve-Path -LiteralPath ".").Path
$packageDir = [System.IO.Path]::GetFullPath((Join-Path $outputRoot $OutputDir))
New-Item -ItemType Directory -Force $packageDir | Out-Null

$env:PATH = "$LlvmBin;D:\Rust\.cargo\bin;" + $env:PATH
$env:RUSTC = $rustc
$env:RUSTDOC = $rustdoc
$env:CC = $linker
$env:AR = $archiveTool
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER = $linker
$env:CARGO_INCREMENTAL = "0"

if (-not $SkipBuild) {
    & $cargo build --target $Target "--$Profile"
}

$exePath = Join-Path $outputRoot "target\$Target\$Profile\rtk.exe"
if (-not (Test-Path $exePath)) {
    throw "built executable not found: $exePath"
}

Copy-Item $exePath (Join-Path $packageDir "rtk.exe") -Force

$runtimeDlls = @(Get-ImportedRuntimeDlls -ExePath $exePath -LlvmBinPath $LlvmBin)
foreach ($dllName in $runtimeDlls) {
    Copy-Item (Join-Path $LlvmBin $dllName) (Join-Path $packageDir $dllName) -Force
}

if ($DeployTo) {
    New-Item -ItemType Directory -Force $DeployTo | Out-Null
    Copy-Item (Join-Path $packageDir "*") $DeployTo -Recurse -Force
}

Write-Host "Package ready: $packageDir"
Write-Host "Executable: $(Join-Path $packageDir 'rtk.exe')"
if ($runtimeDlls.Count -gt 0) {
    Write-Host "Runtime DLLs: $($runtimeDlls -join ', ')"
}
if ($DeployTo) {
    Write-Host "Deployed to: $DeployTo"
}
