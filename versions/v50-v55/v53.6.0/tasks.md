# Tasks: v53.6.0 — cookbook 更新（parallel-pipeline + schema-validation）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3173 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v53600_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53600_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53500_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53500_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `site/content/cookbook/schema-validation.mdx` が**存在しない**ことを確認:
  - [x] `ls site/content/cookbook/schema-validation.mdx 2>/dev/null` → NOT FOUND
- [x] `site/content/cookbook/parallel-pipeline.mdx` が存在し `par` / `Merge` を含むことを確認
- [x] `Cargo.toml` の現在バージョンが `53.5.0` であることを確認

---

## T1 — `site/content/cookbook/schema-validation.mdx` 新規作成

- [x] `schema-validation.mdx` を `site/content/cookbook/` に作成:
  - [x] フロントマター（`title` / `description`）を含む
  - [x] `assert_schema<T>` の使用例コードブロックを含む
  - [x] nullable フィールド（`String?`）の説明を含む
  - [x] `fav run --audit-log ./audit.log` のコマンド例を含む
  - [x] OTel span への自動付与の説明を含む
  - [x] `fav explain --error E0419` のコマンド例を含む
- [x] 内容確認:
  - [x] `grep "assert_schema" site/content/cookbook/schema-validation.mdx` → 1 件以上
  - [x] `grep "\-\-audit-log" site/content/cookbook/schema-validation.mdx` → 1 件以上

---

## T2 — `driver.rs` — `v53600_tests` 追加

- [x] `rg -n "v53500_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53500_tests` モジュールの直前に `v53600_tests` を追加:
  - [x] `cookbook_parallel_pipeline_exists` テスト:
    - [x] `include_str!("../../site/content/cookbook/parallel-pipeline.mdx")` で内容を読み込む
    - [x] `"par ["` または `"par [A"` を含むことを assert
    - [x] `"Merge"` または `"merge"` を含むことを assert
  - [x] `cookbook_schema_validation_exists` テスト:
    - [x] `include_str!("../../site/content/cookbook/schema-validation.mdx")` で内容を読み込む
    - [x] `"assert_schema"` を含むことを assert
    - [x] `"--audit-log"` を含むことを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.5.0"` → `version = "53.6.0"` に変更
- [x] v53500_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3175 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T4 — 後処理

- [x] `CHANGELOG.md` に v53.6.0 エントリ追加（直前の v53.5.0 エントリと同形式であることを確認）
- [x] `versions/current.md` を v53.6.0（3175 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.6.0 実績欄を更新（未実施 → COMPLETE、テスト数 3175）
  - [x] 推定値 3169 → 実績 3175 の差異を注記
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
