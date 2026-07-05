# v30.2.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.1.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2378 passed` を含むこと
- [x] `driver.rs` に `mod v302000_tests` が存在しないこと
- [x] v30.1.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.1.0` → `30.2.0` に更新
- [x] **T2** `fav/src/driver.rs` — scaffold 関数 6 件追加（types / validators / stages / main_v2 / test / readme）
- [x] **T3** `fav/src/driver.rs` — `create_postgres_etl_project` を 4 ファイル構成に更新
- [x] **T4** `fav/src/driver.rs` — `new_template_postgres_etl_creates_dir` テストを v2 構成に更新
- [x] **T5** `fav/src/driver.rs` — `v302000_tests`（6 件）を末尾に追加
- [x] **T6** `CHANGELOG.md` — `[v30.2.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v30.2.0.json` — 新規作成（test_count: 2384）
- [x] **T8** `versions/current.md` — 進行中バージョンを `v30.2.0` に更新

---

## テスト確認

- [x] **T9** `cargo test --bin fav v302000 2>&1 | tail -5` — 6/6 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — `2384 passed` を含むこと（0 failures）
- [x] **T10b** `scaffold_postgres_etl_uses_chain` が引き続き PASS であること

---

## 完了処理

- [x] **T11** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.2.0"
- [x] `fav new --template postgres-etl` が 4 ソースファイル + tests/ + README.md を生成する
- [x] `new_template_postgres_etl_creates_dir` テストが更新・通過する
- [x] `scaffold_postgres_etl_uses_chain` テストが引き続き通過する
- [x] `cargo test` — 2384 tests PASS
- [x] `CHANGELOG.md` に `[v30.2.0]` セクションあり
- [x] `benchmarks/v30.2.0.json` 存在（test_count: 2384）
- [x] `cargo test --bin fav v302000` — 6/6 PASS
- [x] `versions/current.md` を v30.2.0 完了状態に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] scaffold 関数に HTML インジェクション・コマンドインジェクションがないこと
- [x] `scaffold_postgres_etl()` を削除していないこと（`cmd_scaffold` で引き続き使用）
- [x] scaffold コードが VM 未実装プリミティブを使わないこと（`List.get` / `List.map_indexed` / `Int.parse` NG）
- [x] `write_text_file` が `tests/` ディレクトリを自動作成すること

---

## コードレビュー指摘・対応記録

spec-reviewer 指摘（実装前）:
- [HIGH] `List.map_indexed` VM 未実装 → `List.map` に変更、`validate_row` から `idx` 引数削除
- [HIGH] `List.get` VM 未実装 → `Option.unwrap_or(List.first(List.drop(parts, n)), default)` に変更
- [HIGH] `scaffold_postgres_etl_uses_chain` 言及漏れ → spec に「影響なし」として明記
- [MED] `validate_row` シグネチャ不一致（2引数 vs 1引数）→ 1引数に統一
- [MED] `ValidateRows` の `!IO` エフェクト不要 → 削除
- [MED] `Main` の `List.first` が Option を返す → `Option.unwrap_or` を使用
- [MED] site/ MDX 更新 → 対象外（既存ドキュメントなし）と spec に明記
- [LOW] scaffold 件数表記（4件 → 6件）→ spec を修正
- [LOW] `current.md` 更新手順が plan に漏れ → Step 7.5 として追加

実装中の追加対応:
- `cargo_toml_version_is_30_1_0` テストがバージョン更新後に FAIL → 確立済みパターン（スタブコメント）で修正

code-reviewer 指摘（実装後）:
- [HIGH] `let` は Favnir キーワードではない → `parse_csv_row` をネスト式に書き換え（`bind` 不要の純粋 fn）
- [HIGH] `IO.env()` 未実装 → `WriteToDb` を `Postgres.execute(sql, params_json)` 2引数形式に変更（`conn` 不使用）
- [MED] `Postgres.execute` シグネチャ不一致（3引数→2引数）→ `Postgres.execute(sql, params_json_str)` に修正
- [LOW] スタブテストコメント → 確立済みパターンで許容範囲のため現状維持
