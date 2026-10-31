<#
.SYNOPSIS
  依赖方向校验：解析各 Cargo.toml 的声明，对照 contracts/architecture-boundaries.md §2 的依赖矩阵。

.DESCRIPTION
  两类违规：
  1. 内部依赖越界（例如 index 依赖 storage、core 依赖 kb）；
  2. 外部引擎库出现在不允许的 crate（例如 search 依赖 tantivy）。
  新增 workspace crate 必须同时更新本脚本与 docs/project-structure.md §2。

  为什么直读 Cargo.toml 而不调 cargo metadata：边界检查关心的是“声明”，直读可以在无网络、
  无 cargo 的环境跑（离线评审机）；编译期的最终兜底是 CI 的 cargo build/clippy 作业。

.EXAMPLE
  pwsh scripts/check-deps.ps1
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

# 允许的内部依赖（白名单；与 architecture-boundaries.md §2 一致）
$Allowed = @{
    'perknow-contracts' = @()
    'perknow-core'      = @('perknow-contracts')
    'perknow-storage'   = @('perknow-contracts', 'perknow-core')
    'perknow-pipeline'  = @('perknow-contracts', 'perknow-core', 'perknow-storage')
    'perknow-index'     = @('perknow-contracts', 'perknow-core')
    'perknow-llm'       = @('perknow-contracts', 'perknow-core')
    'perknow-fs'        = @('perknow-contracts', 'perknow-core', 'perknow-storage')
    'perknow-sync'      = @('perknow-contracts', 'perknow-core', 'perknow-storage', 'perknow-pipeline', 'perknow-index', 'perknow-fs')
    'perknow-search'    = @('perknow-contracts', 'perknow-core', 'perknow-storage', 'perknow-index', 'perknow-llm')
    'perknow-history'   = @('perknow-contracts', 'perknow-core', 'perknow-storage')
    'perknow-kb'        = @('perknow-contracts', 'perknow-core', 'perknow-storage', 'perknow-pipeline', 'perknow-index', 'perknow-fs', 'perknow-settings', 'perknow-sync')
    'perknow-settings'  = @('perknow-contracts', 'perknow-core', 'perknow-storage')
    'perknow-desktop'   = @('perknow-contracts', 'perknow-core', 'perknow-storage', 'perknow-pipeline', 'perknow-index', 'perknow-llm', 'perknow-kb', 'perknow-settings', 'perknow-fs', 'perknow-sync', 'perknow-search', 'perknow-history')
}

# 禁止的外部库（引擎/框架：只能在唯一出口 crate 里出现）
$Forbidden = @{
    'perknow-contracts' = @('tauri', 'rusqlite', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime')
    'perknow-core'      = @('tauri', 'rusqlite', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime')
    'perknow-storage'   = @('tauri', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime')
    'perknow-pipeline'  = @('tauri', 'qdrant-client', 'tantivy')
    'perknow-index'     = @('tauri', 'rusqlite')
    'perknow-llm'       = @('tauri', 'rusqlite', 'qdrant-client', 'tantivy')
    'perknow-fs'        = @('tauri', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime')
    'perknow-sync'      = @('tauri', 'qdrant-client', 'tantivy')
    'perknow-search'    = @('tauri', 'qdrant-client', 'tantivy')
    'perknow-history'   = @('tauri', 'qdrant-client', 'tantivy')
    'perknow-kb'        = @('tauri', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime', 'rusqlite')
    'perknow-settings'  = @('tauri', 'qdrant-client', 'tantivy', 'ort', 'onnxruntime', 'rusqlite')
    'perknow-desktop'   = @('qdrant-client', 'rusqlite', 'tantivy', 'ort', 'onnxruntime')
}

function Get-Manifest([string]$Path) {
    $name = ''
    $deps = @{}
    $section = ''
    foreach ($line in Get-Content -LiteralPath $Path -Encoding UTF8) {
        $trim = $line.Trim()
        if ($trim.Length -eq 0) { continue }
        if ($trim.StartsWith('#')) { continue }
        if ($trim -match '^\[(.+)\]$') {
            $section = $Matches[1].Trim()
            continue
        }
        if (($section -eq 'package') -and ($trim -match '^name\s*=\s*"([^"]+)"$')) {
            $name = $Matches[1]
            continue
        }
        if (($section -eq 'dependencies') -or ($section -eq 'build-dependencies') -or ($section -eq 'dev-dependencies')) {
            if ($trim -match '^([A-Za-z0-9_-]+)\s*=') {
                $deps[$Matches[1]] = $true
            }
        }
    }
    return [pscustomobject]@{
        Name         = $name
        Dependencies = $deps
    }
}

Write-Host '== check-deps =='

$rootManifest = Join-Path $RepoRoot 'Cargo.toml'
if (-not (Test-Path -LiteralPath $rootManifest)) {
    Write-Host '[FAIL] 根 Cargo.toml 不存在'
    exit 1
}

$rootText = Get-Content -LiteralPath $rootManifest -Raw -Encoding UTF8
$membersBlock = [regex]::Match($rootText, '(?s)members\s*=\s*\[(.*?)\]')
if (-not $membersBlock.Success) {
    Write-Host '[FAIL] 根 Cargo.toml 找不到 [workspace] members'
    exit 1
}
$members = @([regex]::Matches($membersBlock.Groups[1].Value, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })

foreach ($dir in Get-ChildItem -LiteralPath (Join-Path $RepoRoot 'crates') -Directory) {
    $rel = 'crates/' + $dir.Name
    if (-not ($members -contains $rel)) {
        Add-Failure "crates/$($dir.Name) 不在根 Cargo.toml 的 members 中"
    }
}
if (-not ($members -contains 'apps/desktop/src-tauri')) {
    Add-Failure 'apps/desktop/src-tauri 不在根 Cargo.toml 的 members 中'
}

$manifests = @()
foreach ($member in $members) {
    $path = Join-Path $RepoRoot ($member + '/Cargo.toml')
    if (-not (Test-Path -LiteralPath $path)) {
        Add-Failure "members 里的 $member 找不到 Cargo.toml"
        continue
    }
    $manifests += [pscustomobject]@{ Member = $member; Manifest = (Get-Manifest $path) }
}

foreach ($entry in $manifests) {
    $name = $entry.Manifest.Name
    if ([string]::IsNullOrEmpty($name)) {
        Add-Failure "$($entry.Member)/Cargo.toml 缺少 [package] name"
        continue
    }
    if (-not $Allowed.ContainsKey($name)) {
        Add-Failure "未登记的 workspace 包：$name（新增 crate 需同步本脚本、project-structure.md §2、architecture-boundaries.md §2）"
        continue
    }
    foreach ($dep in $entry.Manifest.Dependencies.Keys) {
        if ($dep.StartsWith('perknow-')) {
            if (-not ($Allowed[$name] -contains $dep)) {
                Add-Failure "$name 依赖了不允许的内部 crate：$dep"
            }
        }
        elseif ($Forbidden[$name] -contains $dep) {
            Add-Failure "$name 依赖了禁用外部库：$dep（见 architecture-boundaries.md §2/§3.2）"
        }
    }
}

Write-Host "[info] 检查 workspace 包：$($manifests.Count)"
Write-Host ''
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) {
        Write-Host "[FAIL] $f" -ForegroundColor Red
    }
    Write-Host "check-deps: $($Failures.Count) 项失败"
    exit 1
}
Write-Host 'check-deps: 依赖方向全部合规'
exit 0
