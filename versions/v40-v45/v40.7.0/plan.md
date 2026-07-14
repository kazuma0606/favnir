# v40.7.0 実装計画

## 概要

Streaming Foundations スプリント第 7 版。
`BenchOpts` に `stream: bool` フィールドを追加し、
`cmd_bench` にスタブ分岐、`main.rs` に `--stream` フラグ解析を追加する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2832 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.6.0` であることを確認
- `v40600_tests::cargo_toml_version_is_40_6_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `BenchOpts` に `stream` フィールドが存在しないことを確認（`driver.rs` 行 5128〜5134 付近）
- `driver.rs` に `v40700_tests` モジュールが存在しないことを確認

### Step 2 — driver.rs: BenchOpts + cmd_bench 更新
1. `BenchOpts` に `pub stream: bool` フィールドを追加（`json: bool` の直後）
2. `BenchOpts::default()` の構築部に `stream: false` を追加
3. `cmd_bench` 関数先頭に `--stream` スタブ分岐を追加

### Step 3 — main.rs: --stream フラグ解析追加
`bench` アームの `--json` 解析の直後に `"--stream"` アームを追加。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.6.0"` → `"40.7.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v40.7.0]` エントリを `[v40.6.0]` の直後に追加。

### Step 6 — driver.rs: テストモジュール更新
1. `v40600_tests::cargo_toml_version_is_40_6_0` をスタブ化
2. `v40700_tests` モジュール（3 テスト）を末尾に追加（`use super::*` 付き）

### Step 7 — cargo test 実行
`cargo test` で 2835 tests / 0 failures を確認。

### Step 8 — バージョン管理ドキュメント更新
`versions/current.md`・ロードマップ完了マーク・`tasks.md` COMPLETE 更新。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（driver.rs — BenchOpts + cmd_bench）
       └→ Step 6（driver.rs — bench_opts_has_stream_field）
  └→ Step 3（main.rs — --stream パース）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_40_7_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v40_7_0）
            └→ Step 7（cargo test）
                 └→ Step 8（docs 更新）
```

Step 2〜5 は相互に独立しており並列実施可能。

---

## リスクと注意点

- `BenchOpts::default()` の構築部に `stream: false` の追加を忘れると `Default` impl がコンパイルエラーになる
- `main.rs` の `--stream` アームは `--json` アームの直後に追加する（順序依存なし、位置の一貫性のため）
- `v40700_tests` は `BenchOpts` を直接参照するため `use super::*` が必要
- `cmd_bench` のスタブ分岐は関数の先頭ロジック（ファイル収集前）に置く
