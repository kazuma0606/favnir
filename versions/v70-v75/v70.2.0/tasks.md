# v70.2.0 タスクリスト — `fav migrate` 完成（構文自動移行ツール）

Date: 2026-08-09
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `70.1.0` であることを確認
- [x] `cargo test` が全 pass（3561 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）
- [x] `resolve_use_effects` が `"v13"` / `"13"` のみを扱っていることを確認

---

## T1: `resolve_use_effects` に `"v35"` 追加

- [x] `fav/src/driver.rs` の `resolve_use_effects` を開く（行 18660 付近）
- [x] `matches!` パターンに `Some("v35") | Some("35")` を追加する
- [x] `cargo test` で既存テスト（3561 件）が全 pass することを確認

---

## T2: `migrate_io_calls_in_source` 追加

- [x] `migrate_effects_in_source` 関数の直後（行 18656 付近）に `migrate_io_calls_in_source` を追加する
- [x] `migrate_io_calls_in_line` ヘルパーも追加する（置換順序: write_file → read_file → println → args）
- [x] 関数が `pub` であることを確認（テストから `super::` でアクセスするため）
- [x] `cargo test` で既存テスト（3561 件）が全 pass することを確認

---

## T3: `cmd_migrate` で IO コール変換を適用

- [x] `cmd_migrate` 内の `use_effects` ブランチ（行 18991 付近）を修正する
- [x] `migrate_effects_in_source` の結果に `migrate_io_calls_in_source` を連鎖適用する
- [x] `else` ブランチ（`migrate_source`）は変更しない
- [x] `cargo test` で既存テスト（3561 件）が全 pass することを確認

---

## T4: `v702000_tests` モジュール追加

- [x] driver.rs の末尾に `mod v702000_tests` を追加する
- [x] `migrate_effect_annotation_to_ctx` テストを実装する
  - `fn run(x: Int) -> Unit !IO` から `!IO` が除去されることを assert
  - 既存実装は ctx パラメータを自動挿入せず W010 警告で手動追加を促す設計のため、`!IO` の除去のみを検証する
- [x] `migrate_io_stdlib_to_ctx_io` テストを実装する
  - 4 種の IO.* コールが `ctx.io.*` に変換されることを assert
  - 変換後に `IO.` が残らないことを assert
- [x] `cargo test v702000` で 2 件 pass することを確認

---

## T5: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"70.1.0"` → `"70.2.0"` に変更する
- [x] driver.rs 内の旧バージョン文字列アサーション（`version = \"70.1.0\"`）を `replace_all: true` で一括更新

---

## T6: CHANGELOG.md 更新

- [x] `CHANGELOG.md` の先頭（v70.1.0 エントリの直前）に v70.2.0 エントリを追加する
- [x] エントリに以下を含める:
  - Added: `migrate_io_calls_in_source`
  - Added: `resolve_use_effects` に `"v35"` / `"35"` 追加
  - Added: `v702000_tests` 2 件（3561 → 3563 tests）

---

## T7: versions/current.md 更新

- [x] `versions/current.md` を開く
- [x] 「進行中バージョン」を `v70.2.0`（fav migrate 完成）に更新する

---

## T8: 最終確認

- [x] `cargo test v702000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3563 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `70.2.0` であることを確認
- [x] `versions/current.md` が正しく更新されていることを確認
- [x] site/ MDX 更新: `site/content/docs/tools/migrate.mdx` が存在しないためスコープ外

---

## コードレビュー指摘対応

- **実装時判明**: `migrate_effects_in_source` は `!IO` を除去するが `ctx:` パラメータを自動挿入しない（W010 警告で手動追加を促す設計）。テストのアサーションを `ctx: AppCtx` 含有から `!IO` 除去確認に修正した。spec-reviewer [MED] 指摘（assert 条件の齟齬）も同時に解消済み。
- **[HIGH] コードレビュー対応**: `infer_ctx_type_from_effect_names` の `has_io` 判定が `effects.contains(&"Io")` のみで `"IO"`（大文字）を認識しない既存バグを修正。`migrate_effects_in_line` は元ファイルの `!IO` から `"IO"` を抽出するため、`has_io` が常に `false` → `!IO` 単体が `CommonCtx` でなく `AppCtx` + W010 に誤判定されていた。`any(|&e| matches!(e, "IO" | "Io"))` に修正。

---

## 完了チェックリスト

- [x] 全タスク（T0〜T8）が完了している
- [x] `migrate_effect_annotation_to_ctx` が pass
- [x] `migrate_io_stdlib_to_ctx_io` が pass
- [x] テスト総数: 3563（+2）
