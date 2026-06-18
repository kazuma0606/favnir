# Favnir v11.8.0 Tasks

Date: 2026-06-06
Theme: `fav transpile` CLI 完成 + checker 統合

---

## Phase A — emit_python.rs: lineage_comments フィールド + 新 API

- [x] A-1: `use std::collections::HashMap` 追加
- [x] A-2: `Emitter` 構造体に `lineage_comments: HashMap<String, String>` 追加
- [x] A-3: `Emitter::new()` に `lineage_comments: HashMap::new()` 追加
- [x] A-4: `emit_python_with_lineage(prog, source_path, comments)` 関数追加

---

## Phase B — emit_fn_def / emit_trf_def コメント挿入

- [x] B-1: `emit_fn_def` の先頭に `lineage_comments` 参照コメント挿入
- [x] B-2: `emit_trf_def` の先頭に `lineage_comments` 参照コメント挿入

---

## Phase C — driver.rs: build_lineage_comments ヘルパー

- [x] C-1: `build_lineage_comments(report) -> HashMap<String, String>` 追加
  - `effects` が空なら `"Pure"`、それ以外は `effects.join(", ")`
  - `sources` / `sinks` が空なら `"-"`
  - コメント形式: `# [lineage] effects: X | sources: Y | sinks: Z`

---

## Phase D — cmd_transpile: --no-check / --lineage + 型チェック統合

- [x] D-1: `do_no_check: bool` / `do_lineage: bool` 変数追加
- [x] D-2: `--no-check` / `--lineage` のパース追加
- [x] D-3: `check_source_str(&src)` を emit_python 前に呼ぶ（`--no-check` でスキップ）
- [x] D-4: 型エラー時に `format_diagnostic` で表示して exit 1
- [x] D-5: `--lineage` 時に `emit_python_with_lineage` を使う分岐
- [x] D-6: `check_source_str_pub` wrapper を追加（テスト用 pub 関数）

---

## Phase E — テスト（6 件）

- [x] E-1: `v11800_tests` モジュール追加
  - [x] `transpile_blocks_on_type_error` — `!Postgres` なし → E0315 検出
  - [x] `transpile_type_check_passes_valid` — 正常コードでエラーなし
  - [x] `transpile_lineage_comment_effects` — `# [lineage] effects:` コメント付与
  - [x] `transpile_lineage_comment_pure_fn` — エフェクトなし fn で `Pure` コメント
  - [x] `transpile_no_check_skips_error` — `--no-check` 相当で型エラーコードも Python 生成
  - [x] `transpile_lineage_postgres_fn` — `!Postgres` fn に `!Postgres` コメント付与
- [x] E-2: `cargo test v11800` — 6 件通過
- [x] E-3: `cargo test --lib` — 705 件以上通過

---

## Phase F — バージョン更新 + コミット

- [x] F-1: `fav/Cargo.toml` version → `"11.8.0"`
- [x] F-2: `cargo build` で `Cargo.lock` 更新
- [ ] F-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav transpile` が型チェックを実行し、エラーで Python 生成をブロック | |
| `--no-check` で型チェックをスキップできる | |
| `--lineage` で `# [lineage] effects: ...` コメントが付与される | |
| エフェクトなし fn で `Pure` が表示される | |
| `cargo test v11800` 6 件通過 | |
| `cargo test --lib` 705 件以上通過 | |
