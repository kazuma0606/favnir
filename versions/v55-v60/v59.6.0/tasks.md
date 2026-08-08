# v59.6.0 Tasks — Enterprise 認定チェックリスト（`fav certify`）

Date: 2026-07-30
Status: COMPLETE（2026-07-30）— 3320 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3318 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.5.0"` であることを確認
- [x] `fav/src/driver.rs` に `cmd_certify` がまだ存在しないことを確認
- [x] `fav/src/driver.rs` に `generate_enterprise_cert` がまだ存在しないことを確認
- [x] `fav/src/driver.rs` に `v59600_tests` がまだ存在しないことを確認
- [x] `grep -o '59\.5\.0' fav/src/driver.rs | wc -l` でローリング文字列件数を確認（14 件: assertion 7 件 + failure メッセージ 7 件）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.5.0"` → `"59.6.0"`

---

## T2: driver.rs に cmd_certify / generate_enterprise_cert 追加

- [x] `cmd_certify() -> String` を `migrate_enterprise_import` の直後に追加
  - Enterprise 1.0 の 6 項目チェック結果を文字列で返す
  - `[OK]` 5 件 + `[WARN]` 1 件（SLA enforcement）
  - 末尾: `"Enterprise 1.0 certification: 5/6 checks passed (1 warning)\n"`
- [x] `generate_enterprise_cert() -> String` を `cmd_certify` の直後に追加
  - JSON 証明書文字列を返す
  - フィールド: `version`, `issued_at`, `checks_passed`, `checks_total`, `warnings`, `certification`

---

## T4: driver.rs テストモジュール追加

- [x] T2（関数追加）を先に実施済み
- [x] `v59600_tests` モジュールを `v59500_tests` の直前に挿入
  - [x] `cmd_certify_passes` テスト: `super::cmd_certify()` が `[OK]` / `RBAC` / `5/6 checks passed` を含む
  - [x] `cmd_certify_generates_cert` テスト: `super::generate_enterprise_cert()` が `enterprise-1.0` / `checks_passed` / `certification` を含む

---

## T5: driver.rs ローリングチェック更新

- [x] `version = \"59.5.0\"` → `\"59.6.0\"` に一括更新（7 件）
- [x] failure メッセージ 7 件を `"59.6.0"` に更新（詳細は plan.md Step 4 参照）
  - 通常 5 件: `"Cargo.toml version should be 59.5.0, got: {}"` → `"59.6.0, got: {}"`
    - `cargo_toml_version_is_59_0_0` / `v58900` / `v58000` / `v57900` / `v56300`
  - **特殊書式** 2 件（`rolling check from` サフィックスあり）:
    - `cargo_toml_version_is_57_0_0`: `"Cargo.toml version should be 59.5.0 (rolling check from v57.0.0), got: {}"` → `"59.6.0 (rolling check from v57.0.0), got: {}"`
    - `cargo_toml_version_is_56_9_0`: `"Cargo.toml version should be 59.5.0 (rolling check from v56.9.0), got: {}"` → `"59.6.0 (rolling check from v56.9.0), got: {}"`
  - **注意**: `v59000_tests` は rolling check あり（対象）。`v59100_tests`〜`v59500_tests` は rolling check なし（対象外）

---

## T6: main.rs — Some("certify") アーム追加

- [x] インポート行に `cmd_certify, generate_enterprise_cert` を追加
- [x] `Some("migrate")` アームの直後（`Some("upgrade")` の前）に `Some("certify")` アームを追加
  - `--level enterprise` で `cmd_certify()` を出力し `enterprise-cert.json` を書き出す
  - それ以外の `--level` は `eprintln!` + `process::exit(1)`

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v59600_tests::cmd_certify_passes` pass を確認
- [x] `v59600_tests::cmd_certify_generates_cert` pass を確認
- [x] 総テスト数 **3320** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v59.6.0 エントリを追加
- [x] `versions/current.md` を v59.6.0 / 3320 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.6.0 実績欄を更新
- [x] `roadmap-v59.1-v60.0.md` の v59.7.0 ベース数を「着手時に更新」→ `3320` に確定（T7 でテスト数確認後に実施）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

Status: COMPLETE（2026-07-30）— 3320 tests passed, 0 failed
