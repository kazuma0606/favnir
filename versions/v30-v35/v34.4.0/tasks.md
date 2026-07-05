# v34.4.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `34.3.0` であること
- [x] `benchmarks/v34.3.0.json` の `tests_passed` が 2551 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2551 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v344000_tests` が存在しないこと
- [x] v34.3.0 が COMPLETE であること
- [x] `cargo_toml_version_is_34_3_0` が v343000_tests 内に存在すること（スタブ化対象）
  ```bash
  grep -A3 "cargo_toml_version_is_34_3_0" fav/src/driver.rs | head -5
  # assert! が残っていること（スタブ化前）を確認
  ```
- [x] `cargo test --bin fav v343000` が 5/5 PASS であること（前バージョン 5 件 PASS を確認）
- [x] `site/content/docs/tools/security-audit-v2.mdx` が存在しないこと（新規作成対象）
  ```bash
  ls site/content/docs/tools/security-audit-v2.mdx 2>/dev/null || echo "does not exist"
  # does not exist であることを確認
  ```
- [x] `site/content/docs/tools/oss-licenses.mdx` が存在しないこと（新規作成対象）
  ```bash
  ls site/content/docs/tools/oss-licenses.mdx 2>/dev/null || echo "does not exist"
  # does not exist であることを確認
  ```
- [x] `SECURITY_MODEL.md` に `"v34"` が含まれていないこと（追記対象）
  ```bash
  grep "v34" SECURITY_MODEL.md | wc -l
  # 0 であることを確認
  ```
- [x] `SECURITY_MODEL.md` の末尾（最終 20 行）が ctx 移行セクション追記対象の末尾であること（T4 実施後の確認: `tail -20 SECURITY_MODEL.md` で v34 セクションが末尾に位置していること）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `34.3.0` → `34.4.0` に更新
- [x] **T2** `site/content/docs/tools/security-audit-v2.mdx` — セキュリティ審査 v2 レポートを新規作成（W021・認証情報・sandbox・OSS ライセンスの 4 項目）
- [x] **T3** `site/content/docs/tools/oss-licenses.mdx` — OSS 依存ライセンス一覧を新規作成（主要 26 クレート）
- [x] **T4** `SECURITY_MODEL.md` — v34.x ctx 移行との関係セクションを末尾に追記（`"v34"` を含む）
- [x] **T5** `fav/src/driver.rs` — `cargo_toml_version_is_34_3_0` をスタブ化
- [x] **T6** `fav/src/driver.rs` — `v344000_tests`（5 件）を追加
        挿入位置: `v343000_tests` 直後・`// ── v31.7.0 tests` の前
        `use super::*` なし、import なし（`include_str!` のみ使用）
- [x] **T7** `CHANGELOG.md` — `[v34.4.0]` セクションを先頭に追記
- [x] **T8** `benchmarks/v34.4.0.json` — 新規作成（`tests_passed`: 2556 実測値）
- [x] **T9** `versions/current.md` — 「最新安定版」欄を v34.4.0 に更新

---

## テスト確認

- [x] **T10** `cargo test --bin fav v344000 2>&1 | tail -8` — 5/5 PASS
- [x] **T11** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2556 passed、0 failures）

---

## 完了処理

- [x] **T12** `benchmarks/v34.4.0.json` の `tests_passed` を実測値で更新（2556 確定）
- [x] **T13** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `cargo clean` 不要（x.4.0 のため実施しない）
- [x] `Cargo.toml` version = `"34.4.0"`
- [x] `cargo_toml_version_is_34_3_0` が空スタブになっていること（他テストは残存）
- [x] `cargo test --bin fav v344000` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（2556 passed、0 failures）
- [x] `site/content/docs/tools/security-audit-v2.mdx` が存在し `"W021"` を含むこと
- [x] `site/content/docs/tools/security-audit-v2.mdx` が `"sandbox"` を含むこと
- [x] `site/content/docs/tools/security-audit-v2.mdx` が認証情報ガイドライン（`"環境変数"` 等）を含むこと【手動確認済み】
- [x] `site/content/docs/tools/oss-licenses.mdx` が存在し `"MIT"` を含むこと
- [x] `SECURITY_MODEL.md` に `"v34"` 言及があること
- [x] `CHANGELOG.md` に `[v34.4.0]` セクション
- [x] `benchmarks/v34.4.0.json` 存在かつ `tests_passed` が実測値（2556）
- [x] `versions/current.md` が v34.4.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v344000_tests` に `use super::*` が**ない**こと
- [x] `v344000_tests` に import 文が**ない**こと（`include_str!` のみ）
- [x] WASM ゲートがないこと（ファイル読み込みのみ）
- [x] `cargo_toml_version_is_34_3_0` が空スタブになっていること（コメント付き）
- [x] `security_audit_v2_page_exists` で `src.contains("W021")` を assert していること
- [x] `oss_licenses_page_exists` で `src.contains("MIT")` を assert していること
- [x] `security_model_has_v34_section` で `src.contains("v34")` を assert していること
- [x] `security_audit_v2_covers_sandbox` で `src.contains("sandbox") || src.contains("サンドボックス")` を assert していること
- [x] 挿入位置が `v343000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v34.4.0.json` の `milestone` が `"Production Ready"` であること
- [x] `versions/current.md` が v34.4.0 に更新されていること
- [x] `security-audit-v2.mdx` が 4 審査項目（W021・認証情報・sandbox・OSS ライセンス）をすべてカバーしていること
- [x] `oss-licenses.mdx` に GPL 依存がゼロであることの確認記述があること
- [x] `SECURITY_MODEL.md` の ctx 移行セクションが末尾に追加されていること
