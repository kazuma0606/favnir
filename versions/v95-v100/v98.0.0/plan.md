# Plan: v98.0.0 — SAP Workflow 1.0 宣言

## 実装順序

### Step 1: `fav/Cargo.toml` バージョン更新

`version = "97.0.0"` → `version = "98.0.0"` に変更する。

### Step 2: `CHANGELOG.md` に v98.0.0 エントリを追加

先頭に `[v98.0.0]` エントリを追加する（`changelog_has_v98_0_0` テストが先に通る必要があるため）。

内容:
- SAP Workflow 1.0 宣言
- v97.1.0〜v97.9.0 の Sprint 成果サマリ
- 4 テスト追加

### Step 3: `MILESTONE.md` に v98.0.0 エントリを追加

既存の v97.0.0 エントリの直後（最新エントリとして先頭）に追加する。

内容:
- v98.0 — SAP Workflow 1.0
- 宣言文
- v97.1.0〜v97.9.0 の達成内容リスト

### Step 4: `README.md` に v98.0 セクションを追加

`## v97.0 — SAP Multi-system 1.0` の直前（最新セクションとして上位）に追加する。

内容:
- `## v98.0 — SAP Workflow 1.0`
- 宣言文・主要成果

### Step 5: `fav/src/driver.rs` に `mod v98000_tests` を追加

5-a. `driver.rs` 全体の `"97.0.0"` 文字列を `"98.0.0"` に一括置換する（`cargo_toml_version_is_97_0_0` テスト名・文字列リテラルを含む全箇所が対象）。

5-b. `mod v97900_tests` の直後に `mod v98000_tests` を追加する（4 テスト）:
- `cargo_toml_version_is_98_0_0`: `include_str!` でバージョン確認
- `changelog_has_v98_0_0`: `include_str!` で CHANGELOG 確認
- `milestone_has_sap_workflow`: `include_str!` で MILESTONE 確認
- `readme_mentions_sap_workflow`: `include_str!` で README 確認

### Step 6: `cargo clean` を実施（★クリーンアップ）

`cd fav && cargo clean` を実行して target/ をクリーンアップする。

### Step 7: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、4,235 tests, 0 failures を確認する。

### Step 8: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v98.0.0` に更新する
- 最新安定版を `v98.0.0` に更新する（テスト数 4,235）
- マイルストーン表に `v98.0 — SAP Workflow 1.0` を追加する

### Step 9: CI 事前確認

Step 7 の `cargo test` 実行後、`target/debug/fav` バイナリが存在することを前提とする（cargo clean 後は cargo test により再ビルドされる）。

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
