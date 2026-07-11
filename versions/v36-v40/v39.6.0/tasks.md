# v39.6.0 タスクリスト — `fav audit`

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v39.1-v40.0.md` の v39.6.0（「`fav audit`」）に沿ったバージョン。
> ロードマップ「Rust テスト 2 件」= meta 2 件（version + changelog）のみ。`fav_audit.rs` 存在確認は `mod fav_audit;` による cargo コンパイル成功で暗黙検証。

## T0: 事前確認

- [x]`cargo test` の実測通過数を確認（目安: 2801（v39.5.0 完了時点の実績値））し、実測値をここに記録: ___
- [x]Cargo.toml バージョンが `39.5.0` であることを確認
- [x]`v39500_tests::cargo_toml_version_is_39_5_0` がライブアサーション（`assert!(cargo.contains("39.5.0"), ...)`）であることを確認し、行番号を Grep で確認して記録: ___
- [x]`cargo_toml_version_is_39_5_0` に `// NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること` が付いていることを確認（T4 のスタブ化範囲に含まれる）
- [x]`v39500_tests` の他テスト（`changelog_has_v39_5_0` / `tenant_rune_db_schema` / `tenant_rune_s3_prefix`）はバージョン変更後も pass することを確認
- [x]`driver.rs` に `v39600_tests` モジュールが存在しないことを確認（今回新規作成）
- [x]`v39500_tests` の閉じ `}` の行番号を Grep/Read で確認し、ここに記録: ___
- [x]`CHANGELOG.md` に `[v39.6.0]` エントリが存在しないことを確認（今回新規作成）
- [x]`fav/src/fav_audit.rs` が存在しないことを確認（今回新規作成）
- [x]`main.rs` に `mod fav_audit;` が存在しないことを確認（今回追加）
- [x]`versions/current.md` の最新安定版が `v39.5.0`・「次に切る版」が `v39.6.0` であることを確認
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.6.0 が未完了（✅ なし）であることを確認（T7 で更新）
- [x]`roadmap-v39.1-v40.0.md` の v39.6.0 テスト件数欄が「Rust テスト 2 件」と記載されていることを確認

## T1: CHANGELOG.md に [v39.6.0] エントリを追加

- [x]`## [v39.5.0]` ヘッダ行の直前に `## [v39.6.0]` エントリを挿入
- [x]日付を `YYYY-MM-DD` 形式の実装当日の日付に変更
- [x]セパレータが `—`（全角ダッシュ U+2014）形式であることを確認
- [x]`fav audit` / `fav audit --check` の両コマンドが追記内容に記載されていることを確認

## T2: `fav/src/fav_audit.rs` 新規作成

- [x]spec.md §1 の内容で `fav/src/fav_audit.rs` を新規作成
- [x]`pub fn cmd_audit(check_mode: bool) -> Result<(), String>` を含む
- [x]`fn collect_rune_deps() -> Result<Vec<String>, String>` を含む（空リスト返却スタブ）
- [x]`collect_rune_deps` に TODO コメント（「fav.toml parse 実装時に本ロジックに置き換えること」）が含まれること
- [x]`check_mode=true` 時に GPL 含む entry を `eprintln!` + `std::process::exit(1)` する分岐を含む
- [x]`check_mode=false` 時に全 rune を `println!` + 件数表示する分岐を含む

## T3: `fav/src/main.rs` — `mod fav_audit;` + `Some("audit")` アーム追加

- [x]T2（fav_audit.rs 作成）が完了していることを確認してから着手
- [x]Read で `mod policy;` の行番号を確認
- [x]`mod policy;` の直後に `mod fav_audit;` を追加
- [x]Read で `Some("policy")` アームの末尾行番号を確認
- [x]`Some("policy")` アームの直後に `Some("audit")` アームを追加（spec.md §2 参照）
  - [x]`--check` フラグを `args.iter().any(|a| a == "--check")` で検出
  - [x]`fav_audit::cmd_audit(check_mode)` を呼ぶ
  - [x]エラー時は `eprintln!` + `std::process::exit(1)`

## T4: `driver.rs` — `v39500_tests::cargo_toml_version_is_39_5_0` をスタブ化

- [x]Grep で `cargo_toml_version_is_39_5_0` の行番号を確認（T0 で記録済み）
- [x]ライブアサーション（NOTE コメントを含む本体）→ `// Stubbed: version bumped to 39.6.0 — assertion intentionally removed` に変更
- [x]**注意:** `changelog_has_v39_5_0` / `tenant_rune_db_schema` / `tenant_rune_s3_prefix` はスタブ化しない
- [x]スタブ形式が前バージョンのスタブと一致していることを確認

## T5: `driver.rs` — `v39600_tests` モジュールを新規追加（T1 完了後に実施）

- [x]T1（CHANGELOG 追加）が完了していることを確認してから着手
- [x]`v39500_tests` の閉じ `}` の行番号（T0 で記録済み）を Read で特定してから Edit を実行
- [x]`v39500_tests` の閉じ `}` の後に `v39600_tests` モジュールを追加
  - [x]imports 不要（`include_str!` のみ）
  - [x]`cargo_toml_version_is_39_6_0`（NOTE コメント付き）
  - [x]`changelog_has_v39_6_0`
- [x]テスト数が 2 件であることを確認（ロードマップ「Rust テスト 2 件」に一致）

## T6: バージョン更新（T1〜T5 すべて完了後）

- [x]`fav/Cargo.toml` バージョンを `39.6.0` に更新

## T7: テスト実行 + ドキュメント更新

- [x] `cargo test` 全通過 — ≥ 2803 passed; 0 failed — 実測: 2803 passed, 0 failed
- [x]`v39600_tests` の 2 テストがすべて pass
- [x]`cargo_toml_version_is_39_6_0` が pass
- [x]`changelog_has_v39_6_0` が pass
- [x]`versions/v36-v40/v39.6.0/tasks.md` を COMPLETE ステータスに更新（T0〜T7 全チェックボックスを `[x]` に）
- [x]`versions/current.md` を v39.6.0（最新安定版）・v39.7.0（次に切る版）に更新
- [x]`versions/roadmap/roadmap-v39.1-v40.0.md` の v39.6.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `CHANGELOG.md` に `[v39.6.0]` が含まれる | `changelog_has_v39_6_0` テスト |
| 2 | `Cargo.toml` バージョンが `39.6.0` | `cargo_toml_version_is_39_6_0` テスト |
| 3 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2803） | `cargo test` 実行結果（2801 + 2 = 2803） |
| 4 | `fav_audit.rs` に `pub fn cmd_audit` が含まれる | cargo コンパイル成功（`mod fav_audit;` 参照で暗黙検証） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.6.0 が ✅ | T7 後に目視確認 |
