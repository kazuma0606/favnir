# v37.6.0 実装計画 — `fav lineage --graph`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/lineage.rs` | 変更 | `render_lineage_dot` / `render_lineage_svg` 追加 |
| `fav/src/driver.rs` | 変更 | `pub use` 更新 / `cmd_explain_lineage` に `dot`/`svg` 追加 / help テキスト更新 / `v37500_tests::cargo_toml_version_is_37_5_0` スタブ化 / `v37600_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.5.0"` → `"37.6.0"` |
| `CHANGELOG.md` | 追記 | `[v37.6.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.6.0 を完了済みにマーク（✅）・テスト件数を 2 件 → 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v37.6.0、次バージョン v37.7.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.6.0] エントリ追加

`## [v37.5.0]` の `---` セパレータ直後に挿入:

```markdown
## [v37.6.0] - 2026-07-09

### Added
- `fav explain --lineage --format dot` — Graphviz DOT 形式のリネージグラフ出力
- `fav explain --lineage --format svg` — インライン SVG 形式のリネージグラフ出力
- `render_lineage_dot` / `render_lineage_svg` を `lineage.rs` に追加
- `v37600_tests` 4 テスト追加

---
```

### Step 2: `lineage.rs` に `render_lineage_dot` / `render_lineage_svg` 追加

`render_lineage_d2` の閉じ `}` の直後（`sanitize_mermaid_id` の前）に挿入する。

**手順:**
1. `lineage.rs` の `render_lineage_d2` 終端行と `sanitize_mermaid_id` 開始行を Read で特定
2. その間に 2 つの関数を Edit で挿入

追加コードは spec.md §1・§2 に従う:
- `render_lineage_dot`: `sanitize_mermaid_id` を流用（同ファイル内なので直接呼び出し可）
- `render_lineage_svg`: `std::collections::HashMap` を使用（標準ライブラリ、追加 import 不要）

### Step 3: `driver.rs` — `pub use` 更新

`render_lineage_d2,` の後に `, render_lineage_dot, render_lineage_svg` を追加。

```rust
pub use crate::lineage::{
    extract_tables_from_sql, lineage_analysis,
    render_lineage_json, render_lineage_text,
    render_lineage_mermaid, render_lineage_d2,
    render_lineage_dot, render_lineage_svg,
};
```

### Step 4: `driver.rs` — `cmd_explain_lineage` に `dot` / `svg` 追加

`"d2"` アームの直後に 2 アームを挿入。エラーメッセージも更新。

### Step 5: `driver.rs` — help テキスト更新

`--format <text|json|mermaid|d2>` を `--format <text|json|mermaid|d2|dot|svg>` に変更。

### Step 6: driver.rs — `v37500_tests::cargo_toml_version_is_37_5_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.6.0 — assertion intentionally removed` に変更。

**注意:** `changelog_has_v37_5_0` / `cdc_rune_file_exists` / `cdc_rune_toml_exists` はスタブ化しない。

### Step 7: driver.rs — `v37600_tests` モジュール追加

`v37500_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。

spec.md §6 のコードブロックに従う:
- `use super::{render_lineage_dot, render_lineage_svg}`
- `use crate::lineage::{LineageReport, LineageEntry, PipelineLineage}`
- `make_report()` ヘルパー関数（テスト用レポート生成）
- 4 テスト: meta 2 件 + 機能 2 件

### Step 8: Cargo.toml バージョン更新

Step 1〜7 完了後に `37.5.0` → `37.6.0` に更新。

## 依存関係

- `render_lineage_dot` / `render_lineage_svg` は既存 struct `LineageReport` / `LineageEntry` / `PipelineLineage` を使用
- `sanitize_mermaid_id` は同ファイル内プライベート関数 — `render_lineage_dot` 内から直接呼び出し可能
- `std::collections::HashMap` は標準ライブラリ — 追加 Cargo.toml 依存なし
- petgraph の `Dot` 構造体は使わない（手書き DOT 生成で十分）

## リスク

| リスク | 対処 |
|---|---|
| `sanitize_mermaid_id` が `render_lineage_dot` から呼べない（pub でない） | 同ファイル（`lineage.rs`）内の関数なので `pub` でなくてもアクセス可能 |
| `std::collections::HashMap` の import が必要か | Rust では `std::collections::HashMap` はフルパス記述で import 不要 |
| `v37600_tests` の `use super::` で `render_lineage_dot/svg` が見つからない | Step 3 で `pub use` に追加済みならアクセス可能 |
| `lineage.rs` への挿入で `sanitize_mermaid_id` の前後を誤って編集 | Read で行番号を確認してから Edit を実行 |
| SVG の `name_to_idx` のライフタイムコンパイルエラー | `nodes` は `Vec<&LineageEntry>` で `report` 参照を保持 — 同スコープ内なので問題なし |
