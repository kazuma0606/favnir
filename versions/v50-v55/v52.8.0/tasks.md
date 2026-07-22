# Tasks: v52.8.0 — ドキュメントサイト Data Quality 記事

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3151 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `site/content/docs/data-quality/` が**存在しない**ことを確認:
  - [x] `ls site/content/docs/data-quality/` → エラー（新規ディレクトリ作成が必要）
- [x] `site/content/docs/tools/` が存在することを確認:
  - [x] `ls site/content/docs/tools/` → 既存ディレクトリ
- [x] `site/content/docs/tools/audit-log.mdx` が**存在しない**ことを確認:
  - [x] `ls site/content/docs/tools/audit-log.mdx` → エラー（新規作成対象）
- [x] `site/content/docs/tools/lineage-enhanced.mdx` が**存在しない**ことを確認:
  - [x] `ls site/content/docs/tools/lineage-enhanced.mdx` → エラー（新規作成対象）
  - [x] このファイルは `include_str!` テストなし（目視確認のみ）— 意図的省略であることを確認
- [x] `site/content/docs/governance/audit-log.mdx` が**既存・別物**であることを確認:
  - [x] `ls site/content/docs/governance/audit-log.mdx` → 存在する（Audit Rune 説明・別ファイル）
  - [x] 新規作成する `tools/audit-log.mdx` は `fav run --audit-log` フラグの説明であり概念が異なることを確認
- [x] `driver.rs` に `v52800_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v52800_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v52700_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v52700_tests" fav/src/driver.rs` → 行 47565 を特定
- [x] `include_str!` パスの整合性確認（`driver.rs` は `fav/src/driver.rs`）:
  - [x] `fav/src/` から `../../` → `favnir/` → `site/content/...` ✓
- [x] `Cargo.toml` の現在バージョンが `52.7.0` であることを確認

---

## T1 — MDX ファイル作成

- [x] `site/content/docs/data-quality/assert-schema.mdx` を新規作成:
  - [x] title に "assert_schema" を含む
  - [x] `assert_schema` キーワードを含む
  - [x] `nullable` または `optional` キーワードを含む（"nullable フィールド" セクション）
  - [x] `strict-schema` または `strict_schema` キーワードを含む（"--strict-schema フラグ" セクション）
  - [x] E0419 エラーコードに言及
  - [x] `bind validated <- assert_schema<OrderRow>(row)` 使用例を含む
- [x] `site/content/docs/tools/lineage-enhanced.mdx` を新規作成（テストなし・目視確認のみ）:
  - [x] `--with-schema` オプションの説明を含む（目視確認）
  - [x] `--format html` オプションの説明を含む（目視確認）
  - [x] `-o <file>` オプションの説明を含む（目視確認）
- [x] `site/content/docs/tools/audit-log.mdx` を新規作成:
  - [x] `audit-log` または `audit_log` キーワードを含む
  - [x] `JSONL` または `jsonl` キーワードを含む
  - [x] JSONL 出力例（ts/op/effect フィールド）を含む
  - [x] `fav audit`（Enterprise Governance）との違いに言及
  - [x] `site/content/docs/governance/audit-log.mdx`（別物）との違いを冒頭 Note に明記

---

## T2 — `driver.rs` — `v52800_tests` 追加

- [x] `rg -n "v52700_tests" fav/src/driver.rs` で挿入位置（行 47565）を確認
- [x] `v52700_tests` モジュールの直前に `v52800_tests` を追加:
  - [x] `docs_assert_schema_page_exists` テスト:
    - [x] `include_str!("../../site/content/docs/data-quality/assert-schema.mdx")` を使用
    - [x] `src.contains("assert_schema")` を assert
    - [x] `src.contains("nullable") || src.contains("optional")` を assert
    - [x] `src.contains("strict-schema") || src.contains("strict_schema")` を assert
  - [x] `docs_audit_log_page_exists` テスト:
    - [x] `include_str!("../../site/content/docs/tools/audit-log.mdx")` を使用
    - [x] `src.contains("audit-log") || src.contains("audit_log")` を assert
    - [x] `src.contains("jsonl") || src.contains("JSONL")` を assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `fav/Cargo.toml` バージョン更新

- [x] `version = "52.7.0"` → `version = "52.8.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3154 passed, 0 failed を確認（code-reviewer 指摘対応で +1）
- [x] `cargo clippy -- -D warnings` クリーンを確認（T2 cargo build 成功後）

---

## T4 — 後処理

- [x] `CHANGELOG.md` に v52.8.0 エントリ追加
- [x] `versions/current.md` を v52.8.0（3153 tests）に更新
- [x] `roadmap-v52.1-v53.0.md` の v52.8.0 実績欄を更新（未実施 → COMPLETE）:
  - [x] 実績テスト数を記録（3153）
  - [x] v52.9.0 の推定値をロードマップ現在値 3155 から 3156（v52.8.0 実績 3154 + 2）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
