# Spec — v58.3.0 — スキーママイグレーション / バージョニング

## 概要

`fav schema migrate` コマンドを追加し、スキーマバージョン間のデータ変換機能を提供する。
`apply_migration_transform` ヘルパー関数を driver.rs に追加して migration 変換ロジックを検証し、
`cmd_schema_migrate` 関数で `--from`/`--to`/`--data` フラグを処理する。
既存の `Some("schema")` arm（main.rs）に `"migrate"` サブコマンドを追加する。
Rust テスト 2 件（`schema_migration_transforms` / `cmd_schema_migrate`）で動作を検証する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v58.1-v59.0.md` — v58.3.0 セクション
- ベーステスト数: **3283**（v58.2.0 完了時点の実績値、code-review 対応込み）
- 目標テスト数: **3285**（+2）、かつ `cargo test` failures=0
- rolling チェック更新: **5 件**（v56300 / v56900 / v57000 / v57900 / v58000）を `58.2.0` → `58.3.0`

---

## スコープ外項目

| 項目 | 備考 |
|---|---|
| `migration` ブロックの AST ノード追加 / parser 統合 | 本バージョンは driver レベルのスタブ実装。AST 統合は将来版 |
| JSONL ファイルの実読み込み（I/O） | `cmd_schema_migrate` は引数受け取りと出力のみ（`data_file` を表示するが読まない） |
| `assert_schema` バージョン引数追加 | 「v52.0 実装済みを活用」だが本バージョンではスコープ外 |
| マイグレーション chain（v1→v2→v3） | 単一ステップのみ |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "58.3.0"
```

### 2. `fav/src/driver.rs` — `apply_migration_transform` 追加

スキーマ変換のコアロジック。既存フィールドを保持しつつ、不足フィールドをデフォルト値で補完する。

```rust
/// スキーママイグレーション変換: record に不足しているフィールドを defaults で補完する。
pub fn apply_migration_transform(
    mut record: serde_json::Value,
    defaults: &[(&str, serde_json::Value)],
) -> serde_json::Value {
    if let Some(map) = record.as_object_mut() {
        for (k, v) in defaults {
            map.entry(k.to_string()).or_insert_with(|| v.clone());
        }
    }
    record
}
```

`serde_json` は既存依存関係（`Cargo.toml` の `serde_json = "1"`）を使用。
`apply_migration_transform` は `cmd_schema_diff`（line 11011）の直後に追加する。

### 3. `fav/src/driver.rs` — `cmd_schema_migrate` 追加

`fav schema migrate --from <ver> --to <ver> --data <file>` を処理する関数。

```rust
pub fn cmd_schema_migrate(from: &str, to: &str, data_file: &str) -> i32 {
    println!("Schema migration: {} → {}", from, to);
    println!("  Input : {}", data_file);
    println!("  Status: OK (dry-run mode)");
    0
}
```

`apply_migration_transform` の直後に追加する（挿入順: `cmd_schema_diff` → `apply_migration_transform` → `cmd_schema_migrate`）。

### 4. `fav/src/main.rs` — `Some("schema")` arm に `"migrate"` 追加

既存の `Some("diff")` アームに続けて `Some("migrate")` を追加する。

```rust
Some("schema") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("diff") => {
            let old_file = args.get(3).map(|s| s.as_str());
            let new_file = args.get(4).map(|s| s.as_str());
            cmd_schema_diff(old_file, new_file);
        }
        Some("migrate") => {
            let from = args.windows(2).find(|w| w[0] == "--from")
                .map(|w| w[1].as_str()).unwrap_or("v1");
            let to = args.windows(2).find(|w| w[0] == "--to")
                .map(|w| w[1].as_str()).unwrap_or("v2");
            let data = args.windows(2).find(|w| w[0] == "--data")
                .map(|w| w[1].as_str()).unwrap_or("data.jsonl");
            std::process::exit(driver::cmd_schema_migrate(from, to, data));
        }
        sub => {
            eprintln!(
                "error: unknown subcommand `{}` for `fav schema`\n  usage: fav schema diff <old.fav> <new.fav>",
                sub.unwrap_or("(none)")
            );
            process::exit(1);
        }
    }
}
```

main.rs の `cmd_schema_diff` および `cmd_schema_migrate` は driver インポートを通じて使用する。

### 5. `fav/src/driver.rs` — `v58300_tests` 追加

`v58200_tests` の直前に挿入する。

```rust
// -- v58300_tests (v58.3.0) -- スキーママイグレーション / バージョニング --
#[cfg(test)]
mod v58300_tests {
    use super::{apply_migration_transform, cmd_schema_migrate};

    #[test]
    fn schema_migration_transforms() {
        // v1 レコードに currency フィールドを補完して v2 に変換
        let v1 = serde_json::json!({"id": 1, "amount": 99.9});
        let v2 = apply_migration_transform(
            v1,
            &[("currency", serde_json::Value::String("JPY".to_string()))],
        );
        assert_eq!(v2["id"], 1);
        assert_eq!(v2["amount"], 99.9);
        assert_eq!(v2["currency"], "JPY");
    }

    #[test]
    fn cmd_schema_migrate_test() {
        let code = cmd_schema_migrate("v1", "v2", "orders.jsonl");
        assert_eq!(code, 0, "schema migrate should succeed");
    }
}
```

**注意**: ロードマップのテスト名は `cmd_schema_migrate` だが、`cmd_schema_migrate` は関数名と衝突するため
テスト関数名は `cmd_schema_migrate_test` とする（driver.rs の慣例: 関数名と同一テスト名は避ける）。

### 6. Rolling バージョンチェック更新（5 件）

```
v56300_tests  : "58.2.0" → "58.3.0"
v56900_tests  : "58.2.0" → "58.3.0"
v57000_tests  : "58.2.0" → "58.3.0"
v57900_tests  : "58.2.0" → "58.3.0"
v58000_tests  : "58.2.0" → "58.3.0"
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `schema_migration_transforms` | `apply_migration_transform` が `currency: "JPY"` を補完する |
| `cmd_schema_migrate_test` | `cmd_schema_migrate("v1", "v2", "orders.jsonl")` が 0 を返す |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3285 tests passed, 0 failed**、ベース 3283 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v58300_tests` 2 件全 pass
- `fav schema migrate --from v1 --to v2 --data orders.jsonl` が `Some("migrate")` アームで処理される
- rolling チェック 5 件が `"58.3.0"` になっている

---

## 備考

- `apply_migration_transform` は `serde_json::Value` を引数に取る（コピーではなく `mut` で受け取り）
- `cmd_schema_migrate` は実際のファイル読み込みを行わない（I/O スタブ）
- rolling チェックは宣言バージョン以外でも全件更新が必要（v58.1.0/v58.2.0 で確認済みパターン）
- テスト名 `cmd_schema_migrate_test` は関数名衝突を避けるための命名（driver.rs 慣例）
