# v21.2.0 — fav explain 可視化強化 タスク

## ステータス: DONE

---

## タスク一覧

### T1: `fav/src/lineage.rs` — Mermaid / D2 レンダラー追加

- [x] **事前確認**: `grep -n "^pub fn render_lineage" fav/src/lineage.rs` で既存レンダラーを確認
- [x] `render_lineage_mermaid(report: &LineageReport) -> String` を追加（plan.md T1-1 に従う）
  - [x] `flowchart LR` ヘッダー出力
  - [x] stage ノード定義（`id["name\neffects"]` 形式）
  - [x] `sanitize_mermaid_id` ヘルパー（英数字 + `_` のみ）
  - [x] pipeline steps → `-->` エッジ出力
- [x] `render_lineage_d2(report: &LineageReport) -> String` を追加（plan.md T1-2 に従う）
  - [x] ノード定義（`name: "name (effects)"` 形式）
  - [x] pipeline steps → `->` エッジ出力
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/driver.rs` — `cmd_explain_lineage` 更新

- [x] **事前確認**: `grep -n "render_lineage" fav/src/driver.rs | head -5` で既存 pub use を確認
- [x] `pub use crate::lineage::` の既存ブロックを `render_lineage_mermaid` / `render_lineage_d2` 追加版に **丸ごと置き換え**（追記すると重複エラー）
- [x] 既存の `lineage_tests`（`cargo test lineage_tests`）が引き続きパスすること
- [x] `cmd_explain_lineage` の format 分岐を `match` に変更し `"mermaid"` / `"d2"` アームを追加
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/main.rs` — CLI 更新

- [x] **事前確認**: `grep -n "lineage\|--format" fav/src/main.rs | head -10` で既存パースを確認
- [x] `fav explain --lineage` のヘルプ文言に `mermaid` / `d2` を追記（`--format text|json|mermaid|d2`）
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "21.1.0"` → `"21.2.0"` に変更
- [x] **事前確認**: `grep -n "mod v211000_tests\|version_is_21_1_0" fav/src/driver.rs` で行番号を確認
- [x] `v211000_tests` の `version_is_21_1_0` に `#[ignore]` を追加
  - `fav/src/driver.rs` の `v211000_tests` モジュール内 `version_is_21_1_0` 関数に追加
  - 既存 `#[test]` の直下に `#[ignore]` を挿入
- [x] `cargo test v211000` — `version_is_21_1_0` が skip されること（0件 passではなく `0 passed; 0 failed; 1 ignored`）
- [x] `cargo build` でコンパイルエラー 0

---

### T5: `CHANGELOG.md` + `site/content/docs/tools/lineage.mdx`

- [x] `CHANGELOG.md` の先頭に v21.2.0 エントリを追加（plan.md T5 の内容に従う）
  - [x] `### Added` — `--format mermaid` / `--format d2` / 新レンダラー関数 / MDX
- [x] **事前確認**: `ls site/content/docs/tools/` で `lineage.mdx` が既存かどうかを確認
- [x] `site/content/docs/tools/lineage.mdx` を新規作成（既存の場合は上書き更新）
  - [x] 4形式（text / json / mermaid / d2）の使い方
  - [x] Mermaid 出力例
  - [x] D2 出力例
  - [x] GitHub への貼り付け方法（コードブロック）

---

### T6: `fav/src/driver.rs` — `v212000_tests` 追加

- [x] `v211000_tests::version_is_21_1_0` に `#[ignore]` が付いていること（T4 で実施済み）
- [x] `v212000_tests` モジュールを追加（plan.md T6 の内容に従う）
  - [x] `version_is_21_2_0` — Cargo.toml に `"21.2.0"` が含まれる
  - [x] `render_lineage_mermaid_basic` — ノード定義が生成される
  - [x] `render_lineage_mermaid_pipeline_edges` — `-->` エッジが生成される
  - [x] `render_lineage_d2_basic` — ノード定義と `->` エッジが生成される
  - [x] `lineage_format_mermaid_no_panic` — 実際のソースで mermaid 出力がパニックしない
- [x] 各テストに `#[cfg(not(target_arch = "wasm32"))]` ガードを付与（モジュールレベル）
- [x] `cargo test v212000` — 5/5 PASS を確認

---

## テスト（v212000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_2_0` | Cargo.toml に `"21.2.0"` が含まれる |
| `render_lineage_mermaid_basic` | ノード定義に `flowchart LR` とステージ名が含まれる |
| `render_lineage_mermaid_pipeline_edges` | `-->` エッジが pipeline の steps から生成される |
| `render_lineage_d2_basic` | D2 形式にノード定義と `->` エッジが含まれる |
| `lineage_format_mermaid_no_panic` | 実際のソースで `render_lineage_mermaid` がパニックしない |

---

## 完了条件チェックリスト

- [x] `fav explain --lineage --format mermaid <file>` が Mermaid テキストを stdout に出力する
- [x] `fav explain --lineage --format d2 <file>` が D2 テキストを stdout に出力する
- [x] `fav explain --lineage --format text` / `json` の既存動作がリグレッションしない（`cargo test lineage_tests` PASS）
- [x] Mermaid 出力に `flowchart LR` ヘッダー、ノード定義、エッジが含まれる
- [x] D2 出力にノード定義と `->` エッジが含まれる
- [x] `site/content/docs/tools/lineage.mdx` が存在する
- [x] `cargo test v212000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（exit 0）
- [x] `CHANGELOG.md` に v21.2.0 エントリが追加されている
- [x] `fav/Cargo.toml` version が `21.2.0`

---

## 優先度

```
T1（lineage.rs — render_lineage_mermaid / render_lineage_d2）  ← 最初
T2（driver.rs — cmd_explain_lineage 更新）                     ← T1 完了後
T3（main.rs — CLI 更新）                                       ← T2 完了後
T4（Cargo.toml バージョン）                                    ← T1 と並列可
T5（CHANGELOG + MDX）                                          ← T3 完了後
T6（driver.rs テスト）                                         ← T1 完了後
```

Rust コードへの変更は T1（lineage.rs）・T2（driver.rs）・T6（テスト）のみ。
T3・T5 はドキュメント / 文字列のみ。

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| ノード名に Mermaid 予約語 / 特殊文字が含まれる | `sanitize_mermaid_id` で英数字 + `_` のみに変換 |
| stage 名に日本語が含まれる | `sanitize_mermaid_id` は全角を `_` に変換。ラベル（`["..."]`）には元の名前を使う |
| `par [A, B]` の分岐エッジが `PipelineLineage.steps` に含まれない | v21.2.0 では `par` 分岐エッジは対象外（将来版持ち越し） |
| D2 のノード名にスペースが含まれる | ノード ID は `sanitize_mermaid_id` で変換（スペース → `_`）。D2 は識別子にスペース不可 |
| `pub use` 追記で重複エラー | 既存 `pub use crate::lineage::{...}` ブロックを丸ごと置き換える |
