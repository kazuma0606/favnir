# Plan — v58.3.0 — スキーママイグレーション / バージョニング

## 実装方針

### アーキテクチャ上の判断

**AST 統合は行わない（本バージョン）**
ロードマップは `migration` ブロックの AST/parser 追加を言及しているが、
v58.x の一貫したパターン（deploy/canary など）はすべて driver.rs レベルの実装で完結させている。
AST 統合は大規模変更を伴い、2 テスト追加の目標に対してリスクが高い。
`apply_migration_transform` 関数でコア変換ロジックを driver.rs に実装し、
`cmd_schema_migrate` 関数で CLI サブコマンドとして公開する設計を採用する。

**`cmd_schema_migrate` の戻り値は `i32`**
`cmd_schema_diff` は `void` だが、`cmd_deploy_strategy`（v58.1/58.2）と整合させるため
`cmd_schema_migrate` は `i32` を返す設計とし、main.rs から `std::process::exit` で終了する。

**`apply_migration_transform` は `serde_json::Value` ベース**
`serde_json` はすでに Cargo.toml に登録済み。
JSON を扱うことで JSONL データ変換の実態に近いロジックを表現できる。

---

## 実装順序

```
T1: Cargo.toml 58.3.0
T2: driver.rs apply_migration_transform 追加
T3: driver.rs cmd_schema_migrate 追加（cmd_schema_diff の直後）
T4: main.rs Some("schema") arm に "migrate" 追加
T5: driver.rs v58300_tests 追加（v58200_tests の直前）
T6: rolling チェック 5 件更新
T7: cargo build
T8: cargo test（3285 passed 確認）
T9: cargo clippy
```

---

## ファイル変更一覧

| ファイル | 変更種別 | 詳細 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | `58.2.0` → `58.3.0` |
| `fav/src/driver.rs` | 追加 | `apply_migration_transform` 関数 |
| `fav/src/driver.rs` | 追加 | `cmd_schema_migrate` 関数（cmd_schema_diff の直後） |
| `fav/src/driver.rs` | 追加 | `v58300_tests` モジュール（v58200_tests の直前） |
| `fav/src/driver.rs` | 更新 | rolling チェック 5 件（v56300/v56900/v57000/v57900/v58000） |
| `fav/src/main.rs` | 更新 | `Some("schema")` arm に `Some("migrate")` 追加 |

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `apply_migration_transform` の関数シグネチャが `serde_json` import を要求する | driver.rs は既に `use serde_json;` または `serde_json::Value` を使用済みのはず。事前に確認する |
| テスト名 `cmd_schema_migrate` が関数名と衝突 | テスト関数名を `cmd_schema_migrate_test` とする |
| `Some("schema")` の既存 `_` アームエラーメッセージが "diff" のみ言及 | `migrate` 追加後もエラーメッセージは変更しない（今回はスコープ外） |
| rolling チェック 5 件の更新漏れ | sed / replace_all で一括更新し、`cargo test` で確認 |

---

## ポスト処理

- `CHANGELOG.md` に `[v58.3.0]` エントリ追加
- `versions/current.md` を v58.3.0 / 3285 tests に更新
- `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.3.0 実績を COMPLETE に更新
- `versions/v55-v60/v58.3.0/tasks.md` を COMPLETE に更新
