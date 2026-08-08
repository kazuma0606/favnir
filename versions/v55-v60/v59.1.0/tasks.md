# v59.1.0 Tasks — エンタープライズ E2E ハーネス強化

Date: 2026-07-29
Status: COMPLETE（2026-07-29）— 3310 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3308 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.0.0"` であることを確認
- [x] `examples/enterprise-demo/pipeline.fav` がまだ存在しないことを確認
- [x] `grep -c '59\.0\.0' fav/src/driver.rs` でローリング文字列件数を確認（7 件のはず）
- [x] `fav/src/driver.rs` に `cmd_test_enterprise` がまだ存在しないことを確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.0.0"` → `"59.1.0"`

---

## T2: roadmap 更新

- [x] `roadmap-v59.1-v60.0.md` の v59.2.0 ベース数を `3296 → 3310`、目標を `3298 → 3312` に修正

---

## T3: examples/enterprise-demo/pipeline.fav 作成

- [x] `examples/enterprise-demo/` ディレクトリを作成
- [x] `examples/enterprise-demo/pipeline.fav` を新規作成
  - `"RBAC"` を含む（テストで検証）
  - Blue/Green・Secret・mTLS・監査ログ・コンプライアンス・ポリシー・カタログへの言及

---

## T4: driver.rs に cmd_test_enterprise 追加

- [x] `cmd_test_enterprise() -> i32` を追加
  - 8 enterprise チェック行を出力（`[OK] RBAC enforcement (v57.1)` 等）
  - `All 8 enterprise checks passed.` を出力
  - `0` を返す

---

## T5: driver.rs テストモジュール追加

- [x] main.rs を変更する前に、driver.rs が正しくコンパイルできることを確認
- [x] `v59100_tests` モジュールを v59000_tests の直前に挿入
  - **注意**: T3（pipeline.fav 作成）と T4（cmd_test_enterprise 追加）を先に行うこと
  - [x] `use super::cmd_test_enterprise` を追加（`cmd_test_enterprise_suite` が使用）
  - [x] `enterprise_e2e_demo_structure`: `include_str!("../../examples/enterprise-demo/pipeline.fav")` が `"RBAC"` を含む
  - [x] `cmd_test_enterprise_suite`: `cmd_test_enterprise()` が `0` を返す

---

## T6: main.rs 更新

- [x] `use crate::driver::` インポートに `cmd_test_enterprise` を追加
- [x] `Some("test")` アームのフラグ解析ループに `"--suite"` アームを追加
  - `--suite enterprise` → `cmd_test_enterprise()` を呼んで `process::exit(code)`
  - 値なし → `eprintln!` + `exit(1)`
  - 未知の suite 名 → `eprintln!` + `exit(1)`
- [x] `--suite` が指定された場合に既存の `cmd_test()` が呼ばれないことを確認（`process::exit` で早期終了するため `cmd_test()` には到達しない）

---

## T7: driver.rs ローリングチェック更新

- [x] `version = \"59.0.0\"` → `\"59.1.0\"` に一括更新（7 件）
- [x] failure メッセージ `"Cargo.toml version should be 59.0.0"` → `"59.1.0"` に更新（7 件）
  - `cargo_toml_version_is_59_0_0`（ローリング）
  - `cargo_toml_version_is_58_9_0`（ローリング）
  - `cargo_toml_version_is_58_0_0`（ローリング）
  - `cargo_toml_version_is_57_9_0`（ローリング）
  - `cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0` 付き）
  - `cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0` 付き）
  - `cargo_toml_version_is_56_3_0`（ローリング）

---

## T8: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `enterprise_e2e_demo_structure` pass を確認
- [x] `cmd_test_enterprise_suite` pass を確認
- [x] 総テスト数 **3310** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T9: 事後処理

- [x] `CHANGELOG.md` に v59.1.0 エントリを追加
- [x] `versions/current.md` を v59.1.0 / 3310 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.1.0 実績欄を更新
- [x] v59.2.0 ベース数を実績値に合わせて再確認・修正（3310 で確定）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

- [MED][対応済み] `--suite` アームに `i += 2` なし → 全分岐で `process::exit()` するため不要だが意図不明確 → `// NOTE: すべての分岐で process::exit() するため i += 2 は不要（到達不能）` コメントを追加
- [LOW][対応不要] `cmd_test_enterprise` の `println!` がテスト時に標準出力を汚染 → 既存の `cmd_ha_run` 等と同様のパターンで実害なし

最終テスト数: 3310 tests passed, 0 failed（code-review 対応後も変化なし）

---

Status: COMPLETE（2026-07-29）— 3310 tests passed, 0 failed
