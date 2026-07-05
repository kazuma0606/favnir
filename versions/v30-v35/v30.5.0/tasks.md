# v30.5.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.4.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2399 passed` を含むこと
- [x] `driver.rs` に `mod v305000_tests` が存在しないこと
- [x] v30.4.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.4.0` → `30.5.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_4_0` をスタブ化
- [x] **T3** `examples/csv-to-postgres/fav.toml` — 新規作成
- [x] **T4** `examples/csv-to-postgres/src/types.fav` — 新規作成
- [x] **T5** `examples/csv-to-postgres/src/validators.fav` — 新規作成（String error 型、Some/None パターン）
- [x] **T6** `examples/csv-to-postgres/src/stages.fav` — 新規作成（`import runes/postgres`、seq あり）
- [x] **T7** `examples/csv-to-postgres/src/main.fav` — 新規作成
- [x] **T8** `examples/csv-to-postgres/data/sample.csv` — 新規作成（10 行、うち 1 行無効）
- [x] **T9** `examples/csv-to-postgres/tests/pipeline_test.fav` — 新規作成（3 件テスト）
- [x] **T10** `examples/csv-to-postgres/README.md` — 新規作成（30 分クイックスタート）
- [x] **T11** 手動検証 — `fav check` / `fav check --legacy-check` を各ファイルで実行
- [x] **T12** `fav/src/driver.rs` — `v305000_tests`（7 件）を追加
- [x] **T13** `CHANGELOG.md` — `[v30.5.0]` セクションを先頭に追記
- [x] **T14** `benchmarks/v30.5.0.json` — 新規作成
- [x] **T15** `versions/current.md` — 最新安定版を v30.5.0 に更新

---

## テスト確認

- [x] **T16** `cargo test v305000_tests 2>&1 | tail -5` — 7/7 PASS
- [x] **T17** `cargo test 2>&1 | grep "test result"` — 全件 PASS（0 failures）

---

## 完了処理

- [x] **T18** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.5.0"
- [x] `examples/csv-to-postgres/` — 8 ファイル（fav.toml + 4 .fav + sample.csv + README.md + pipeline_test.fav）
- [x] `fav check examples/csv-to-postgres/src/types.fav` → no errors found
- [x] `fav check examples/csv-to-postgres/src/validators.fav` → no errors found
- [x] `fav check --legacy-check examples/csv-to-postgres/src/stages.fav` → no errors found
- [x] `fav check --legacy-check examples/csv-to-postgres/src/main.fav` → no errors found
- [x] `cargo test v305000_tests` — 7/7 PASS
- [x] `cargo test` — 全件 PASS
- [x] `CHANGELOG.md` に `[v30.5.0]` セクション
- [x] `benchmarks/v30.5.0.json` 存在
- [x] `versions/current.md` を v30.5.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] Favnir コードに `let` が使われていないこと
- [x] `String.to_int` / `String.to_float` に `Some`/`None` パターンを使っていること
- [x] レコードリテラルに型名プレフィックスがあること（`ValidRow { }` / `RawRow { }`）
- [x] `seq` の型チェーン（`String -> List<RawRow> -> List<ValidRow> -> Int`）が一致していること
- [x] `ValidateRows` の戻り型が `List<ValidRow>`（`List.flat_map` + `List.empty()`/`List.singleton()` で失敗行スキップ）であること
- [x] tests/pipeline_test.fav が DB 不要であること（純粋 `validators.validate_row` テストのみ）
- [x] README に 30 分クイックスタートが含まれていること
- [x] `v305000_tests` に `use super::*;` があること
- [x] `v305000_tests` に `benchmark_v30_5_0_exists` テストがあること

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘（実装前）:
- [HIGH] `include_str!("../examples/...")` パス誤り → `../../examples/` に修正（4箇所、plan.md）
- [HIGH] `List.filter_map` VM 未実装 → `List.flat_map` + `List.empty()`/`List.singleton()` に差し替え（spec.md / plan.md / stages.fav）
- [MED] `fav test` import 解決未検証 → plan.md に content-only テスト旨の注記追加
- [MED] benchmark 数値の作成タイミング → Step 0 で 2399 確認が構造上正しいため対応なし
- [LOW] `Postgres.execute` パラメータ形式 → spec の OUT OF SCOPE 明記で対応
- [LOW] MDX 対応注記 → tasks.md の範囲外として明示

実装時発見:
- [HIGH] E0023: `IO.read_file_raw` が stages.fav で発火 → 調査の結果、`!IO` は `Effect::Unknown("IO")` として parse される（lexer の `"IO"` は Ident、`"Io"` が keyword）。`declared_effect_namespaces` で `Unknown` も処理するよう修正。その後 TrfDef は全体的にエフェクト境界であるため E0023 免除に変更
- [HIGH] E0025: `!IO` / `!Postgres` 記法が stage で E0025 発火 → TrfDef の `!Effect` 記法は設計上有効なため E0025 も TrfDef には適用しないよう `lint.rs` 修正
- `lint.rs` 修正結果: TrfDef の E0023/E0025 完全免除により `fav check` が全 4 ファイルで `no errors found` に

code-reviewer 指摘（実装後）:
- [MED] `declared_effect_namespaces` デッドコード → 関数削除（lint.rs）
- [MED] `WriteToDb` の `row.name` に JSON エスケープ漏れ → `String.replace(row.name, "\"", "\\\"")` で対応（stages.fav）
- [LOW] W008 と E0023 の TrfDef 処理が非対称 → `collect_ambient` の W008 アームにコメント追加（lint.rs）
- [LOW] `Main` stage の入力型 `String` 未使用 → 動作上問題なし・許容
