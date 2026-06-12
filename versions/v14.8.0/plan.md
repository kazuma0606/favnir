# v14.8.0 Plan — Rune ファイル --legacy 明示化 + fs.fav バグ修正

Date: 2026-06-12

---

## Phase A — `runes/fs/fs.fav` バグ修正

### A-1: `glob` 関数の非 Result `bind` をインライン化

**問題箇所（`glob` 関数内）:**

```fav
// Before (問題あり — String / List に bind を使っている)
public fn glob(dir: String, suffix: String) -> Result<List<String>, String> !IO {
    match IO.list_dir_raw(dir) {
        Err(e)    => Result.err(e)
        Ok(names) => {
            bind sep      <- "/"
            bind filtered <- List.filter(names, |name| String.ends_with(name, suffix))
            bind paths    <- List.map(filtered, |name| String.concat(String.concat(dir, sep), name))
            Result.ok(paths)
        }
    }
}
```

**修正後（中間値をインライン化）:**

```fav
// After
public fn glob(dir: String, suffix: String) -> Result<List<String>, String> !IO {
    match IO.list_dir_raw(dir) {
        Err(e)    => Result.err(e)
        Ok(names) => Result.ok(
            List.map(
                List.filter(names, |name| String.ends_with(name, suffix)),
                |name| String.concat(String.concat(dir, "/"), name)
            )
        )
    }
}
```

---

## Phase B — 全 ambient rune ファイルに `--legacy compatible` コメントを追加

各ファイルのヘッダコメント（最初の `//` 行の直下）に以下を追加する:

```
// NOTE: このファイルは --legacy compatible です。
//       関数シグネチャに !Effect アノテーションを使用していますが、
//       VM プリミティブを直接ラップする Rune として意図的に設計されています。
//       ctx-aware 移行は v15.0 以降で計画予定です。
```

対象ファイル:
- `runes/cache/cache.fav`
- `runes/fs/fs.fav`
- `runes/log/emitter.fav`
- `runes/log/metric.fav`
- `runes/queue/queue.fav`
- `runes/gen/output.fav`
- `runes/http/request.fav`
- `runes/graphql/client.fav`
- `runes/grpc/server.fav`
- `runes/duckdb/query.fav`
- `runes/duckdb/io.fav`
- `runes/db/connection.fav`

---

## Phase C — `CHANGELOG.md` 更新

`## [v14.7.0]` エントリの直前に追加:

```markdown
## [v14.8.0] — 2026-06-12

### Changed
- `runes/fs/fs.fav`: `glob` 関数内の非 Result `bind` をインライン化（バグ修正）
- rune ファイル 12 件に `--legacy compatible` コメントを追加（意図を明示）
  - `cache/cache.fav`, `fs/fs.fav`, `log/emitter.fav`, `log/metric.fav`,
    `queue/queue.fav`, `gen/output.fav`, `http/request.fav`, `graphql/client.fav`,
    `grpc/server.fav`, `duckdb/query.fav`, `duckdb/io.fav`, `db/connection.fav`
- `aws/dynamodb.fav`, `aws/sqs.fav` に `--legacy` 専用コメントを追加（v14.7.0 の継続）

### Internal
- Cargo.toml version: `14.8.0`
- `v148000_tests`: 3 件追加
```

---

## Phase D — `README.md` 更新

### D-1: 「現在の状態」見出しを `v14.8.0` に更新

```markdown
**v14.8.0（2026-06-12）— Rune ファイル整備完了**
```

テスト件数: `1540+` に更新。

### D-2: ロードマップ表に v14.8.0 行を追記

```markdown
| v14.8.0 | Rune ファイル --legacy 明示化 + fs.fav バグ修正 | ✅ |
```

---

## Phase E — `fav/src/driver.rs`: v148000_tests + バージョンバンプ

### E-1: `v148000_tests` モジュールを追加（`v147000_tests` の直前）

```rust
// ── v148000_tests (v14.8.0) — Rune ファイル整備 ──────────────────────────────
#[cfg(test)]
mod v148000_tests {
    #[test]
    fn version_is_14_8_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.8.0");
    }

    #[test]
    fn fs_rune_glob_no_bind_string() {
        // fs.fav の glob 関数に非 Result bind がないことを確認
        let fs_fav = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .join("runes/fs/fs.fav")
        ).expect("runes/fs/fs.fav should exist");
        // "bind sep" パターンが消えていること
        assert!(!fs_fav.contains("bind sep"),
            "runes/fs/fs.fav should not contain non-Result bind 'bind sep'");
    }

    #[test]
    fn ambient_runes_have_legacy_comment() {
        // ambient rune ファイルに --legacy compatible コメントが存在することを確認
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap();
        let rune_files = [
            "runes/cache/cache.fav",
            "runes/log/emitter.fav",
            "runes/queue/queue.fav",
        ];
        for path in &rune_files {
            let content = std::fs::read_to_string(base.join(path))
                .unwrap_or_else(|_| panic!("{} should exist", path));
            assert!(content.contains("--legacy compatible"),
                "{} should contain '--legacy compatible' comment", path);
        }
    }
}
```

### E-2: `v147000_tests` の `version_is_14_7_0` を `>=` 比較に修正

```rust
assert!(env!("CARGO_PKG_VERSION") >= "14.7.0",
    "expected >= 14.7.0, got {}", env!("CARGO_PKG_VERSION"));
```

### E-3: `fav/Cargo.toml` バージョンを `"14.8.0"` にバンプ

---

## Phase F — 全テスト + コミット

```bash
cargo test v148000  # 3 件全パス
cargo test          # 全件パス
git commit -m "feat: v14.8.0 — rune ファイル整備（--legacy 明示化 + fs.fav バグ修正）"
```
