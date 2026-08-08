# v59.5.0 Tasks — Migration Toolkit（v1 → Enterprise マイグレーション）

Date: 2026-07-30
Status: COMPLETE（2026-07-30）— 3318 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3316 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.4.0"` であることを確認
- [x] `fav/src/driver.rs` に `cmd_migrate_dry_run` がまだ存在しないことを確認
- [x] `fav/src/driver.rs` に `migrate_enterprise_import` がまだ存在しないことを確認
- [x] `fav/src/driver.rs` に `v59500_tests` がまだ存在しないことを確認
- [x] `grep -c '59\.4\.0' fav/src/driver.rs` でローリング文字列件数を確認（14 件: assertion 7 件 + failure メッセージ 7 件）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.4.0"` → `"59.5.0"`

---

## T2: roadmap 更新

- [x] `roadmap-v59.1-v60.0.md` の v59.6.0 ベース数を「着手時に更新」→ `3318` に確定

---

## T3: driver.rs に cmd_migrate_dry_run 追加

- [x] `cmd_migrate_dry_run() -> String` を `cmd_marketplace_publish` の直後に追加
  - サンプルソース（`import rune "kafka"` 含む）を内部で保持
  - W035 WARN・TLS WARN・RBAC INFO・multi-env INFO を出力
  - 戻り値: `String`

---

## T4: driver.rs に migrate_enterprise_import 追加

- [x] `migrate_enterprise_import(src: &str) -> String` を `cmd_migrate_dry_run` の直後に追加
  - `import rune "X"` → `import X` の変換（1 行単位、インデント保持）
  - 末尾 `\n` を元ソースに合わせて保持

---

## T5: driver.rs テストモジュール追加

- [x] **注意**: T3・T4（関数追加）を先に行うこと
- [x] `v59500_tests` モジュールを `v59400_tests` の直前に挿入
  - [x] テスト関数名 `cmd_migrate_dry_run` が pub fn と同名 → `use super::` を使わず `super::` 修飾のみで呼び出す
  - [x] `cmd_migrate_dry_run` テスト: `super::cmd_migrate_dry_run()` が `[WARN]` / `import rune` / `RBAC` を含むことを検証
  - [x] `cmd_migrate_auto_fix_import` テスト: `super::migrate_enterprise_import(src)` が `import kafka` を含み `import rune "kafka"` を含まないことを検証

---

## T6: driver.rs ローリングチェック更新

- [x] `version = \"59.4.0\"` → `\"59.5.0\"` に一括更新（7 件）
- [x] failure メッセージ 7 件を `"59.5.0"` に更新（詳細は plan.md Step 6 参照）
  - 通常 5 件: `"Cargo.toml version should be 59.4.0, got: {}"` → `"59.5.0, got: {}"`
    - `cargo_toml_version_is_59_0_0` / `v58900` / `v58000` / `v57900` / `v56300`
  - **特殊書式** 2 件（`rolling check from` サフィックスあり）:
    - `cargo_toml_version_is_57_0_0`: `"Cargo.toml version should be 59.4.0 (rolling check from v57.0.0), got: {}"` → `"59.5.0 (rolling check from v57.0.0), got: {}"`
    - `cargo_toml_version_is_56_9_0`: `"Cargo.toml version should be 59.4.0 (rolling check from v56.9.0), got: {}"` → `"59.5.0 (rolling check from v56.9.0), got: {}"`
  - **注意**: `v59000_tests` は rolling check あり（対象）。`v59100_tests`〜`v59400_tests` は rolling check なし（対象外）

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v59500_tests::cmd_migrate_dry_run` pass を確認
- [x] `v59500_tests::cmd_migrate_auto_fix_import` pass を確認
- [x] 総テスト数 **3318** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v59.5.0 エントリを追加
- [x] `versions/current.md` を v59.5.0 / 3318 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.5.0 実績欄を更新
- [x] v59.6.0 ベース数を実績値（3318）に確定（T2 で修正済みであることを確認）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

実装中に発見した既存テスト 9 件の不具合を修正（v59.5.0 実装の副産物）:
- `vm.rs` の `clear_state_value_store` が `STATE_VALUE_STORE` のみをクリアし `STATE_STORE` を残していたバグ → 両方クリアに修正
- 予約語 `test` を関数名に使用（`fn test(...)` → `fn check_val(...)`）
- interface body で `fn name(self) -> Type` 構文（非サポート）→ `name: Self -> Type` に修正
- `impl TypeName : InterfaceName` → `impl InterfaceName for TypeName` に修正
- `where T: Interface` → `T with Interface` に修正（Favnir 構文）
- E0325 → E0422（v56.1.0 でエラーコード統一）に追随

---

Status: COMPLETE（2026-07-30）— 3318 tests passed, 0 failed
