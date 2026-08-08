# Tasks — v57.2.0 — シークレット管理統合（Vault / AWS Secrets Manager）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.2.0 セクションを確認
- [x] ベーステスト数 3255（v57.1.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `57.1.0` であることを確認（更新前）
- [x] `fav/src/toml.rs` に `SecretsConfig` 構造体が存在しないことを確認（新規追加対象）
- [x] `fav/src/toml.rs` の `FavToml` 構造体に `secrets` フィールドが存在しないことを確認
- [x] `v57200_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v57100_tests` が `driver.rs` に存在することを確認（`v57200_tests` の挿入位置として使用）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"57.1.0"` を期待していることを確認（更新対象）
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` が `"57.1.0"` を期待していることを確認（更新対象）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` が `"57.1.0"` を期待していることを確認（更新対象・rolling）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `57.2.0` に更新
- [x] T2: `fav/src/toml.rs` — `SecretsConfig` 構造体と `list_keys` メソッドを追加
  - [x] `SecretsConfig { provider: String, region: String, bindings: HashMap<String, String> }` 定義
  - [x] `#[derive(Debug, Clone, Default)]` を付与
  - [x] `list_keys(&self) -> Vec<&str>` 実装（ソート済み）
  - [x] `FavToml` 構造体に `secrets: Option<SecretsConfig>` フィールドを追加
  - [x] `parse_fav_toml` に `"[secrets]"` / `"[secrets.bindings]"` セクション解析を追加
  - [x] `parse_fav_toml` の `FavToml` 返却部に `secrets: ...` を追加（provider が空なら `None`）
  - [x] `driver.rs`（1）/ `resolver.rs`（3）/ `checker.rs`（2）の `FavToml { ... }` 直接初期化 6 箇所に `secrets: None` を追加
- [x] T3: `fav/src/driver.rs` — `v57200_tests` モジュールを `v57100_tests` の直前に追加
  - [x] `use crate::toml::SecretsConfig` を使用
  - [x] `make_secrets()` ヘルパー関数（SNOWFLAKE_PASSWORD / KAFKA_API_KEY バインディング）
  - [x] `secrets_provider_config_parsed`: provider / region / bindings.len() / 特定キー値を検証
  - [x] `cmd_secrets_list`: `list_keys()` がソート済みキー名を返すことを検証
- [x] T4: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.1.0"` → `"57.2.0"` に更新
  - [x] failure メッセージも `"should be 57.2.0"` に更新
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.1.0"` → `"57.2.0"` に更新
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.1.0"` → `"57.2.0"` に更新
  - [x] モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x] T5: `cargo build` でコンパイルエラーがないことを確認
- [x] T6: `cargo test` 全通過（**3257 tests passed, 0 failed**）
  - [x] `v57200_tests::secrets_provider_config_parsed` ok
  - [x] `v57200_tests::cmd_secrets_list` ok
  - [x] 既存 3255 件全通過
- [x] T7: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T8: `CHANGELOG.md` に v57.2.0 エントリを追加
- [x] T9: `versions/current.md` を v57.2.0 / 3257 tests に更新
- [x] T10: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.2.0 実績を COMPLETE に更新
  - [x] `3255 + 2 = 3257 tests passed, 0 failed（日付）` を追記
- [x] T11: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.2.0 実績欄も COMPLETE に更新
  - [x] テスト数推移テーブルに v57.2.0 行（3257）を追加

---

## 完了確認

- [x] `secrets_provider_config_parsed` pass
- [x] `cmd_secrets_list` pass
- [x] **3257 tests passed, 0 failed**（ベース 3255 + 2）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/toml.rs` に `SecretsConfig` と `list_keys` が追加されている
- [x] `CHANGELOG.md` に `[v57.2.0]` エントリが追加されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.2.0"` になっている
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.2.0"` になっている（rolling）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.2.0"` になっている（rolling）
- [x] `versions/current.md` が v57.2.0 / 3257 tests を反映
- [x] T10 / T11 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `SecretsConfig` は `toml.rs` の `RbacConfig` の直後、`FavToml` の直前に配置
- `[secrets.bindings]` のパース: `KEY = "value"` 形式の文字列値を読む（`rbac.bindings` の配列とは異なる）
- `list_keys` はソートして返す（HashMap のイテレーション順が非決定的なため）
- `driver.rs`（1）/ `resolver.rs`（3）/ `checker.rs`（2）の `FavToml { ... }` 直接初期化（計 6 箇所）にも `secrets: None` が必要
- `v57100_tests` には `cargo_toml_version_is_57_1_0` テストがないため、v57200_tests 作業での rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
- v57.2.0 COMPLETE 後、次バージョン（v57.3.0）の T0 でこの実装メモを参照すること
