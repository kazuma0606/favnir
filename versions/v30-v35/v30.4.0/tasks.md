# v30.4.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.3.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2391 passed` を含むこと
- [x] `driver.rs` に `mod v304000_tests` が存在しないこと
- [x] v30.3.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.3.0` → `30.4.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_3_0` をスタブ化
- [x] **T3** `fav/tests/fixtures/multifile_rune_import/fav.toml` — 新規作成
- [x] **T4** `fav/tests/fixtures/multifile_rune_import/src/types.fav` — 新規作成（Rune import なし）
- [x] **T5** `fav/tests/fixtures/multifile_rune_import/src/validators.fav` — 新規作成（Rune import なし、String error 型）
- [x] **T6** `fav/tests/fixtures/multifile_rune_import/src/stages.fav` — 新規作成（`import runes/postgres`、純粋関数のみ）
- [x] **T7** `fav/tests/fixtures/multifile_rune_import/src/main.fav` — 新規作成（`import runes/postgres` — 2 つ目）
- [x] **T8** 手動検証 — `fav check` が各ファイルで `no errors found` を返すこと確認
- [x] **T9** `fav/src/driver.rs` — `v304000_tests`（8 件）を追加
- [x] **T10** `CHANGELOG.md` — `[v30.4.0]` セクションを先頭に追記
- [x] **T11** `benchmarks/v30.4.0.json` — 新規作成（tests_passed: 2399）
- [x] **T12** `versions/current.md` — 最新安定版を v30.4.0 に更新

---

## テスト確認

- [x] **T13** `cargo test v304000_tests` — 8/8 PASS
- [x] **T14** `cargo test 2>&1 | grep "test result"` — 2399 passed（0 failures）

---

## 完了処理

- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.4.0"
- [x] `fav/tests/fixtures/multifile_rune_import/` — 5 ファイル（fav.toml + 4 .fav）
- [x] `fav check` が各 `.fav` ファイルで通ること（手動検証）
- [x] `stages.fav` と `main.fav` 両方に `import runes/postgres` あり
- [x] `validators.fav` に `import runes/` なし
- [x] `cargo test v304000_tests` — 8/8 PASS
- [x] `cargo test` — 2399 passed、0 failures
- [x] `CHANGELOG.md` に `[v30.4.0]` セクション
- [x] `benchmarks/v30.4.0.json` 存在
- [x] `versions/current.md` を v30.4.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] フィクスチャの `.fav` コードに `let` が使われていないこと
- [x] `String.to_int` / `String.to_float` に `Some`/`None` パターンを使っていること（validators.fav）
- [x] レコードリテラルに型名プレフィックスがあること（`ValidRow { }` / `RawRow { }`）
- [x] `ValidateRows` 戻り型が `Result<List<ValidRow>, RowError>` であること（v30.3.0 [HIGH] 対応）
- [x] `v304000_tests` に `use super::*;` があること（v30.3.0 [LOW] 対応）
- [x] `v304000_tests` に `benchmark_v30_4_0_exists` テストがあること（v30.3.0 [LOW] 対応）

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘（実装前）:
- [HIGH] seq 型合成の不整合 → `seq EtlPipeline` を削除、純粋関数構成に変更
- [HIGH] 手動検証がファイル単体のみ → `fav check --dir` を追加。ただし調査の結果、`--dir` モード（legacy checker）は CWD からの相対パス解決に失敗するため、各ファイル単体チェックが現実的と判断
- [MED] spec.md 背景記述が v30.3.0 と不一致 → 「v30.2.0 スキャフォールド」に修正
- [LOW] types.fav テストなし → `multifile_rune_import_types_fav_exists` 追加（8 件目）

実装時発見:
- [HIGH] `IO.read_file_raw` / `Postgres.execute` を stage 内で使うと E0023 (ambient effect call) → `fav check` (non-legacy) が失敗
  → stages.fav / main.fav を純粋関数（SQL 文字列生成）のみに変更してE0023 回避
- [HIGH] `fav check --dir` (legacy checker) は CWD 相対でパス解決するため、fixture の `import runes/postgres` が E0213 失敗 → `fav check <file>` 単体で検証
- v30.3.0 code-reviewer [HIGH]: `scaffold_postgres_etl_stages` の `ValidateRows` 戻り型を修正（同時対応）
- v30.3.0 code-reviewer [MED]: `benchmarks/v30.3.0.json` の `tests_passed: 2347` → `2391` に修正（同時対応）
