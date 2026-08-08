# Tasks — v57.3.0 — TLS / mTLS サポート（HTTP / gRPC Rune）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.3.0 セクションを確認
- [x] ベーステスト数 3257（v57.2.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `57.2.0` であることを確認（更新前）
- [x] `fav/src/toml.rs` に `TlsConfig` 構造体が存在しないことを確認（新規追加対象）
- [x] `fav/src/toml.rs` の `FavToml` 構造体に `tls` フィールドが存在しないことを確認
- [x] `v57300_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v57200_tests` が `driver.rs` に存在することを確認（`v57300_tests` の挿入位置として使用）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"57.2.0"` を期待していることを確認（更新対象）
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` が `"57.2.0"` を期待していることを確認（更新対象）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` が `"57.2.0"` を期待していることを確認（更新対象・rolling）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `57.3.0` に更新
- [x] T2: `fav/src/toml.rs` — `TlsConfig` 構造体と `is_mtls` メソッドを追加
  - [x] `TlsConfig { ca_cert: Option<String>, tls_cert: Option<String>, tls_key: Option<String>, verify: bool }` 定義
  - [x] `#[derive(Debug, Clone, Default)]` を付与
  - [x] `is_mtls(&self) -> bool` 実装（tls_cert と tls_key 両方 Some のとき true）
  - [x] `FavToml` 構造体に `tls: Option<TlsConfig>` フィールドを追加（secrets フィールドの直後）
  - [x] `parse_fav_toml` に `"[security.tls]"` セクション解析を追加
  - [x] 各フィールドのパース（`expand_env_vars` 適用）: `ca_cert` / `tls_cert` / `tls_key` / `verify`
  - [x] `verify` は `val == "true"` の場合のみ `true`
  - [x] `TlsConfig::default().verify` が `false` であることを確認（`#[derive(Default)]` で自動保証）
  - [x] `parse_fav_toml` の `FavToml` 返却部に `tls: tls_cfg` を追加
  - [x] `driver.rs`（1）/ `resolver.rs`（3）/ `checker.rs`（2）の `FavToml { ... }` 直接初期化 6 箇所に `tls: None` を追加
- [x] T3: `fav/src/driver.rs` — `v57300_tests` モジュールを `v57200_tests` の直前に追加
  - [x] `use crate::toml::TlsConfig` を使用
  - [x] `make_tls()` ヘルパー関数（ca_cert / tls_cert / tls_key / verify: true）
  - [x] `tls_config_parsed`: 全フィールドを `as_deref()` / assert で検証
  - [x] `mtls_cert_injected`: `is_mtls()` が true を返すことを検証
- [x] T4: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.2.0"` → `"57.3.0"` に更新
  - [x] failure メッセージも `"should be 57.3.0"` に更新
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.2.0"` → `"57.3.0"` に更新
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.2.0"` → `"57.3.0"` に更新
  - [x] モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x] T5: `cargo build` でコンパイルエラーがないことを確認
- [x] T6: `cargo test` 全通過（**3259 tests passed, 0 failed**）
  - [x] `v57300_tests::tls_config_parsed` ok
  - [x] `v57300_tests::mtls_cert_injected` ok
  - [x] 既存 3257 件全通過
- [x] T7: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T8: `CHANGELOG.md` に v57.3.0 エントリを追加
- [x] T9: `versions/current.md` を v57.3.0 / 3259 tests に更新
- [x] T10: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.3.0 実績を COMPLETE に更新
  - [x] `3257 + 2 = 3259 tests passed, 0 failed（日付）` を追記
- [x] T11: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.3.0 実績欄も COMPLETE に更新
  - [x] テスト数推移テーブルに v57.3.0 行（3259）を追加

---

## 完了確認

- [x] `tls_config_parsed` pass
- [x] `mtls_cert_injected` pass
- [x] **3259 tests passed, 0 failed**（ベース 3257 + 2）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/toml.rs` に `TlsConfig` と `is_mtls` が追加されている
- [x] `CHANGELOG.md` に `[v57.3.0]` エントリが追加されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.3.0"` になっている
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.3.0"` になっている（rolling）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.3.0"` になっている（rolling）
- [x] `versions/current.md` が v57.3.0 / 3259 tests を反映
- [x] T10 / T11 のロードマップ更新（実績 COMPLETE）が完了している

> **注**: `tls_config_parsed` / `mtls_cert_injected` は構造体直接構築による検証。
> `parse_fav_toml` のパースパス（TOML 文字列 → TlsConfig）の統合テストは後続バージョンで追加予定。

---

## 実装メモ

- `TlsConfig` は `toml.rs` の `SecretsConfig` の直後、`FavToml` の直前に配置
- アキュムレータは `Option<TlsConfig>` パターン（`let mut tls_cfg: Option<TlsConfig> = None`）— RbacConfig と同様
- `[security.tls]` 検出時に `section = "security.tls"` に遷移。フィールドは `tls_cfg.get_or_insert_with(TlsConfig::default)` を使って取得
- `verify` の Default は `false`（Rust 標準の bool::default()）。TOML に `verify = true` と明示した場合のみ `true`
- 文字列値には必ず `expand_env_vars` を適用（v57.2.0 code review 教訓）
- `driver.rs` / `resolver.rs` / `checker.rs` の `FavToml { ... }` 直接初期化（計 6 箇所）にも `tls: None` が必要
- `v57100_tests` / `v57200_tests` には `cargo_toml_version_is_*` テストがないため、rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
