# v37.9.0 実装計画 — v38.0 前調整・安定化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/lineage.rs` | 変更 | `render_lineage_text` にサマリー行追加 |
| `site/content/docs/multi-source-etl.mdx` | 新規作成 | v37.x 系 Multi-Source ETL 機能一覧ドキュメント |
| `fav/src/driver.rs` | 変更 | `v37800_tests::cargo_toml_version_is_37_8_0` スタブ化 / `v37900_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.8.0"` → `"37.9.0"` |
| `CHANGELOG.md` | 追記 | `[v37.9.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.9.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v37.9.0、次バージョン v38.0.0 |
| `versions/v36-v40/v37.9.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T8 全チェック） |

## 実装順序

### Step 1: CHANGELOG.md に [v37.9.0] エントリ追加

`## [v37.8.0]` の `---` セパレータ直後に挿入:

```markdown
## [v37.9.0] — 2026-07-09

### Added
- `render_lineage_text` にサマリー行追加（`Total: N stage(s), M pipeline(s)`）
- `site/content/docs/multi-source-etl.mdx` — Multi-Source ETL 機能一覧ドキュメント
- `v37900_tests` 4 テスト追加

---
```

### Step 2: `lineage.rs` — `render_lineage_text` にサマリー行追加

`render_lineage_text` 末尾の `out` 返却直前（`Pipelines:` ブロックの後）に追加:

```rust
    // v37.9.0: サマリー行
    out.push('\n');
    out.push_str(&format!(
        "Total: {} stage(s), {} pipeline(s)\n",
        report.transformations.len(),
        report.pipelines.len(),
    ));

    out
```

**変更前の末尾（確認用):**
```rust
        }
    }

    out
}
```

### Step 3: `site/content/docs/multi-source-etl.mdx` 新規作成

spec.md §2 に従い作成。必須要素:
- YAML frontmatter（`title` / `description`）
- `List.join_on` を含むコード例（`multi_source_etl_doc_exists` テストの必須キーワード）

### Step 4: `driver.rs` — `v37800_tests::cargo_toml_version_is_37_8_0` をスタブ化

```rust
// Stubbed: version bumped to 37.9.0 — assertion intentionally removed
```

### Step 5: `driver.rs` — `v37900_tests` モジュール追加

`v37800_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 4 テスト: `cargo_toml_version_is_37_9_0` / `changelog_has_v37_9_0` / `lineage_text_has_summary_line` / `multi_source_etl_doc_exists`

`multi_source_etl_doc_exists` の `include_str!` は Step 3（MDX 作成）完了後に追加すること（コンパイルが通る順序）。

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `37.8.0` → `37.9.0` に更新。

### Step 7: ドキュメント更新

- `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.9.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v37.9.0（最新安定版）・v38.0.0（次バージョン）に更新

## 依存関係

- `lineage.rs` 変更は Rust コードのみ — MDX ファイルへの依存なし
- `multi_source_etl_doc_exists` の `include_str!` は**コンパイル時**にファイル存在を要求するため、Step 3（MDX 作成）が Step 5（テスト追加）より先に完了している必要がある
- `v37800_tests::cargo_toml_version_is_37_8_0` のスタブ化は Step 4 で実施（バージョン更新前）

## リスク

| リスク | 対処 |
|---|---|
| `render_lineage_text` の変更が他の lineage テストに影響する | v37600_tests の `lineage_dot_contains_digraph` / `lineage_svg_contains_svg_tag` は DOT/SVG レンダラーをテストしており `render_lineage_text` とは独立。影響なし。 |
| `include_str!("lineage.rs")` でパスエラー | `driver.rs` は `fav/src/driver.rs`、`lineage.rs` も `fav/src/lineage.rs` → 同一ディレクトリ → `include_str!("lineage.rs")` で正しい |
| MDX 内のバッククォートトリプルネスト | Write ツールで直接作成（spec.md からコピーしない） |
