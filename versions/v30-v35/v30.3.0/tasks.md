# v30.3.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.2.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2384 passed` を含むこと
- [x] `driver.rs` に `mod v303000_tests` が存在しないこと
- [x] v30.2.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.2.0` → `30.3.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_2_0` をスタブ化
- [x] **T3** `fav/tests/fixtures/multifile_etl/fav.toml` — 新規作成
- [x] **T4** `fav/tests/fixtures/multifile_etl/src/types.fav` — 新規作成
- [x] **T5** `fav/tests/fixtures/multifile_etl/src/validators.fav` — 新規作成（`Some`/`None` パターン）
- [x] **T6** `fav/tests/fixtures/multifile_etl/src/main.fav` — 新規作成（戻り型 `Result<List<ValidRow>, RowError>`）
- [x] **T7** 手動検証 — `fav check` が各ファイルで通ること確認・バグ修正（型名プレフィックス付きレコードリテラルに修正）
- [x] **T8** `fav/src/driver.rs` — `v303000_tests`（7 件）を末尾に追加
- [x] **T9** `CHANGELOG.md` — `[v30.3.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v30.3.0.json` — 新規作成（test_count: 2391）
- [x] **T11** `versions/current.md` — 進行中バージョンを `v30.3.0` に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v303000 2>&1 | tail -5` — 7/7 PASS
- [x] **T13** `cargo test 2>&1 | grep "test result"` — `2391 passed` を含むこと（0 failures）

---

## 完了処理

- [x] **T14** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.3.0"
- [x] `fav/tests/fixtures/multifile_etl/` が 4 ファイルで構成されている（fav.toml + 3 .fav）
- [x] `fav check` が各 .fav ファイルで通ること（手動検証）
- [x] `cargo test` — 2391 tests PASS
- [x] `CHANGELOG.md` に `[v30.3.0]` セクションあり
- [x] `benchmarks/v30.3.0.json` 存在
- [x] `cargo test --bin fav v303000` — 7/7 PASS
- [x] `versions/current.md` を `v30.3.0` に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] フィクスチャの `.fav` コードに `let` が使われていないこと
- [x] `String.to_int` / `String.to_float` に `Some`/`None` パターンを使っていること
- [x] `Pipeline` stage の返値型が `Result<List<ValidRow>, RowError>` であること
- [x] 手動検証（Step 3）が完了し、発見バグが修正されていること

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘（作成前）:
- [HIGH] `String.to_int` は `Option<Int>` 返し → `Ok`/`Err` → `Some`/`None` に修正
- [HIGH] `Pipeline` stage 返値型が `List<ValidRow>` → `Result<List<ValidRow>, RowError>` に修正
- [HIGH] `fav run` / `fav test` がロードマップに存在するが spec に未記載 → スコープ外理由を spec に明記
- [MED] テスト数の前提確認（2384）→ benchmarks/v30.2.0.json と整合確認済み
- [MED] `path.to_str().unwrap()` の OS 差異 → 固定文字列に変更
- [MED] `main.fav` の parse テストがない → 7 件目（`changelog_has_v30_3_0`）追加
- [LOW] `versions/current.md` が完了条件に未記載 → チェックリストに追加
- [LOW] site/ MDX 追加なし → スコープ外として spec に明記

実装時発見:
- [HIGH] パーサーはベア `{ field: value }` を block として扱う → `TypeName { field: value }` が必須
  - `validators.fav`: `{ field: "id", ... }` → `RowError { field: "id", ... }` / `ValidRow { id: id, ... }` に修正
  - `driver.rs` scaffold_postgres_etl_validators: `Err`/`Ok` → `None`/`Some`、ベアレコード → 型付きレコードに修正
  - `driver.rs` scaffold_postgres_etl_stages: `RawRow { ... }` 型名プレフィックス追加
  - `driver.rs` scaffold_postgres_etl_test: `RawRow { ... }` 型名プレフィックス追加
- [LOW] assert! メッセージ内の `{ ... }` は format string と解釈される → `{{ ... }}` にエスケープ
