# Tasks: v85.9.0 — 安定化・コードフリーズ

Status: COMPLETE

> 安定化バージョン。新機能追加なし。バグ修正のみ受け入れる。
> MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,947 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85800_tests` が存在することを確認する（v85.8.0 完了済みの証拠）
- [x] `runes/sap-odata/rune.toml` に `name` フィールドが存在することを確認する
- [x] `infra/e2e-demo/sap-odata/docker-compose.yml` が存在することを確認する
- [x] （手動・任意）Docker が利用可能な環境で `docker compose up -d` を実行し、モックサーバーが起動することを確認する
- [x] （手動・任意）`fav.toml [sap]` + `SAP_*` 環境変数を設定し `fav run` で `inject_sap_config` が動作することを確認する（CI 環境外のため任意）

## T1: `rune.toml` の書式確認

- [x] `runes/sap-odata/rune.toml` を読み、`name` フィールドの正確な書式を確認する
- [x] テストの assert 文字列（`content.contains("sap-odata")`）が適切かを判断する

## T2: `mod v85900_tests` を追加

- [x] `mod v85800_tests { ... }` の直後に `#[cfg(test)] mod v85900_tests { ... }` を追加する
- [x] `sap_foundation_rune_toml_has_correct_name` テストを実装する
  - `../runes/sap-odata/rune.toml` を読み込み、`"sap-odata"` が含まれることを確認
- [x] `sap_foundation_docker_compose_has_sap_mock_service` テストを実装する
  - `../infra/e2e-demo/sap-odata/docker-compose.yml` を読み込み、`"sap-mock"` が含まれることを確認

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,949 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.9.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `sap_foundation_rune_toml_has_correct_name` の assert を `content.contains("sap-odata")` から `content.lines().any(|l| l.contains("name") && l.contains("sap-odata"))` に変更 — `name` フィールド行に限定した精密な検証に強化
