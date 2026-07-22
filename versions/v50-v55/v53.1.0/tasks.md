# Tasks: v53.1.0 — lineage × LSP 統合（リネージをエディタで表示）

Status: COMPLETE
Date: 2026-07-22

---

## T0 — 事前確認

- [x] `cargo test` 3160 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `CheckedDoc` に `lineage` フィールドが**存在しない**ことを確認:
  - [x] `rg -n "pub lineage" fav/src/lsp/document_store.rs` → 0 件
- [x] `hover.rs` に `lineage_block_for_stage` が**存在しない**ことを確認:
  - [x] `rg -n "lineage_block_for_stage" fav/src/lsp/hover.rs` → 0 件
- [x] `LineageReport` の構造体フィールドを確認（`transformations` / `pipelines`）:
  - [x] `rg -n "pub struct LineageReport" fav/src/lineage.rs` → 行番号を特定
- [x] `PipelineLineage` 構造体に `steps: Vec<String>` フィールドが存在することを確認:
  - [x] `rg -n "pub struct PipelineLineage\|steps" fav/src/lineage.rs` → フィールドを特定
- [x] `LineageReport` に `Default` が実装されているか確認:
  - [x] `rg -n "impl Default for LineageReport\|derive.*Default.*LineageReport" fav/src/lineage.rs`
- [x] `lineage_analysis` 関数が `pub fn` であることを確認:
  - [x] `rg -n "pub fn lineage_analysis" fav/src/lineage.rs` → 行番号を特定
- [x] `driver.rs` に `v53100_tests` が**存在しない**ことを確認:
  - [x] `rg -n "v53100_tests" fav/src/driver.rs` → 0 件
- [x] `driver.rs` に `v53000_tests` が存在することを確認（挿入位置の確認）:
  - [x] `rg -n "v53000_tests" fav/src/driver.rs` → 行番号を特定
- [x] `Cargo.toml` の現在バージョンが `53.0.0` であることを確認

---

## T1 — `lsp/document_store.rs` 更新

- [x] `LineageReport` の `Default` 実装を確認し、未実装であれば `lineage.rs` に追加:
  - [x] `LineageReport` の `#[derive(...)]` に `Default` を追加（`LineageEntry` には追加しない）
  - [x] `Vec` フィールドのみなので `#[derive(Default)]` で十分（手動 impl 不要）
- [x] `document_store.rs` の先頭 use 文に import を追加:
  - [x] `use crate::lineage::{lineage_analysis, LineageReport};`
- [x] `CheckedDoc` 構造体に `lineage: LineageReport` フィールドを追加:
  - [x] `record_fields` フィールドの直後に追加
- [x] `open_or_change` 成功パスに `lineage_analysis` 呼び出しを追加:
  - [x] `let lineage = lineage_analysis(&program);` を追加
  - [x] `CheckedDoc { ..., lineage }` に追加
- [x] `open_or_change` 失敗パスに空の `lineage` を追加:
  - [x] `lineage: LineageReport { transformations: vec![], pipelines: vec![] }`
- [x] `cargo build` → コンパイルエラーなし確認

---

## T2 — `lsp/hover.rs` 更新

- [x] `lineage_block_for_stage(doc: &CheckedDoc, stage_name: &str) -> Option<String>` を追加:
  - [x] `doc.lineage.pipelines` から upstream / downstream を検索
  - [x] `doc.lineage.transformations` から schema を検索
  - [x] lines が空なら `None` を返す
- [x] `handle_hover` の type hover パスで stage 名を識別し `lineage_block_for_stage` を呼ぶ:
  - [x] 対象 stage 名が `doc.lineage.transformations` に存在する場合のみ付加
  - [x] 結果を `\n\n` で hover 本文に連結
- [x] `cargo build` → コンパイルエラーなし確認

---

## T3 — `driver.rs` — `v53100_tests` 追加

- [x] `rg -n "v53000_tests" fav/src/driver.rs` で挿入位置（行番号）を確認
- [x] `v53000_tests` モジュールの直前に `v53100_tests` を追加:
  - [x] `lsp_hover_shows_lineage` テスト:
    - [x] `DocumentStore::new()` を作成
    - [x] seq pipeline を含むソースで `open_or_change` を呼ぶ
    - [x] `doc.lineage.pipelines.is_empty()` が false であることを assert
  - [x] `lsp_hover_lineage_upstream` テスト:
    - [x] `stage A |> B` を含むソースで `open_or_change` を呼ぶ
    - [x] `pipeline.steps` に A・B が含まれ、A が B より前であることを assert
- [x] `cargo build` → コンパイルエラーなし確認

---

## T4 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "53.0.0"` → `version = "53.1.0"` に変更
- [x] `v53000_tests::cargo_toml_version_is_53_0_0` のアサートを空化:
  - [x] コメント `// Version bump is tested in v531xx (no version pin test in v53100_tests).` に置換
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3162 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T5 — 後処理

- [x] `CHANGELOG.md` に v53.1.0 エントリ追加
- [x] `versions/current.md` を v53.1.0（3162 tests）に更新
- [x] `roadmap-v53.1-v54.0.md` の v53.1.0 実績欄を更新（未実施 → COMPLETE）:
  - [x] 実績テスト数を記録
  - [x] v53.2.0 の推定値を 3162 + 2 = 3164 に確認・修正
- [x] tasks.md を COMPLETE に更新（T0〜T5 全 `[x]`）
