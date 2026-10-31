<#
.SYNOPSIS
  禁令扫描：只看源码里的禁用 API 与越界调用（contracts/architecture-boundaries.md §3.2）。

.DESCRIPTION
  扫描 crates/ 与 apps/desktop/src-tauri/src 下的 .rs，以及（若存在）apps/ui/src 下的 .ts/.tsx：
  - BAN-05：仅 apps/desktop 可用 tauri、可用 emit
  - BAN-03：仅 crates/storage 可用 rusqlite / 拼 SQL
  - BAN-04：仅 crates/index 可用 qdrant_client / tantivy
  - BAN-13：仅 crates/pipeline 可用 ort/onnxruntime
  - BAN-08/01：仅 apps/ui/src/lib/tauri-api 可用 invoke；UI 不得 fetch 外部服务
  注释行（// 与 /*）不参与匹配。

.EXAMPLE
  pwsh scripts/check-boundaries.ps1
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

$RustRules = @(
    @{ Token = '(\buse\s+tauri\b|\btauri::)'; Allowed = @('perknow-desktop'); Ban = 'BAN-05'; Hint = 'L1-L3 不得依赖 tauri，事件走 EventSink::emit' },
    @{ Token = '\.emit\s*\('; Allowed = @('perknow-desktop'); Ban = 'BAN-05'; Hint = '事件唯一 emit 点在外壳的 EventSink 实现' },
    @{ Token = '\brusqlite\b'; Allowed = @('perknow-storage'); Ban = 'BAN-03'; Hint = '唯一 SQL 出口是 crates/storage' },
    @{ Token = '\bqdrant_client::'; Allowed = @('perknow-index'); Ban = 'BAN-04'; Hint = '唯一 Qdrant 出口是 crates/index' },
    @{ Token = '\btantivy::'; Allowed = @('perknow-index'); Ban = 'BAN-04'; Hint = '唯一 Tantivy 出口是 crates/index' },
    @{ Token = '\b(ort|onnxruntime)::'; Allowed = @('perknow-pipeline'); Ban = 'BAN-13'; Hint = '推理只在 crates/pipeline' },
    @{ Token = '\b(SELECT\s+|INSERT\s+INTO\s+|DELETE\s+FROM\s+)'; Allowed = @('perknow-storage'); Ban = 'BAN-03'; Hint = '禁止在 storage 之外拼 SQL' }
)

function Get-CrateOf([string]$Path) {
    $normalized = $Path -replace '\\', '/'
    $m = [regex]::Match($normalized, '/crates/([^/]+)/')
    if ($m.Success) {
        return "perknow-$($m.Groups[1].Value)"
    }
    if ($normalized -match '/apps/desktop/src-tauri/') {
        return 'perknow-desktop'
    }
    return ''
}

function Test-IsComment([string]$Line) {
    $trimmed = $Line.TrimStart()
    return ($trimmed.StartsWith('//') -or $trimmed.StartsWith('*') -or $trimmed.StartsWith('/*'))
}

function Get-RelPath([string]$Path) {
    return $Path.Substring($RepoRoot.Length).TrimStart([char[]]@('\', '/'))
}

Write-Host '== check-boundaries =='

$rustFiles = @()
foreach ($root in @((Join-Path $RepoRoot 'crates'), (Join-Path $RepoRoot 'apps/desktop/src-tauri/src'))) {
    if (Test-Path -LiteralPath $root) {
        $rustFiles += Get-ChildItem -LiteralPath $root -Recurse -Filter '*.rs' -File
    }
}

$scanned = 0
foreach ($file in $rustFiles) {
    $crate = Get-CrateOf $file.FullName
    if ([string]::IsNullOrEmpty($crate)) {
        continue
    }
    $scanned++
    $lineNo = 0
    foreach ($line in Get-Content -LiteralPath $file.FullName -Encoding UTF8) {
        $lineNo++
        if (Test-IsComment $line) {
            continue
        }
        foreach ($rule in $RustRules) {
            if ($rule.Allowed -contains $crate) {
                continue
            }
            if ($line -match $rule.Token) {
                $where = (Get-RelPath $file.FullName) + ':' + $lineNo
                Add-Failure ($rule.Ban + ' ' + $where + ' 出现禁用用法（' + $crate + '）：' + $rule.Hint)
            }
        }
    }
}
Write-Host "[info] 扫描 Rust 文件：$scanned"

# 前端：只有 lib/tauri-api 允许 invoke；不得 fetch 外部服务
$uiSrc = Join-Path $RepoRoot 'apps/ui/src'
if (Test-Path -LiteralPath $uiSrc) {
    $tsFiles = @(Get-ChildItem -LiteralPath $uiSrc -Recurse -Include '*.ts', '*.tsx' -File)
    foreach ($file in $tsFiles) {
        $normalized = ($file.FullName -replace '\\', '/')
        $lineNo = 0
        foreach ($line in Get-Content -LiteralPath $file.FullName -Encoding UTF8) {
            $lineNo++
            if (Test-IsComment $line) {
                continue
            }
            $rel = (Get-RelPath $file.FullName) + ':' + $lineNo
            if (($line -match '\binvoke\s*\(') -and ($normalized -notmatch '/lib/tauri-api/')) {
                Add-Failure ('BAN-08 ' + $rel + ' 只能通过 lib/tauri-api 调用命令')
            }
            if ($line -match '\bfetch\s*\(') {
                Add-Failure ('BAN-02 ' + $rel + ' 前端不得直接访问外部服务')
            }
        }
    }
    Write-Host "[info] 扫描 TS/TSX 文件：$($tsFiles.Count)"
}
else {
    Write-Host '[skip] apps/ui 尚未建立（D 的 PD-02），跳过前端扫描'
}

Write-Host ''
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) {
        Write-Host "[FAIL] $f" -ForegroundColor Red
    }
    Write-Host "check-boundaries: $($Failures.Count) 项失败"
    exit 1
}
Write-Host 'check-boundaries: 未发现越界用法'
exit 0
