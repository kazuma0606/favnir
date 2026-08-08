# Tasks — v57.1.0 — RBAC（ロールベースアクセス制御）for Rune

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.1.0 セクションを確認
- [x] ベーステスト数 3252（v57.0.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `57.0.0` であることを確認（更新前）
- [x] `fav/src/toml.rs` に `RbacConfig` 構造体が存在しないことを確認（新規追加対象）
- [x] `fav/src/toml.rs` の `FavToml` 構造体に `rbac` フィールドが存在しないことを確認
- [x] `fav/src/error_catalog.rs` に `E0424` が存在しないことを確認（新規追加対象）
- [x] `v57100_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v57000_tests` が `driver.rs` に存在することを確認（`v57100_tests` の挿入位置として使用）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"57.0.0"` を期待していることを確認（更新対象）
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` が `"57.0.0"` を期待していることを確認（更新対象）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `57.1.0` に更新
- [x] T2: `fav/src/toml.rs` — `RbacConfig` 構造体と `is_allowed` メソッドを追加
  - [x] `RbacConfig { roles: Vec<String>, bindings: HashMap<String, Vec<String>> }` 定義
  - [x] `#[derive(Debug, Clone, Default)]` を付与
  - [x] `is_allowed(&self, rune: &str, role: &str) -> bool` 実装
  - [x] `FavToml` 構造体に `rbac: Option<RbacConfig>` フィールドを追加
  - [x] `parse_fav_toml` に `"[security.rbac]"` / `"[security.rbac.bindings]"` セクション解析を追加
  - [x] `parse_fav_toml` の `FavToml` 返却部に `rbac: rbac_cfg` を追加
  - [x] `driver.rs` / `resolver.rs` / `checker.rs` の `FavToml { ... }` 直接初期化に `rbac: None` を追加
- [x] T3: `fav/src/error_catalog.rs` — E0424 追加
  - [x] E0423 エントリの直後に E0424（RBAC access denied）を挿入
- [x] T4: `fav/src/driver.rs` — `v57100_tests` モジュールを `v57000_tests` の直前に追加
  - [x] `use crate::toml::RbacConfig` を使用（`use super::*` 不要）
  - [x] `make_rbac()` ヘルパー関数（snowflake → writer/admin のみ許可）
  - [x] `rbac_access_denied`: `is_allowed("snowflake", "reader")` → false
  - [x] `rbac_access_granted`: `is_allowed("snowflake", "writer")` → true
- [x] T5: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.0.0"` → `"57.1.0"` に更新
  - [x] failure メッセージも `"should be 57.1.0"` に更新
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.0.0"` → `"57.1.0"` に更新
  - [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値（rolling）も `"57.0.0"` → `"57.1.0"` に更新（実装時判明）
  - [x] モジュール名・関数名は変更しない（慣例）

---

## テスト・検証

- [x] T6: `cargo build` でコンパイルエラーがないことを確認
- [x] T7: `cargo test` 全通過（**3255 tests passed, 0 failed**）
  - [x] `v57100_tests::rbac_access_denied` ok
  - [x] `v57100_tests::rbac_access_granted` ok
  - [x] `v57100_tests::rbac_unrestricted_rune` ok
  - [x] 既存 3252 件 + コードレビュー追加 1 件全通過
- [x] T8: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T9: `CHANGELOG.md` に v57.1.0 エントリを追加
- [x] T10: `versions/current.md` を v57.1.0 / 3255 tests に更新
- [x] T11: `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.1.0 実績を COMPLETE に更新
  - [x] `3252 + 3 = 3255 tests passed, 0 failed（2026-07-27）` を追記
  - [x] `ベース 3250 + 2 = 3252` を `ベース 3252 + 3 = 3255` に修正（推定値の補正）
- [x] T12: `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.1.0 実績欄も COMPLETE に更新
  - [x] テスト数推移テーブルに v57.1.0 行（3255）を追加

---

## 完了確認

- [x] `rbac_access_denied` pass
- [x] `rbac_access_granted` pass
- [x] **3255 tests passed, 0 failed**（ベース 3252 + 3）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/src/toml.rs` に `RbacConfig` と `is_allowed` が追加されている
- [x] `fav/src/error_catalog.rs` に `E0424` エントリが追加されている
- [x] `CHANGELOG.md` に `[v57.1.0]` エントリが追加されている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"57.1.0"` になっている
- [x] `v56900_tests::cargo_toml_version_is_56_9_0` の期待値が `"57.1.0"` になっている（rolling）
- [x] `v57000_tests::cargo_toml_version_is_57_0_0` の期待値が `"57.1.0"` になっている（rolling）
- [x] `versions/current.md` が v57.1.0 / 3255 tests を反映
- [x] T11 / T12 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `RbacConfig` は `FavToml` 定義の直前に配置（`toml.rs` の構造体群の末尾）
- `[security.rbac.bindings]` のパース: `key = ["role1", "role2"]` 形式の配列値を読む
- `parse_fav_toml` 返却部の `FavToml { ... }` はコンパイラが全フィールド要求するため
  `rbac: rbac_cfg` の追加を忘れると即座にコンパイルエラーで検出できた
- `driver.rs` / `resolver.rs` / `checker.rs` の `FavToml { ... }` 直接初期化（5箇所）にも `rbac: None` が必要だった
- `v57000_tests::cargo_toml_version_is_57_0_0` も rolling バージョンチェックのため更新が必要（spec T5 に記載なかったが実装時に判明）
  → 今後の spec の T5 には「v57000_tests の期待値更新」も含めること
- サイトドキュメント（`enterprise/rbac.mdx` 等）は v57.8.0 で対応（v57.1.0 スコープ外）
- checker 統合（E0424 発火）・`fav run --role` CLI フラグは v57.2.0 以降に延期（v57.1.0 スコープ外）
