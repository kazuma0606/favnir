# Tasks: v89.7.0 — OSS 整備

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,031 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89600_tests` が存在することを確認する（v89.6.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する
- [x] `../CONTRIBUTING.md`（`favnir/CONTRIBUTING.md`）が存在することを確認する（`ls ../CONTRIBUTING.md` で確認）
- [x] `.github/ISSUE_TEMPLATE/` ディレクトリが存在することを確認する（`quality-feedback.md` 等）

## T1: `CONTRIBUTING.md` に SAP Rune エンティティ追加手順を追記

- [x] ファイル末尾に `## SAP Rune — 新エンティティの追加手順` セクションを追加する
- [x] 手順 1: 型定義ファイルの作成（`runes/sap-odata/<entity>.fav`）を記述する
- [x] 手順 2: 関数実装（スタブから実装へ）を記述する
- [x] 手順 3: `sap_odata.fav` への re-export 追加を記述する
- [x] 手順 4: `driver.rs` テスト追加を記述する
- [x] 手順 5: Registry デプロイ（`/deploy-registry`）を記述する
- [x] Favnir コードサンプルに `let` キーワードが混在していないことを確認する（`bind` 使用ルールの遵守確認）

## T2: `.github/ISSUE_TEMPLATE/sap-integration-feedback.md` を作成

- [x] `quality-feedback.md` と同形式のフロントマター（name / about / title / labels / assignees）を記述する
- [x] フィードバック種別チェックリスト（誤動作・型定義不一致・認証エラー・新エンティティリクエスト・その他）を追加する
- [x] 詳細・再現手順・環境情報（Favnir バージョン / SAP バージョン）のセクションを追加する

## T3: `mod v89700_tests` を `driver.rs` に追加

- [x] `mod v89600_tests { ... }` の直後に `#[cfg(test)] mod v89700_tests { ... }` を追加する
- [x] `contributing_has_sap_section` テストを実装する（`"../CONTRIBUTING.md"` に `"SAP Rune"` を確認）
- [x] `issue_template_sap_feedback_exists` テストを実装する（`"../.github/ISSUE_TEMPLATE/sap-integration-feedback.md"` の存在確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,033 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
