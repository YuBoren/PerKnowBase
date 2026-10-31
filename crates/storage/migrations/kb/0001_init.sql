-- PerKnowBase · 知识库库 · 0001_init.sql
--
-- 目标库：<storage_root>/<kb_id>/kb.sqlite3（每库一份，contracts/data-model.md §4.2）
-- 执行时机：新建库（空库全量执行）、打开旧库（只跑未执行脚本）、导入旧包（补齐缺失脚本）。
--
-- 硬规则（docs/project-structure.md §2.1、BAN-15）：
--   1. 已合入的脚本禁止修改，只能追加 0002_*、0003_*。
--   2. 文件名前缀 = schema_version，必须连续、单调递增。
--   3. 枚举列不加 CHECK：新增枚举取值属于契约 MINOR（contracts/README.md §2），
--      加 CHECK 会迫使整表重建，与此冲突；取值校验在 Rust 侧（serde + 契约类型）完成。
--   4. 时间一律 epoch 毫秒 INTEGER；JSON 列一律 TEXT，写入前经 schema 校验。

PRAGMA journal_mode = WAL;

-- 迁移账本：与 app 库同构，独立编号（schema_version 不跟随 contract_version）。
CREATE TABLE IF NOT EXISTS schema_migrations (
    version    INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL,
    checksum   TEXT    NOT NULL
);

-- 单行元数据（contracts/data-model.md §4.2 kb_meta）。
CREATE TABLE IF NOT EXISTS kb_meta (
    kb_id             TEXT    PRIMARY KEY,
    name              TEXT    NOT NULL,
    root_dir          TEXT    NOT NULL,
    status            TEXT    NOT NULL,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    schema_version    INTEGER NOT NULL,
    contract_version  TEXT    NOT NULL,
    app_version       TEXT    NOT NULL,
    index_version     INTEGER NOT NULL DEFAULT 0,
    vector_model      TEXT    NOT NULL,
    vector_dim        INTEGER NOT NULL,
    settings_snapshot TEXT    NOT NULL,
    stats_files       INTEGER NOT NULL DEFAULT 0,
    stats_chunks      INTEGER NOT NULL DEFAULT 0,
    stats_vectors     INTEGER NOT NULL DEFAULT 0,
    stats_failed      INTEGER NOT NULL DEFAULT 0
);

