# Tasks: v53.7.0 — ドキュメントサイト全体最終チェック

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3175 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v53700_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53700_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53600_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53600_tests" fav/src/driver.rs` → 行番号を特定（47602）
- [x] `site/content/docs/glossary.mdx` が**存在しない**ことを確認:
  - [x] `ls site/content/docs/glossary.mdx 2>/dev/null` → エラー
- [x] 主要 docs ファイルが存在することを確認（`docs_no_broken_links` テストの前提）:
  - [x] `ls site/content/docs/introduction.mdx` → 存在
  - [x] `ls site/content/docs/quickstart.mdx` → 存在
  - [x] `ls site/content/docs/installation.mdx` → 存在
- [x] `Cargo.toml` の現在バージョンが `53.6.0` であることを確認

---

## T1 — `site/content/docs/glossary.mdx` 新規作成

- [x] `glossary.mdx` を `site/content/docs/` に作成:
  - [x] フロントマター（`title: "用語集"` / `description`）を含む
  - [x] `## par` セクション: `par [A, B] |> Merge.ordered` の説明を含む
  - [x] `## assert_schema` セクション: `assert_schema<T>(map)` と E0419 の説明を含む
  - [x] `## lineage` セクション: upstream/downstream・LSP ホバー統合の説明を含む
  - [x] `## inlay hints` セクション: LSP インレイヒントの説明を含む（`inlay` というキーワードを含む）
  - [x] `## rune` / `## stage` / `## pipeline` セクションを含む
- [x] 内容確認:
  - [x] `grep "## par" site/content/docs/glossary.mdx` → 1 件以上
  - [x] `grep "assert_schema" site/content/docs/glossary.mdx` → 1 件以上
  - [x] `grep "lineage" site/content/docs/glossary.mdx` → 1 件以上
  - [x] `grep "inlay" site/content/docs/glossary.mdx` → 1 件以上

---

## T2 — `driver.rs` — `v53700_tests` 追加

- [x] `rg -n "v53600_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53600_tests` モジュールの直前に `v53700_tests` を追加:
  - [x] `docs_no_broken_links` テスト:
    - [x] `env!("CARGO_MANIFEST_DIR").join("../site/content/docs")` でベースパスを構築
    - [x] `introduction.mdx` / `quickstart.mdx` / `glossary.mdx` / `installation.mdx` の存在を assert
  - [x] `docs_glossary_updated` テスト:
    - [x] `include_str!("../../site/content/docs/glossary.mdx")` で内容を読み込む
    - [x] `"## par"` / `"assert_schema"` / `"lineage"` / `"inlay"` を含むことを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.6.0"` → `version = "53.7.0"` に変更
- [x] v53600_tests にバージョンピンテストは存在しないため空化対象なし（確認済み）
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3177 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T4 — 後処理

- [x] `CHANGELOG.md` に v53.7.0 エントリ追加（直前の v53.6.0 エントリと同形式であることを確認）
- [x] `versions/current.md` を v53.7.0（3177 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.7.0 実績欄を更新（未実施 → COMPLETE、テスト数 3177）
  - [x] 推定値 3171 → 実績 3177 の差異を注記
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
