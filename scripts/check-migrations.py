"""空库执行迁移脚本（被 scripts/check-migrations.ps1 调用）。

做法：每个链（app / kb）建一个全新的临时 SQLite 库，按版本顺序执行全部脚本，
然后核对脚本里声明的表/索引是否真的存在。退出码 0 = 通过。
"""
import glob
import os
import re
import sqlite3
import sys

NAME_RE = re.compile(r"^(\d{4})_([a-z0-9_]+)\.sql$")
TABLE_RE = re.compile(r"(?im)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-z_][a-z0-9_]*)")
INDEX_RE = re.compile(r"(?im)^\s*CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-z_][a-z0-9_]*)")


def check_chain(name, chain_dir, tmp_dir):
    failures = []
    files = sorted(glob.glob(os.path.join(chain_dir, "*.sql")))
    if not files:
        return ["[%s] 链目录下没有 .sql 脚本" % name], 0

    versions = []
    for path in files:
        match = NAME_RE.match(os.path.basename(path))
        if not match:
            failures.append("[%s] 文件名不合规（应为 NNNN_name.sql）：%s" % (name, os.path.basename(path)))
            continue
        versions.append(int(match.group(1)))

    if versions != list(range(1, len(versions) + 1)):
        failures.append("[%s] 版本必须从 0001 起连续单调，实际：%s" % (name, versions))
        return failures, versions[-1] if versions else 0

    db_path = os.path.join(tmp_dir, "probe_%s.sqlite3" % name)
    if os.path.exists(db_path):
        os.remove(db_path)

    con = sqlite3.connect(db_path)
    try:
        con.execute("PRAGMA foreign_keys = ON")
        declared_tables = set()
        declared_indexes = set()
        for path in files:
            with open(path, "r", encoding="utf-8") as fh:
                sql = fh.read()
            declared_tables.update(TABLE_RE.findall(sql))
            declared_indexes.update(INDEX_RE.findall(sql))
            try:
                con.executescript(sql)
            except sqlite3.Error as exc:
                failures.append("[%s] %s 执行失败：%s" % (name, os.path.basename(path), exc))
                return failures, versions[-1]

        actual_tables = {
            row[0]
            for row in con.execute(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
            )
        }
        actual_indexes = {
            row[0]
            for row in con.execute(
                "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
            )
        }

        missing_tables = sorted(declared_tables - actual_tables)
        missing_indexes = sorted(declared_indexes - actual_indexes)
        if missing_tables:
            failures.append("[%s] 声明了但未建成的表：%s" % (name, missing_tables))
        if missing_indexes:
            failures.append("[%s] 声明了但未建成的索引：%s" % (name, missing_indexes))

        ledger = "app_schema_migrations" if name == "app" else "schema_migrations"
        if ledger not in actual_tables:
            failures.append("[%s] 缺少迁移账本表 %s" % (name, ledger))
        else:
            columns = [row[1] for row in con.execute("PRAGMA table_info(%s)" % ledger)]
            for expected in ("version", "applied_at", "checksum"):
                if expected not in columns:
                    failures.append("[%s] 账本表 %s 缺少列 %s" % (name, ledger, expected))
    finally:
        con.close()

    if not failures:
        print("[ok]   [%s] %d 个脚本在空库上顺序执行成功（%d 表 / %d 索引）"
              % (name, len(files), len(actual_tables), len(actual_indexes)))
    return failures, versions[-1]


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    tmp_dir = sys.argv[2] if len(sys.argv) > 2 else os.path.join(root, ".tmp")
    os.makedirs(tmp_dir, exist_ok=True)

    failures = []
    max_versions = {}
    base = os.path.join(root, "crates", "storage", "migrations")
    for chain in ("app", "kb"):
        chain_dir = os.path.join(base, chain)
        if not os.path.isdir(chain_dir):
            failures.append("缺少迁移链目录：crates/storage/migrations/%s" % chain)
            continue
        problems, max_version = check_chain(chain, chain_dir, tmp_dir)
        failures.extend(problems)
        max_versions[chain] = max_version

    for message in failures:
        print("[FAIL] " + message)
    if not failures:
        print("[info] 链版本：%s" % max_versions)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
