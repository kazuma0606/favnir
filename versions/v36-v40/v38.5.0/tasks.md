# v38.5.0 タスクリスト — `fav explain --verbose` LLM 拡張

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v38.1-v39.0.md` の v38.5.0（「`fav explain --verbose` LLM 拡張」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2760（v38.4.0 完了時点の実績値））し、実測値をここに記録: 2760
- [x] Cargo.toml バージョンが `38.4.0` であることを確認
- [x] `v38400_tests::cargo_toml_version_is_38_4_0` がライブアサーション（`assert!(cargo.contains("38.4.0"), ...)`）であることを確認し、行番号を記録: 43745
- [x] `v38400_tests` の他 5 テスト（`changelog_has_v38_4_0` / `lsp_ai_enabled_when_configured` / `lsp_ai_disabled_by_default` / `lsp_ai_explicit_false` / `lsp_ai_not_leaked_to_other_section`）はバージョン変更後も pass することを確認
- [x] `driver.rs` に `v38500_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `v38400_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: 43784（`mod v38400_tests` の後ろの `}` を必ず確認、行番号ハードコード禁止）
- [x] `CHANGELOG.md` に `[v38.5.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `fav/src/explain_verbose.rs` が存在しないことを確認（今回新規作成）
- [x] `main.rs` に `pub(crate) mod explain_verbose;` が存在しないことを確認（今回追加）
- [x] `main.rs` の `Some("explain")` アーム内の `if args.get(2) ... == Some("compiler")` ブロックの `return;` 行番号を確認し、ここに記録: 740（`--verbose` チェックをその直後に挿入）
- [x] `versions/current.md` の最新安定版が `v38.4.0`・次バージョンが `v38.5.0` であることを確認
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.5.0 が未完了（✅ なし）であることを確認（T7 で更新）

## T1: CHANGELOG.md に [v38.5.0] エントリを追加

- [x] `## [v38.4.0]` の直前に `## [v38.5.0]` エントリを挿入
- [x] 日付を `2026-07-10` に設定
- [x] セパレータが `—`（全角ダッシュ U+2014）形式であることを確認

## T2: `fav/src/explain_verbose.rs` 新規作成

- [x] spec.md §1 の内容で `fav/src/explain_verbose.rs` を新規作成
- [x] `pub fn explain_verbose(error_code: &str, location: &str) -> String` を含む
- [x] `fn base_explanation` — E0001 / E0007 / E0008 + デフォルトの match を含む
- [x] `location` が空のとき Context ブロックをスキップ
- [x] 出力に `Fix suggestion: [LLM stub — v38.7.0 で本実装予定]` を含む

## T3: `fav/src/main.rs` — `pub(crate) mod explain_verbose;` 追加

- [x] T2（explain_verbose.rs 作成）が完了していることを確認してから着手
- [x] Read で `pub(crate) mod generate_csv;` の行番号を確認（line 63）
- [x] `pub(crate) mod generate_csv;` の直後に `pub(crate) mod explain_verbose;` を追加

## T4: `fav/src/main.rs` — `--verbose` 分岐追加

- [x] T2・T3 が完了していることを確認してから着手
- [x] Read で `Some("explain")` アーム内の `if args.get(2) ... == Some("compiler")` ブロックの `return;` 行番号を確認（line 740）
- [x] その `return;` の**直後**（`if args.iter().any(|a| a == "--sla")` の直前）に `--verbose` チェックブロックを追加（spec.md §2 のコードブロックに従う）
- [x] **注意**: `compiler` チェックの**後**に挿入すること（前に入れると `fav explain compiler --verbose` が `--verbose` に先に反応してしまう）
  - [x] `args.iter().any(|a| a == "--verbose")` で検出
  - [x] `error_code`: `--` 以外の最初の引数（未指定時 `"E0001"`）
  - [x] `location`: `--` 以外の2番目の引数（未指定時 `""`）
  - [x] `println!("{}", explain_verbose::explain_verbose(error_code, location));` で出力 + `return;`

## T5: `driver.rs` — `v38400_tests::cargo_toml_version_is_38_4_0` をスタブ化

- [x] Read で `cargo_toml_version_is_38_4_0` の行番号を確認（43745）
- [x] ライブアサーション → `// Stubbed: version bumped to 38.5.0 — assertion intentionally removed` に変更
- [x] **注意:** `changelog_has_v38_4_0` / `lsp_ai_*` テストはスタブ化しない
- [x] スタブ形式が前バージョンのスタブと一致していることを確認

## T6: `driver.rs` — `v38500_tests` モジュールを新規追加（T1・T2 完了後に実施）

- [x] T1（CHANGELOG 追加）と T2（explain_verbose.rs 作成）が完了していることを確認してから着手
- [x] `v38400_tests` の閉じ `}` の行番号（Grep/Read で確認済み: 43784）を特定してから Edit を実行
- [x] `v38400_tests` の閉じ `}` の後に `v38500_tests` モジュールを追加（4 テスト）
  - [x] `cargo_toml_version_is_38_5_0`
  - [x] `changelog_has_v38_5_0`
  - [x] `explain_verbose_basic`（`explain_verbose("E0001", "")` が `"E0001"` + `"Fix suggestion"` を含む）
  - [x] `explain_verbose_with_location`（`explain_verbose("E0001", "main.fav:12")` が `"Context"` + `"main.fav:12"` を含む）

## T7: バージョン更新（T1〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `38.5.0` に更新

## T8: テスト実行

- [x] T7（Cargo.toml 更新）が完了していることを確認してから着手
- [x] `cargo test` 全通過 — ≥ 2764 passed; 0 failed — 実測: 2764 passed, 0 failed
- [x] `v38500_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_38_5_0` が pass
- [x] `changelog_has_v38_5_0` が pass
- [x] `explain_verbose_basic` が pass
- [x] `explain_verbose_with_location` が pass

## T9: ドキュメント更新（T8 完了後）

- [x] `versions/v36-v40/v38.5.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）
- [x] `versions/current.md` を v38.5.0（最新安定版）・v38.6.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.5.0 を完了済みにマーク（✅）し、テスト件数を 4 件に更新
- [x] roadmap の v38.5.0 行を Read で確認し ✅ が含まれることをここに記録: ✅ 確認: ### v38.5.0 — `fav explain --verbose` LLM 拡張 ✅
- [x] roadmap の v38.5.0 行を Read で確認し「4 件」が含まれることをここに記録: テスト件数 4 件確認: **完了条件**: Rust テスト 4 件（meta 2 件 + 機能 2 件）（2764 tests passed, 0 failed）
- [x] `versions/current.md` を Read で確認し「v38.5.0」が最新安定版として含まれることをここに記録: 確認: `**v38.5.0** — fav explain --verbose LLM 拡張実装（2026-07-10）`

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `explain_verbose.rs` に `pub fn explain_verbose` が含まれる | `explain_verbose_basic` テスト ✅ |
| 2 | `explain_verbose("E0001", "")` がエラーコードと Fix suggestion を含む文字列を返す | `explain_verbose_basic` テスト ✅ |
| 3 | `location` 指定時に Context ブロックが出力に含まれる | `explain_verbose_with_location` テスト ✅ |
| 4 | `CHANGELOG.md` に `[v38.5.0]` が含まれる | `changelog_has_v38_5_0` テスト ✅ |
| 5 | `Cargo.toml` バージョンが `38.5.0` | `cargo_toml_version_is_38_5_0` テスト ✅ |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2764） | 実測: 2764 passed, 0 failed ✅ |
| 7 | `roadmap-v38.1-v39.0.md` の v38.5.0 が ✅ かつテスト件数が 4 件 | 更新済み ✅ |
| 8 | `versions/current.md` が v38.5.0（最新安定版）に更新されている | 更新済み ✅ |
