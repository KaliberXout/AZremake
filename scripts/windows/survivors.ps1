<#
.SYNOPSIS
    Build, test and run Warcraft Survivors natively on Windows 11 x64.

.DESCRIPTION
    Actions:
      Check  Verify the toolchain (rustup/cargo, MSVC, WoW data) and run `cargo check`.
      Test   Run the survivors unit tests, then the no-data launch check (the game must refuse to
             start without a WoW 1.12.1 Data folder, and say why).
      Lint   rustfmt on the survivors sources, then clippy for the survivors crates only.
      Build  Build warcraft-survivors.exe (dev profile, or release with -Release).
      Run    Build and launch the game. Needs a WoW 1.12.1 Data folder (see -DataDir).

    Nothing here downloads anything. The WoW client is never fetched: point -DataDir at a
    Data folder you already own, or set WOW_DATA. The script changes no machine settings and
    writes only under the repository's target/ folder and the game's benilla-config/ folder.

.PARAMETER Action
    One of Check, Test, Lint, Build, Run. Defaults to Check.

.PARAMETER DataDir
    A WoW 1.12.1 (build 5875) Data folder that contains terrain.MPQ and patch.MPQ. Sets WOW_DATA
    for this process only.

.PARAMETER Release
    Use the release profile (what the Makefile's `native` target builds).

.EXAMPLE
    .\scripts\windows\survivors.ps1 -Action Test

.EXAMPLE
    .\scripts\windows\survivors.ps1 -Action Run -Release -DataDir "D:\WoW-1.12.1\Data"
#>
[CmdletBinding()]
param(
    [ValidateSet('Check', 'Test', 'Lint', 'Build', 'Run')]
    [string]$Action = 'Check',
    [string]$DataDir,
    [switch]$Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Resolve-Path (Join-Path (Join-Path $PSScriptRoot '..') '..')
Set-Location $repo

function Write-Step([string]$text) {
    Write-Host ''
    Write-Host "== $text" -ForegroundColor Cyan
}

function Resolve-Cargo {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        return
    }
    # A fresh rustup install is not on this shell's PATH until the shell restarts.
    $bin = Join-Path (Join-Path $env:USERPROFILE '.cargo') 'bin'
    if (Test-Path (Join-Path $bin 'cargo.exe')) {
        $env:PATH = "$bin;$env:PATH"
        return
    }
    throw 'cargo was not found. Install Rust with: winget install --id Rustlang.Rustup -e'
}

function Invoke-Cargo {
    param([string[]]$Arguments)
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

function Show-Toolchain {
    Write-Step 'Toolchain'
    Resolve-Cargo
    cargo --version
    rustc --version
    $vswhere = Join-Path (Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer') 'vswhere.exe'
    if (Test-Path $vswhere) {
        $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($vs) {
            Write-Host "MSVC build tools: $vs"
        } else {
            Write-Warning 'Visual Studio is installed but has no C++ build tools. Add the "Desktop development with C++" workload.'
        }
    } else {
        Write-Warning 'vswhere.exe not found. Install Visual Studio Build Tools with the C++ workload.'
    }
}

function Resolve-WowData {
    if ($DataDir) {
        if (-not (Test-Path $DataDir -PathType Container)) {
            throw "-DataDir '$DataDir' is not a folder."
        }
        $env:WOW_DATA = (Resolve-Path $DataDir).Path
    }
    if (-not $env:WOW_DATA) {
        $localConfig = Join-Path $repo '.env.local'
        if (Test-Path $localConfig -PathType Leaf) {
            $setting = Get-Content -LiteralPath $localConfig | Where-Object { $_ -match '^\s*WOW_DATA\s*=' } | Select-Object -First 1
            if ($setting) {
                $env:WOW_DATA = ($setting -replace '^\s*WOW_DATA\s*=\s*', '').Trim().Trim('"').Trim("'")
            }
        }
    }
    if (-not $env:WOW_DATA) {
        Write-Warning 'WOW_DATA is not set and -DataDir was not given.'
        return $false
    }
    $complete = (Test-Path (Join-Path $env:WOW_DATA 'terrain.MPQ')) -and (Test-Path (Join-Path $env:WOW_DATA 'patch.MPQ'))
    if ($complete) {
        Write-Host "WoW data: $env:WOW_DATA"
    } else {
        Write-Warning "WOW_DATA=$env:WOW_DATA does not hold terrain.MPQ and patch.MPQ."
    }
    return $complete
}

function Get-ExePath {
    $dir = if ($Release) { 'release' } else { 'debug' }
    return Join-Path (Join-Path (Join-Path $repo 'target') $dir) 'warcraft-survivors.exe'
}

# The game must refuse to start without data, and say why, before it opens a window. The dev
# binary is run with WOW_DATA empty (no install) and must exit non-zero with the launcher's message.
function Test-NoDataLaunch {
    Write-Step 'Launch refuses to start without WoW data'
    Invoke-Cargo @('build', '-p', 'warcraft-survivors')
    $exe = Join-Path (Join-Path (Join-Path $repo 'target') 'debug') 'warcraft-survivors.exe'
    $out = [System.IO.Path]::GetTempFileName()
    $err = [System.IO.Path]::GetTempFileName()
    $saved = $env:WOW_DATA
    try {
        $env:WOW_DATA = ''
        $proc = Start-Process -FilePath $exe -PassThru -Wait -NoNewWindow -RedirectStandardOutput $out -RedirectStandardError $err
        $exit = $proc.ExitCode
        $text = Get-Content -Raw -Path $err
    } finally {
        if ($null -eq $saved) {
            Remove-Item Env:WOW_DATA -ErrorAction SilentlyContinue
        } else {
            $env:WOW_DATA = $saved
        }
        Remove-Item $out, $err -ErrorAction SilentlyContinue
    }
    if ($exit -eq 0) {
        throw 'Expected a non-zero exit without WoW data, got 0.'
    }
    if ($text -notmatch 'needs a WoW 1\.12\.1 Data folder') {
        throw "Launch failed without the expected message. stderr was:`n$text"
    }
    Write-Host "OK: exit $exit with the data message." -ForegroundColor Green
}

switch ($Action) {
    'Check' {
        Show-Toolchain
        Write-Step 'cargo check (warcraft-survivors)'
        Invoke-Cargo @('check', '-p', 'warcraft-survivors')
        Write-Step 'WoW data'
        [void](Resolve-WowData)
    }
    'Test' {
        Show-Toolchain
        Write-Step 'Survivors unit tests'
        Invoke-Cargo @('test', '-p', 'benilla-app', '--lib', 'survivors')
        Test-NoDataLaunch
    }
    'Lint' {
        Show-Toolchain
        Write-Step 'rustfmt (survivors sources)'
        rustfmt --edition 2021 --check (Join-Path $repo 'crates\benilla-app\src\survivors\mod.rs')
        if ($LASTEXITCODE -ne 0) { throw 'rustfmt reports differences in the survivors sources.' }
        Write-Step 'clippy (survivors crates; dependencies are not linted)'
        Invoke-Cargo @('clippy', '--no-deps', '-p', 'benilla-app', '-p', 'warcraft-survivors', '--all-targets', '--', '-D', 'warnings')
    }
    'Build' {
        Show-Toolchain
        if ($Release) {
            Write-Step 'Build warcraft-survivors (release)'
            Invoke-Cargo @('build', '--release', '-p', 'warcraft-survivors')
        } else {
            Write-Step 'Build warcraft-survivors (dev)'
            Invoke-Cargo @('build', '-p', 'warcraft-survivors')
        }
        Write-Host "Built: $(Get-ExePath)"
    }
    'Run' {
        Show-Toolchain
        $hasData = Resolve-WowData
        if (-not $hasData) {
            Write-Warning 'Starting anyway: the game will report the missing data and exit.'
        }
        if ($Release) {
            Write-Step 'Run warcraft-survivors (release)'
            Invoke-Cargo @('run', '--release', '-p', 'warcraft-survivors')
        } else {
            Write-Step 'Run warcraft-survivors (dev)'
            Invoke-Cargo @('run', '-p', 'warcraft-survivors')
        }
    }
}
