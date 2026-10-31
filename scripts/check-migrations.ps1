<#
.SYNOPSIS
  迁移脚本校验：版本连续单调、空库顺序执行成功、声明与建成的表/索引一致、
  SCHEMA_VERSION 与 kb 链最高版本一致。

.DESCRIPTION
  实际执行交给 scripts/check-migrations.py（用临时 SQLite 库，不碰真实知识库）。
  规则见 crates/storage/migrations/README.md 与 docs/project-structure.md §2.1。

.EXAMPLE
  pwsh scripts/check-migrations.ps1
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

Write-Host '== check-migrations =='

$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) {
    Write-Host '[FAIL] 找不到 python（迁移执行需要 sqlite3 模块）'
    exit 1
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ('perknow-migrations-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpRoot | Out-Null
try {
    & $python.Source (Join-Path $PSScriptRoot 'check-migrations.py') $RepoRoot $tmpRoot
    if ($LASTEXITCODE -ne 0) {
        Add-Failure 'check-migrations.py 报告了迁移问题（见上面输出）'
    }
}
finally {
    Remove-Item -LiteralPath $tmpRoot -Recurse -Force -ErrorAction SilentlyContinue
}

# SCHEMA_VERSION 必须等于 kb 链最高版本
$versionRs = Join-Path $RepoRoot 'crates/contracts/src/version.rs'
if (Test-Path -LiteralPath $versionRs) {
    $text = Get-Content -LiteralPath $versionRs -Raw -Encoding UTF8
    $m = [regex]::Match($text, 'SCHEMA_VERSION:\s*u32\s*=\s*([0-9]+)')
    if (-not $m.Success) {
        Add-Failure 'crates/contracts/src/version.rs 找不到 SCHEMA_VERSION'
    }
    else {
        $declared = [int]$m.Groups[1].Value
        $kbDir = Join-Path $RepoRoot 'crates/storage/migrations/kb'
        $files = @(Get-ChildItem -LiteralPath $kbDir -Filter '*.sql' -File)
        $maxVersion = 0
        foreach ($file in $files) {
            $vm = [regex]::Match($file.Name, '^(\d{4})_')
            if ($vm.Success) {
                $current = [int]$vm.Groups[1].Value
                if ($current -gt $maxVersion) { $maxVersion = $current }
            }
        }
        if ($declared -ne $maxVersion) {
            Add-Failure "SCHEMA_VERSION=$declared 与 kb 链最高版本 $maxVersion 不一致"
        }
        else {
            Write-Host "[ok]   SCHEMA_VERSION=$declared 与 kb 链最高版本一致"
        }
    }
}
else {
    Add-Failure 'crates/contracts/src/version.rs 不存在'
}

Write-Host ''
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) {
        Write-Host "[FAIL] $f" -ForegroundColor Red
    }
    Write-Host "check-migrations: $($Failures.Count) 项失败"
    exit 1
}
Write-Host 'check-migrations: 迁移脚本全部通过'
exit 0