-- 文件/目录节点：files/ 的元数据镜像（contracts/data-model.md §2.2、§4.2 file_node）。
CREATE TABLE IF NOT EXISTS file_node (
    file_id      TEXT    PRIMARY KEY,
    kb_id        TEXT    NOT NULL,
    parent_id    TEXT,
    rel_path     TEXT    NOT NULL,
    name         TEXT    NOT NULL,
    kind         TEXT    NOT NULL,
    ext          TEXT,
    size         INTEGER NOT NULL DEFAULT 0,
    mtime        INTEGER NOT NULL,
    content_hash TEXT,
    parse_status TEXT    NOT NULL DEFAULT 'pending',
    index_status TEXT    NOT NULL DEFAULT 'pending',
    error_code   TEXT,
    error_message TEXT,
    metadata     TEXT,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES file_node (file_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_file_node_kb_rel_path
    ON file_node (kb_id, rel_path);
CREATE INDEX IF NOT EXISTS idx_file_node_kb_parent
    ON file_node (kb_id, parent_id);
CREATE INDEX IF NOT EXISTS idx_file_node_kb_kind
    ON file_node (kb_id, kind);
CREATE INDEX IF NOT EXISTS idx_file_node_kb_content_hash
    ON file_node (kb_id, content_hash);
CREATE INDEX IF NOT EXISTS idx_file_node_kb_parse_status
    ON file_node (kb_id, parse_status);
CREATE INDEX IF NOT EXISTS idx_file_node_kb_index_status
    ON file_node (kb_id, index_status);

-- 文件级哈希（独立表，契约要求；content_hash 与 chunk_hash 不拆表，见 DEC-20261026-09）。
CREATE TABLE IF NOT EXISTS file_hash (
    file_id      TEXT    PRIMARY KEY,
    kb_id        TEXT    NOT NULL,
    rel_path     TEXT    NOT NULL,
    content_hash TEXT    NOT NULL,
    algorithm    TEXT    NOT NULL DEFAULT 'sha256',
    size         INTEGER NOT NULL,
    mtime        INTEGER NOT NULL,
    hashed_at    INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file_node (file_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_file_hash_kb_rel_path
    ON file_hash (kb_id, rel_path);
CREATE INDEX IF NOT EXISTS idx_file_hash_kb_content_hash
    ON file_hash (kb_id, content_hash);

-- 块表：content 是 BM25 与嵌入的唯一文本源（BAN-07）。
CREATE TABLE IF NOT EXISTS chunk (
    chunk_id        TEXT    PRIMARY KEY,
    kb_id           TEXT    NOT NULL,
    file_id         TEXT    NOT NULL,
    seq             INTEGER NOT NULL,
    content         TEXT    NOT NULL,
    content_hash    TEXT    NOT NULL,
    source_location TEXT    NOT NULL,
    chunk_type      TEXT    NOT NULL,
    parent_id       TEXT,
    token_count     INTEGER,
    metadata        TEXT,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file_node (file_id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES chunk (chunk_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_chunk_kb_file_seq
    ON chunk (kb_id, file_id, seq);
CREATE INDEX IF NOT EXISTS idx_chunk_kb_file
    ON chunk (kb_id, file_id);
CREATE INDEX IF NOT EXISTS idx_chunk_kb_content_hash
    ON chunk (kb_id, content_hash);
CREATE INDEX IF NOT EXISTS idx_chunk_kb_type
    ON chunk (kb_id, chunk_type);

-- 库级设置：单行 JSON（KbSettings），主键即 kb_id。
CREATE TABLE IF NOT EXISTS settings (
    kb_id      TEXT    PRIMARY KEY,
    json       TEXT    NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 同步任务（contracts/data-model.md §2.10、§5.4）。
CREATE TABLE IF NOT EXISTS sync_task (
    task_id     TEXT    PRIMARY KEY,
    kb_id       TEXT    NOT NULL,
    status      TEXT    NOT NULL,
    trigger     TEXT    NOT NULL,
    scope       TEXT    NOT NULL,
    total       INTEGER NOT NULL DEFAULT 0,
    done        INTEGER NOT NULL DEFAULT 0,
    failed      INTEGER NOT NULL DEFAULT 0,
    errors      TEXT,
    created_at  INTEGER NOT NULL,
    started_at  INTEGER,
    finished_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_sync_task_kb_created
    ON sync_task (kb_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sync_task_status
    ON sync_task (status);

-- 检索历史（软引用：快照不随文件变化改写）。
CREATE TABLE IF NOT EXISTS history (
    history_id     TEXT    PRIMARY KEY,
    kb_id          TEXT    NOT NULL,
    query          TEXT    NOT NULL,
    mode           TEXT    NOT NULL,
    hybrid         TEXT    NOT NULL,
    top_k          INTEGER NOT NULL,
    result_summary TEXT    NOT NULL,
    hit_count      INTEGER NOT NULL,
    degraded       INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_kb_created
    ON history (kb_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_kb_mode
    ON history (kb_id, mode);

-- 历史命中快照（轻量，≤20 条/次）。
CREATE TABLE IF NOT EXISTS history_hit (
    history_id      TEXT    NOT NULL,
    rank            INTEGER NOT NULL,
    chunk_id        TEXT    NOT NULL,
    file_id         TEXT    NOT NULL,
    rel_path        TEXT    NOT NULL,
    file_type       TEXT    NOT NULL,
    source_location TEXT    NOT NULL,
    preview         TEXT    NOT NULL,
    score           REAL    NOT NULL,
    alive           INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (history_id, rank),
    FOREIGN KEY (history_id) REFERENCES history (history_id) ON DELETE CASCADE
);

-- 融合操作整体状态（A 独占写，回滚依据之一）。
CREATE TABLE IF NOT EXISTS merge_op (
    op_id          TEXT    PRIMARY KEY,
    source_kb_id   TEXT    NOT NULL,
    target_kb_id   TEXT    NOT NULL,
    target_rel_dir TEXT    NOT NULL,
    mode           TEXT    NOT NULL,
    conflict       TEXT    NOT NULL,
    status         TEXT    NOT NULL,
    entry_count    INTEGER NOT NULL DEFAULT 0,
    done_count     INTEGER NOT NULL DEFAULT 0,
    error          TEXT,
    created_at     INTEGER NOT NULL,
    finished_at    INTEGER
);

-- 融合日志条目（条目级状态见 content/architecture-boundaries 的数据所有权矩阵）。
CREATE TABLE IF NOT EXISTS merge_journal (
    entry_id    TEXT    PRIMARY KEY,
    op_id       TEXT    NOT NULL,
    seq         INTEGER NOT NULL,
    kb_id_src   TEXT    NOT NULL,
    kb_id_dst   TEXT    NOT NULL,
    action      TEXT    NOT NULL,
    source_path TEXT,
    target_path TEXT,
    status      TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    FOREIGN KEY (op_id) REFERENCES merge_op (op_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_merge_journal_op_seq
    ON merge_journal (op_id, seq);

-- 重构断点（可续跑）。
CREATE TABLE IF NOT EXISTS rebuild_state (
    kb_id      TEXT    PRIMARY KEY,
    op_id      TEXT    NOT NULL,
    reason     TEXT    NOT NULL,
    step       TEXT    NOT NULL,
    done       INTEGER NOT NULL DEFAULT 0,
    total      INTEGER NOT NULL DEFAULT 0,
    started_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
