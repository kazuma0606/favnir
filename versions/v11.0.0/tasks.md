# Favnir v11.0.0 Tasks

Date: 2026-06-05
Theme: Snowflake 統合完成 + リネージ可視化 + ドキュメント整備

---

## Phase A: lineage.rs — `!Snowflake(read/write)` 区別

- [x] A-1: `lineage.rs` に `collect_snowflake_call_kinds(expr) -> (bool, bool)` 追加
- [x] A-2: `lineage_analysis` の TrfDef ループを更新（`snowflake_effects` helper 使用）
- [x] A-3: 同様の更新を FnDef ループにも適用
- [x] A-4: `lineage.rs` の `#[cfg(test)] mod v11000_tests` に 3 件追加
  - [x] A-4a: `lineage_snowflake_write_stage_shows_write_label`
  - [x] A-4b: `lineage_snowflake_read_stage_shows_read_label`
  - [x] A-4c: `lineage_snowflake_undistinguished_falls_back`
- [x] A-5: `cargo test v11000 --lib` — 3 件通過

---

## Phase B: CHANGELOG.md 更新

- [x] B-1: `CHANGELOG.md` 先頭に `[v11.0.0]` エントリ追加
- [x] B-2: `[v10.9.0]` 〜 `[v10.1.0]` の全エントリを追記

---

## Phase C: README.md 更新

- [x] C-1: Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加
- [x] C-2: ロードマップ表に `v10.1.0〜v10.9.0` / `v11.0.0` 行を追記

---

## Phase D: site/content/docs/runes/snowflake.mdx

- [x] D-1: `site/content/docs/runes/snowflake.mdx` 新規作成
  - 概要 / インストール / fav.toml 設定 / 環境変数 / API リファレンス
  - `fav infer --from snowflake` / `fav explain --lineage` / 完全なコード例

---

## Phase E: バージョン更新

- [x] E-1: `fav/Cargo.toml` version → `"11.0.0"`
- [x] E-2: `fav/self/cli.fav` の `run_version` → `"11.0.0"`

---

## Phase F: self-check + cargo test

- [x] F-1: `fav check --legacy-check self/compiler.fav` — エラーなし（実行時確認）
- [x] F-2: `cargo test bootstrap` — 通過
- [x] F-3: `cargo test` — 全件通過（1286 件）

---

## Phase G: 完了処理

- [x] G-1: 本ファイル完了チェック
- [x] G-2: `memory/MEMORY.md` に v11.0.0 完了を記録
- [x] G-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` が区別表示される | ✓ |
| lineage テスト 3 件通過 | ✓ |
| CHANGELOG.md に v10.1.0〜v11.0.0 全履歴が記載されている | ✓ |
| README.md の Rune 表に `snowflake` が含まれる | ✓ |
| `site/content/docs/runes/snowflake.mdx` が存在する | ✓ |
| `cargo test` 全件通過（1286 件） | ✓ |
