# v37.8.0 実装計画 — Multi-Source cookbook 5 本

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/cookbook/join-two-tables.mdx` | 新規作成 | `List.join_on` を使った 2 テーブル結合レシピ |
| `site/content/cookbook/cdc-postgres-to-warehouse.mdx` | 新規作成 | CDC Rune を使った Postgres → ウェアハウス CDC レシピ |
| `site/content/cookbook/fan-out-by-region.mdx` | 新規作成 | `List.fan_out` / `List.fan_in` 地域別分散処理レシピ |
| `site/content/cookbook/generic-etl-function.mdx` | 新規作成 | 境界付きジェネリクス / 行多相ジェネリック ETL レシピ |
| `site/content/cookbook/lineage-visualization.mdx` | 新規作成 | `fav explain --lineage --format dot/svg` リネージ可視化レシピ |
| `fav/src/driver.rs` | 変更 | `v37700_tests::cargo_toml_version_is_37_7_0` スタブ化 / `v37800_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.7.0"` → `"37.8.0"` |
| `CHANGELOG.md` | 追記 | `[v37.8.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.8.0 を完了済みにマーク（✅）・テスト件数を 1 件 → 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v37.8.0、次バージョン v37.9.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.8.0] エントリ追加

`## [v37.7.0]` の `---` セパレータ直後に挿入:

```markdown
## [v37.8.0] - 2026-07-09

### Added
- `site/content/cookbook/join-two-tables.mdx` — `List.join_on` 2 テーブル結合レシピ
- `site/content/cookbook/cdc-postgres-to-warehouse.mdx` — CDC Rune レシピ
- `site/content/cookbook/fan-out-by-region.mdx` — `List.fan_out` / `List.fan_in` レシピ
- `site/content/cookbook/generic-etl-function.mdx` — ジェネリック ETL レシピ
- `site/content/cookbook/lineage-visualization.mdx` — リネージグラフ可視化レシピ
- `v37800_tests` 3 テスト追加

---
```

### Step 2: 5 つの cookbook MDX ファイルを作成

spec.md §1〜§5 のコードブロックに従い各ファイルを作成。

**作成順序（依存なし、任意）:**
1. `join-two-tables.mdx` — `List.join_on` キーワード必須
2. `cdc-postgres-to-warehouse.mdx` — `CDC.filter_inserts` キーワード必須
3. `fan-out-by-region.mdx` — `List.fan_out` キーワード必須
4. `generic-etl-function.mdx` — `Serialize` キーワード必須
5. `lineage-visualization.mdx` — `--format dot` キーワード必須

**各ファイルの共通構造:**
- YAML frontmatter（`title` / `description`）
- H1 タイトル
- 説明文
- コード例（` ```favnir ` または ` ```bash `）
- ポイント箇条書き
- 関連リンク

### Step 3: `driver.rs` — `v37700_tests::cargo_toml_version_is_37_7_0` をスタブ化

ライブアサーション → `// Stubbed: version bumped to 37.8.0 — assertion intentionally removed` に変更。

**注意:** `changelog_has_v37_7_0` / `fav_new_multi_source_ok` はスタブ化しない。

### Step 4: `driver.rs` — `v37800_tests` モジュール追加

`v37700_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §6 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_37_8_0` / `changelog_has_v37_8_0` / `multi_source_cookbook_files_exist`

`multi_source_cookbook_files_exist` テスト内の `include_str!` パスは各ファイルの存在を保証するため、
**Step 2 が完了してから** Step 4 を実行する（コンパイルが通る順序）。

### Step 5: Cargo.toml バージョン更新

Step 1〜4 完了後に `37.7.0` → `37.8.0` に更新。

### Step 6: ドキュメント更新

- `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.8.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v37.8.0（最新安定版）・v37.9.0（次バージョン）に更新

## 依存関係

- MDX ファイルは単純テキスト — Rust コードへの依存なし
- `multi_source_cookbook_files_exist` の `include_str!` は**コンパイル時**にファイル存在を要求するため、
  Step 2（MDX 作成）が Step 4（テスト追加）より先に完了している必要がある
- `v37700_tests::cargo_toml_version_is_37_7_0` のスタブ化は Step 3 で実施（バージョン更新前）

## リスク

| リスク | 対処 |
|---|---|
| MDX 内のバッククォートトリプルがネストして構文が壊れる | Write ツールで直接作成（heredoc 不使用）|
| `include_str!` のパスが相対パスで正しいか | `driver.rs` は `fav/src/driver.rs`、`site/` は `fav/` と兄弟ディレクトリ → `../../site/content/cookbook/` が正しい（他テストで確認済み: 行 37323）|
| Step 4 より先に Cargo.toml を更新すると `cargo_toml_version_is_37_7_0` のスタブ化テストが失敗する | Step 5 を必ず Step 3 より後に実行 |
