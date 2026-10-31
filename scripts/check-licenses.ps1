<#
.SYNOPSIS
  许可证登记校验：所有声明的第三方依赖都必须出现在 LICENSES/THIRD-PARTY.md 中（BAN-20 / NFR-09）。

.DESCRIPTION
  数据源：
  - 根 Cargo.toml 的 [workspace.dependencies]；
  - 各 crate Cargo.toml 的 [dependencies] / [build-dependencies]；
  - 根与 apps/ui 的 package.json（若存在）。
  忽略 workspace 内部包（perknow-*）。

.EXAMPLE
  pwsh scripts/check-licenses.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure([string]$Message) {
    $Failures.Add($Message) | Out-Null
}

function Get-DeclaredDependencies([string]$Path) {
    $names = New-Object System.Collections.Generic.List[string]
    $section = ''
    foreach ($line in Get-Content -LiteralPath $Path -Encoding UTF8) {
        $trim = $line.Trim()
        if ($trim.Length -eq 0) { continue }
        if ($trim.StartsWith('#')) { continue }
        if ($trim -match '^\[(.+)\]$') {
            $section = $Matches[1].Trim()
            continue
        }
        $isDepSection = ($section -eq 'dependencies') -or ($section -eq 'build-dependencies') -or ($section -eq 'workspace.dependencies')
        if (-not $isDepSection) { continue }
        if ($trim -match '^([A-Za-z0-9_-]+)\s*=') {
            $name = $Matches[1]
            if (-not $name.StartsWith('perknow-')) {
                $names.Add($name)
            }
        }
    }
    return $names
}

Write-Host '== check-licenses =='

$registry = Join-Path $RepoRoot 'LICENSES/THIRD-PARTY.md'
if (-not (Test-Path -LiteralPath $registry)) {
    Write-Host '[FAIL] 缺少 LICENSES/THIRD-PARTY.md'
    exit 1
}
$registryText = Get-Content -LiteralPath $registry -Raw -Encoding UTF8

$declared = New-Object System.Collections.Generic.List[string]

$rootManifest = Join-Path $RepoRoot 'Cargo.toml'
if (Test-Path -LiteralPath $rootManifest) {
    foreach ($n in Get-DeclaredDependencies $rootManifest) { $declared.Add($n) }
}

# 只查 workspace members（直读根 Cargo.toml），避免递归扫描 target/ 之类的构建产物目录
$rootText = Get-Content -LiteralPath $rootManifest -Raw -Encoding UTF8
$membersBlock = [regex]::Match($rootText, '(?s)members\s*=\s*\[(.*?)\]')
if (-not $membersBlock.Success) {
    Write-Host '[FAIL] 根 Cargo.toml 找不到 [workspace] members'
    exit 1
}
$members = @([regex]::Matches($membersBlock.Groups[1].Value, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })
foreach ($member in $members) {
    $manifestPath = Join-Path $RepoRoot ($member + '/Cargo.toml')
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        Write-Host "[FAIL] members 里的 $member 找不到 Cargo.toml"
        exit 1
    }
    foreach ($n in Get-DeclaredDependencies $manifestPath) { $declared.Add($n) }
}

foreach ($pkg in @((Join-Path $RepoRoot 'package.json'), (Join-Path $RepoRoot 'apps/ui/package.json'))) {
    if (-not (Test-Path -LiteralPath $pkg)) { continue }
    try {
        $json = Get-Content -LiteralPath $pkg -Raw -Encoding UTF8 | ConvertFrom-Json
    }
    catch {
        Add-Failure "无法解析 $pkg"
        continue
    }
    foreach ($group in @('dependencies', 'devDependencies')) {
        $prop = $json.PSObject.Properties[$group]
        if ($null -eq $prop) { continue }
        foreach ($name in $prop.Value.PSObject.Properties.Name) {
            $declared.Add($name)
        }
    }
}

$unique = @($declared | Sort-Object -Unique)
Write-Host "[info] 已声明第三方依赖：$($unique.Count)"

foreach ($name in $unique) {
    $pattern = '(?m)^\|\s*' + [regex]::Escape($name) + '\s*\|'
    if (-not [regex]::IsMatch($registryText, $pattern)) {
        Add-Failure "LICENSES/THIRD-PARTY.md 未登记：$name"
    }
}

Write-Host ''
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) {
        Write-Host "[FAIL] $f" -ForegroundColor Red
    }
    Write-Host "check-licenses: $($Failures.Count) 项失败"
    exit 1
}
Write-Host 'check-licenses: 依赖许可清单完整'
exit 0
