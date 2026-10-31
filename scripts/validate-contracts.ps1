<#
.SYNOPSIS
  契约一致性校验（AGENTS.md §8 的一条闸门）。

.DESCRIPTION
  校验项：
  1. contract_version 一致：contracts/*.md 的声明、CHANGELOG 最新条目、
     crates/contracts/src/version.rs、生成的 apps/ui/src/lib/tauri-api/version.ts（存在时）。
  2. contracts/schemas/ 五件套齐全，JSON 可解析，示例可解析；有 python + jsonschema 时做深度校验。
  3. 命令表一致：interface-contracts.md §4 与 apps/desktop/src-tauri/src/commands/table.rs。
  4. 错误码一致：interface-contracts.md §2 与 crates/contracts/src/error.rs。
  5. 事件名一致：interface-contracts.md §5 与 crates/contracts/src/events.rs。

.PARAMETER SkipSchemas
  跳过 schema/示例校验（只跑文本一致性）。

.EXAMPLE
  pwsh scripts/validate-contracts.ps1
#>
[CmdletBinding()]
param(
    [switch]$SkipSchemas
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure([string]$Message) {
    $Failures.Add($Message) | Out-Null
}

function Add-Ok([string]$Message) {
    Write-Host "[ok]   $Message"
}

function Add-Skip([string]$Message) {
    Write-Host "[skip] $Message"
}

function Read-Text([string]$Path) {
    Get-Content -LiteralPath $Path -Raw -Encoding UTF8
}

function Get-PatternValues([string]$Text, [string]$Pattern) {
    $values = New-Object System.Collections.Generic.List[string]
    foreach ($m in [regex]::Matches($Text, $Pattern)) {
        $values.Add($m.Groups[1].Value)
    }
    return $values
}

function Get-SectionText([string]$Text, [string]$HeadingRegex) {
    $pattern = '(?ms)^' + $HeadingRegex + '[^\r\n]*\r?\n(.*?)(?=^##\s|\z)'
    $m = [regex]::Match($Text, $pattern)
    if (-not $m.Success) {
        return $null
    }
    return $m.Groups[1].Value
}

Write-Host '== validate-contracts =='

# ---------------------------------------------------------------- 1. 版本一致
$contractsDir = Join-Path $RepoRoot 'contracts'
$declared = New-Object System.Collections.Generic.List[string]

foreach ($file in Get-ChildItem -LiteralPath $contractsDir -Filter '*.md' -File) {
    $text = Read-Text $file.FullName
    foreach ($v in Get-PatternValues $text 'contract_version\s*=\s*([0-9]+\.[0-9]+\.[0-9]+)') {
        $declared.Add("$($file.Name)=$v")
    }
}

$changelogPath = Join-Path $contractsDir 'CHANGELOG.md'
if (Test-Path -LiteralPath $changelogPath) {
    $changelog = Read-Text $changelogPath
    $top = @(Get-PatternValues $changelog '(?m)^##\s+\[([0-9]+\.[0-9]+\.[0-9]+)\]')
    if ($top.Count -gt 0) {
        $declared.Add("CHANGELOG.md=$($top[0])")
    }
    else {
        Add-Failure 'contracts/CHANGELOG.md 找不到形如 ## [x.y.z] 的版本条目'
    }
}

$versionRs = Join-Path $RepoRoot 'crates/contracts/src/version.rs'
if (Test-Path -LiteralPath $versionRs) {
    $rsVersions = @(Get-PatternValues (Read-Text $versionRs) 'CONTRACT_VERSION:\s*&str\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"')
    if ($rsVersions.Count -eq 1) {
        $declared.Add("version.rs=$($rsVersions[0])")
    }
    else {
        Add-Failure 'crates/contracts/src/version.rs 未找到唯一的 CONTRACT_VERSION 定义'
    }
}
else {
    Add-Failure 'crates/contracts/src/version.rs 不存在'
}

$tsVersion = Join-Path $RepoRoot 'apps/ui/src/lib/tauri-api/version.ts'
if (Test-Path -LiteralPath $tsVersion) {
    $tsVersions = @(Get-PatternValues (Read-Text $tsVersion) 'CONTRACT_VERSION\s*=\s*[\x22\x27]?([0-9]+\.[0-9]+\.[0-9]+)')
    if ($tsVersions.Count -eq 1) {
        $declared.Add("version.ts=$($tsVersions[0])")
    }
    else {
        Add-Failure 'apps/ui/src/lib/tauri-api/version.ts 中找不到 CONTRACT_VERSION'
    }
}
else {
    Add-Skip 'apps/ui/src/lib/tauri-api/version.ts 不存在（等 D 的 gen-types 产出）'
}

$distinct = @($declared | ForEach-Object { ($_ -split '=')[1] } | Sort-Object -Unique)
if ($distinct.Count -ne 1) {
    Add-Failure "contract_version 不一致：$($declared -join ', ')"
}
else {
    Add-Ok "contract_version = $($distinct[0])（$($declared -join ', ')）"
}

# ------------------------------------------------------- 2. schema 与示例
$schemasDir = Join-Path $contractsDir 'schemas'
$examplesDir = Join-Path $schemasDir 'examples'
$requiredSchemas = @(
    'app-settings.schema.json',
    'kb-settings.schema.json',
    'export-manifest.schema.json',
    'events.schema.json',
    'sidecar.schema.json'
)

if (-not $SkipSchemas) {
    foreach ($name in $requiredSchemas) {
        $path = Join-Path $schemasDir $name
        if (-not (Test-Path -LiteralPath $path)) {
            Add-Failure "缺少 schema：contracts/schemas/$name"
            continue
        }
        try {
            $obj = Read-Text $path | ConvertFrom-Json
        }
        catch {
            Add-Failure "schema 不是合法 JSON：$name"
            continue
        }
        $props = @($obj.PSObject.Properties.Name)
        if (-not ($props -contains '$schema')) {
            Add-Failure ('schema 缺少 $schema 字段：' + $name)
        }
        if (-not ($props -contains 'title')) {
            Add-Failure ('schema 缺少 title 字段：' + $name)
        }
        if (-not (($props -contains 'type') -or ($props -contains 'oneOf') -or ($props -contains 'anyOf'))) {
            Add-Failure ('schema 既无 type 也无 oneOf/anyOf：' + $name)
        }
    }

    if (-not (Test-Path -LiteralPath $examplesDir)) {
        Add-Failure '缺少 contracts/schemas/examples/ 目录'
    }
    else {
        $stems = @($requiredSchemas | ForEach-Object { ($_ -replace '\.schema\.json$', '') })
        $examples = @(Get-ChildItem -LiteralPath $examplesDir -Filter '*.json' -File)
        if ($examples.Count -eq 0) {
            Add-Failure 'contracts/schemas/examples/ 为空'
        }
        foreach ($ex in $examples) {
            try {
                $null = Read-Text $ex.FullName | ConvertFrom-Json
            }
            catch {
                Add-Failure "示例不是合法 JSON：examples/$($ex.Name)"
                continue
            }
            $stem = ($ex.Name -split '\.')[0]
            if (-not ($stems -contains $stem)) {
                Add-Failure "示例文件名前缀应对应某个 schema 名（如 app-settings.xxx.json）：examples/$($ex.Name)"
            }
        }
        foreach ($stem in $stems) {
            if (-not ($examples | Where-Object { $_.Name -like "$stem.*" })) {
                Add-Failure "schema $stem 缺少示例：examples/$stem.<case>.json"
            }
        }
        Add-Ok "schemas 五件套 + $($examples.Count) 个示例（结构检查）"
    }

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) {
        Add-Skip '未找到 python，跳过 jsonschema 深度校验'
    }
    else {
        $helper = Join-Path $PSScriptRoot 'validate-schemas.py'
        & $python.Source $helper $RepoRoot
        if ($LASTEXITCODE -ne 0) {
            Add-Failure 'validate-schemas.py：schema 与示例不一致（见上面输出）'
        }
        else {
            Add-Ok 'jsonschema 深度校验通过'
        }
    }
}
else {
    Add-Skip '按参数跳过 schema 校验'
}

# ------------------------------------------------------------- 3. 命令表
$ifacePath = Join-Path $contractsDir 'interface-contracts.md'
if (-not (Test-Path -LiteralPath $ifacePath)) {
    Add-Failure 'contracts/interface-contracts.md 不存在'
}
else {
    $ifaceText = Read-Text $ifacePath

    $cmdSection = Get-SectionText $ifaceText '## 4\. Tauri 命令契约'
    if (-not $cmdSection) {
        Add-Failure 'interface-contracts.md 找不到 4. Tauri 命令契约 章节'
    }
    else {
        $docCommands = @(Get-PatternValues $cmdSection '(?m)^\|\s*[`]([a-z][a-z0-9_]*)[`]\s*\|' | Sort-Object -Unique)
        $tableRs = Join-Path $RepoRoot 'apps/desktop/src-tauri/src/commands/table.rs'
        if (-not (Test-Path -LiteralPath $tableRs)) {
            Add-Failure 'apps/desktop/src-tauri/src/commands/table.rs 不存在'
        }
        else {
            $tableText = Read-Text $tableRs
            $codeCommands = @(Get-PatternValues $tableText 'spec\("([a-z][a-z0-9_]*)"' | Sort-Object -Unique)
            $diff = Compare-Object -ReferenceObject $docCommands -DifferenceObject $codeCommands
            if ($diff) {
                foreach ($d in $diff) {
                    if ($d.SideIndicator -eq '<=') {
                        Add-Failure "命令表不一致（只在文档中）：$($d.InputObject)"
                    }
                    else {
                        Add-Failure "命令表不一致（只在 table.rs 中）：$($d.InputObject)"
                    }
                }
            }
            else {
                Add-Ok "命令表一致（$($docCommands.Count) 条）"
            }
            $declaredCount = @(Get-PatternValues $tableText 'COMMANDS:\s*\[CommandSpec;\s*([0-9]+)\]')
            if ($declaredCount.Count -eq 1 -and [int]$declaredCount[0] -ne $codeCommands.Count) {
                Add-Failure "table.rs 声明长度 $($declaredCount[0]) 与实际条目 $($codeCommands.Count) 不一致"
            }
        }
    }

    # --------------------------------------------------------- 4. 错误码
    $errSection = Get-SectionText $ifaceText '## 2\. 错误契约'
    $errorRs = Join-Path $RepoRoot 'crates/contracts/src/error.rs'
    if (-not $errSection) {
        Add-Failure 'interface-contracts.md 找不到 2. 错误契约 章节'
    }
    elseif (-not (Test-Path -LiteralPath $errorRs)) {
        Add-Failure 'crates/contracts/src/error.rs 不存在'
    }
    else {
        $docCodes = @(Get-PatternValues $errSection '(?m)^\|\s*[`]([A-Z][A-Z0-9_]*)[`]\s*\|' | Sort-Object -Unique)
        $codeCodes = @(Get-PatternValues (Read-Text $errorRs) '=>\s*"([A-Z][A-Z0-9_]*)"' | Sort-Object -Unique)
        $diff = Compare-Object -ReferenceObject $docCodes -DifferenceObject $codeCodes
        if ($diff) {
            foreach ($d in $diff) {
                if ($d.SideIndicator -eq '<=') {
                    Add-Failure "错误码不一致（只在文档中）：$($d.InputObject)"
                }
                else {
                    Add-Failure "错误码不一致（只在 error.rs 中）：$($d.InputObject)"
                }
            }
        }
        else {
            Add-Ok "错误码一致（$($docCodes.Count) 个）"
        }
    }

    # --------------------------------------------------------- 5. 事件名
    $eventSection = Get-SectionText $ifaceText '## 5\. 事件契约'
    $eventsRs = Join-Path $RepoRoot 'crates/contracts/src/events.rs'
    if (-not $eventSection) {
        Add-Failure 'interface-contracts.md 找不到 5. 事件契约 章节'
    }
    elseif (-not (Test-Path -LiteralPath $eventsRs)) {
        Add-Failure 'crates/contracts/src/events.rs 不存在'
    }
    else {
        $docEvents = @(Get-PatternValues $eventSection '(?m)^\|\s*[`]([a-z]+://[a-z-]+)[`]\s*\|' | Sort-Object -Unique)
        $codeEvents = @(Get-PatternValues (Read-Text $eventsRs) '=>\s*"([a-z]+://[a-z-]+)"' | Sort-Object -Unique)
        $diff = Compare-Object -ReferenceObject $docEvents -DifferenceObject $codeEvents
        if ($diff) {
            foreach ($d in $diff) {
                if ($d.SideIndicator -eq '<=') {
                    Add-Failure "事件名不一致（只在文档中）：$($d.InputObject)"
                }
                else {
                    Add-Failure "事件名不一致（只在 events.rs 中）：$($d.InputObject)"
                }
            }
        }
        else {
            Add-Ok "事件名一致（$($docEvents.Count) 个）"
        }
    }
}

Write-Host ''
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) {
        Write-Host "[FAIL] $f" -ForegroundColor Red
    }
    Write-Host "validate-contracts: $($Failures.Count) 项失败"
    exit 1
}
Write-Host 'validate-contracts: 全部通过'
exit 0
