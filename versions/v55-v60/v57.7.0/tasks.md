# Tasks — v57.7.0 — マルチテナント分離

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x]`versions/roadmap/roadmap-v57.1-v58.0.md` の v57.7.0 セクションを確認
- [x]`versions/roadmap/roadmap-v55.1-v60.0.md` の v57.7.0 欄が存在することを確認（T10 の更新対象）
- [x]ベーステスト数 3265（v57.6.0 完了時点の実績値）を確認
- [x]`fav/Cargo.toml` が `57.6.0` であることを確認（更新前）
- [x]`v57700_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x]`v57600_tests` が `driver.rs` に存在することを確認（`v57700_tests` の挿入位置として使用）
- [x]`v56300_tests::cargo_toml_version_is_56_3_0` が `"57.6.0"` を期待していることを確認（更新対象）
- [x]`v56900_tests::cargo_toml_version_is_56_9_0` が `"57.6.0"` を期待していることを確認（更新対象）
- [x]`v57000_tests::cargo_toml_version_is_57_0_0` が `"57.6.0"` を期待していることを確認（更新対象・rolling）
- [x]`v57100_tests` 〜 `v57600_tests` に `cargo_toml_version_is_*` が存在しないことを確認（rolling 更新対象外）
- [x]`toml.rs` の `TlsConfig` 直後の位置を確認（`TenancyIsolation` / `TenancyConfig` の挿入位置）
- [x]`toml.rs` の `FavToml` 構造体に `tls: Option<TlsConfig>` フィールドがあることを確認（`tenancy` の挿入位置）
- [x]`driver.rs` / `resolver.rs` / `checker.rs` の `FavToml { ... }` 直接初期化箇所数（計 6 箇所）を確認

---

## 実装タスク

- [x]T1: `fav/Cargo.toml` version を `57.7.0` に更新
- [x]T2: `fav/src/toml.rs` — `TenancyIsolation` / `TenancyConfig` 構造体追加
  - [x]`TenancyIsolation` 構造体（snowflake_schema / kafka_topic_prefix）
  - [x]`TenancyConfig` 構造体（mode / tenant / isolation）+ `is_strict()` メソッド
  - [x]`FavToml` に `pub tenancy: Option<TenancyConfig>` フィールド追加（`tls` の直後）
  - [x]`parse_fav_toml` アキュムレータ宣言 `let mut tenancy_cfg: Option<TenancyConfig> = None;`
  - [x]`parse_fav_toml` セクション検出 `"[tenancy]"` / `"[tenancy.isolation]"` 追加
  - [x]`parse_fav_toml` 処理アーム `"tenancy"` / `"tenancy.isolation"` 追加（expand_env_vars 適用）
  - [x]`parse_fav_toml` 返却部に `tenancy: tenancy_cfg,` 追加
- [x]T3: `FavToml` 直接初期化 6 箇所に `tenancy: None` 追加
  - [x]`driver.rs`（1 箇所）
  - [x]`resolver.rs`（3 箇所）
  - [x]`middle/checker.rs`（2 箇所）
- [x]T4: `fav/src/driver.rs` — `v57700_tests` モジュールを `v57600_tests` の直前に追加
  - [x]`use crate::toml::{TenancyConfig, TenancyIsolation};` import
  - [x]`make_tenancy()` ヘルパー関数
  - [x]`tenancy_config_parsed` テスト: 全フィールド検証
  - [x]`tenancy_strict_enforced` テスト: is_strict() true / false 検証
- [x]T5: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x]`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.6.0"` → `"57.7.0"` に更新
  - [x]failure メッセージも `"should be 57.7.0"` に更新
  - [x]`v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.6.0"` → `"57.7.0"` に更新
  - [x]`v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.6.0"` → `"57.7.0"` に更新
  - [x]モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x]T6: `cargo build` でコンパイルエラーがないことを確認
- [x]T7: `cargo test` 全通過（**3267 tests passed, 0 failed**）
  - [x]`v57700_tests::tenancy_config_parsed` ok
  - [x]`v57700_tests::tenancy_strict_enforced` ok
  - [x]既存 3265 件全通過
- [x]T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x]T9: `CHANGELOG.md` に v57.7.0 エントリを追加
- [x]T10: `versions/current.md` を v57.7.0 / 3267 tests に更新
- [x]T11: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.7.0 実績を COMPLETE に更新
  - [x]`3265 + 2 = 3267 tests passed, 0 failed（2026-07-28）` を追記
- [x]T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.7.0 実績欄も COMPLETE に更新
  - [x]テスト数推移テーブルに v57.7.0 行（3267）を追加

---

## 完了確認

- [x]`tenancy_config_parsed` pass
- [x]`tenancy_strict_enforced` pass
- [x]**3267 tests passed, 0 failed**（ベース 3265 + 2）
- [x]`cargo clippy -- -D warnings` クリーン
- [x]`CHANGELOG.md` に `[v57.7.0]` エントリが追加されている
- [x]`v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.7.0"` になっている
- [x]`v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.7.0"` になっている（rolling）
- [x]`v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.7.0"` になっている（rolling）
- [x]`versions/current.md` が v57.7.0 / 3267 tests を反映
- [x]T11 / T12 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `TenancyIsolation` は `TenancyConfig` の直前（`TlsConfig` の直後）に配置
- アキュムレータは `Option<TenancyConfig>` パターン（`rbac_cfg` / `tls_cfg` と同様）
- `[tenancy.isolation]` 検出時、`tenancy_cfg` がまだ None なら `get_or_insert_with` で初期化してから `isolation` を設定する
- 文字列値には必ず `expand_env_vars` を適用
- `v57600_tests` 〜 `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
