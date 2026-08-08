# Plan — v58.4.0 — Data Catalog 統合（`fav catalog`）

## 実装方針

### アーキテクチャ上の判断

**`!Catalog` エフェクトの AST/IR 統合は行わない（本バージョン）**
v58.3.0（schema migration）・v58.2.0（canary）と同様に、driver.rs レベルのスタブ実装で完結させる。
AST 変更は大規模であり、2 テスト追加の目標に対してリスクが高い。

**`cmd_catalog_push` / `cmd_catalog_search` は `i32` 戻り値**
v58.3.0 の `cmd_schema_migrate` と同じ設計を踏襲する。
main.rs から `std::process::exit(driver::cmd_catalog_xxx(...))` で終了。

**`Some("catalog")` arm は `driver::` プレフィックス経由**
main.rs の既存 `Some("schema")` arm は `cmd_schema_diff` を use インポート経由で呼んでいると想定される。
v58.4.0 では use 行の肥大化を避けるため `driver::` プレフィックス方式を採用する。
T0 で main.rs の `Some("schema")` arm を確認し、呼び出し形式を実装前に確認すること。

**フラグ値欠落は明示エラー（v58.2.0/v58.3.0 で確立したパターン）**
`--catalog` フラグが指定されたが値がない場合は `eprintln!` + `process::exit(1)` で終了する。

---

## 実装順序

```
T1: Cargo.toml 58.4.0
T2: driver.rs cmd_catalog_push 追加（cmd_schema_migrate の直後）
T3: driver.rs cmd_catalog_search 追加（cmd_catalog_push の直後）
T4: main.rs Some("catalog") arm 追加（Some("schema") arm の後、Some("doc") arm の前）
T5: driver.rs v58400_tests 追加（v58300_tests の直前）
T6: rolling チェック 5 件更新（"58.3.0" → "58.4.0"）
T7: cargo build
T8: cargo test（3288 passed 確認）
T9: cargo clippy
```

---

## ファイル変更一覧

| ファイル | 変更種別 | 詳細 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | `58.3.0` → `58.4.0` |
| `fav/src/driver.rs` | 追加 | `cmd_catalog_push` 関数（cmd_schema_migrate の直後） |
| `fav/src/driver.rs` | 追加 | `cmd_catalog_search` 関数（cmd_catalog_push の直後） |
| `fav/src/driver.rs` | 追加 | `v58400_tests` モジュール（v58300_tests の直前） |
| `fav/src/driver.rs` | 更新 | rolling チェック 5 件（v56300/v56900/v57000/v57900/v58000） |
| `fav/src/main.rs` | 追加 | `Some("catalog")` arm（`Some("schema")` の後） |

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `Some("catalog")` arm の挿入位置が main.rs で重複 | 事前に `grep '"catalog"'` で確認する（T0） |
| テスト名 `cmd_catalog_push` が関数名と衝突 | `cmd_catalog_push_test` / `cmd_catalog_search_test` を使用 |
| rolling チェックの replace_all が v58400_tests 内の文字列に影響 | v58400_tests は version 文字列を含まないため影響なし。実施後に grep で確認 |
| `driver::` プレフィックス vs use インポートの混在 | main.rs 既存の `Some("catalog")` がないことを T0 で確認済みであれば問題なし |

---

## ポスト処理

- `CHANGELOG.md` に `[v58.4.0]` エントリ追加
- `versions/current.md` を v58.4.0 / 3288 tests に更新
- `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.4.0 実績を COMPLETE に更新
- `versions/v55-v60/v58.4.0/tasks.md` を COMPLETE に更新
