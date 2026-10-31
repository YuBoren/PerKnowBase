-- PerKnowBase · 应用级库 · 0001_init.sql
--
-- 目标库：<app_data>/app.sqlite3（全局一份，contracts/data-model.md §4.1）
-- 内容：知识库注册表 + 应用设置 + 迁移账本。
--
-- 硬规则（docs/project-structure.md §2.1、BAN-15）：
--   1. 已合入的脚本禁止修改，只能追加 0002_*、0003_*。
--   2. 文件名前缀 = schema_version，必须连续、单调递增。
--   3. 只做 SQLite 结构变更（DDL），不搬运业务数据、不涉及 Qdrant。
--   4. 空库必须能从 0001 顺序跑到最新（scripts/check-migrations.ps1 校验）。
--
-- PRAGMA 由连接层设置（journal_mode=WAL、foreign_keys=ON、busy_timeout=5000），
-- 不放脚本里，避免迁移过程中隐式事务行为不一致。

-- 迁移账本：每执行一个脚本写一行；checksum 不符视为库被篡改（KB_CORRUPTED）。
CREATE TABLE IF NOT EXISTS app_schema_migrations (
    version    INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL,
    checksum   TEXT    NOT NULL
);

-- 应用设置：单行 JSON（AppSettings），id 固定为 1。
CREATE TABLE IF NOT EXISTS app_settings (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    json       TEXT    NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 知识库注册表：一人一行/库；root_dir 唯一，防止同一目录被注册两次。
CREATE TABLE IF NOT EXISTS kb_registry (
    kb_id            TEXT    PRIMARY KEY,
    name             TEXT    NOT NULL,
    root_dir         TEXT    NOT NULL UNIQUE,
    status           TEXT    NOT NULL,
    app_version      TEXT    NOT NULL,
    schema_version   INTEGER NOT NULL,
    contract_version TEXT    NOT NULL,
    created_at       INTEGER NOT NULL,
    last_opened_at   INTEGER,
    stats_files      INTEGER NOT NULL DEFAULT 0,
    stats_chunks     INTEGER NOT NULL DEFAULT 0,
    size_bytes       INTEGER NOT NULL DEFAULT 0
);

-- 启动时按最近打开排序（kb_list、启动界面）。
CREATE INDEX IF NOT EXISTS idx_kb_registry_last_opened
    ON kb_registry (last_opened_at DESC);
