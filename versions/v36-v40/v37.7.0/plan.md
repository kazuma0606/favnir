# v37.7.0 実装計画 — `fav new --template multi-source`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 変更 | `create_multi_source_etl_project` 追加 / `try_cmd_new` 更新 / `TEMPLATE_GALLERY` 更新 / `cmd_new_list` 更新 / `v248000_tests::template_gallery_has_5_entries` の len アサーションスタブ化 / `v37600_tests::cargo_toml_version_is_37_6_0` スタブ化 / `v37700_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.6.0"` → `"37.7.0"` |
| `CHANGELOG.md` | 追記 | `[v37.7.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.7.0 を完了済みにマーク（✅）・テスト件数を 1 件 → 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v37.7.0、次バージョン v37.8.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.7.0] エントリ追加

`## [v37.6.0]` の `---` セパレータ直後に挿入:

```markdown
## [v37.7.0] - 2026-07-09

### Added
- `fav new --template multi-source` — マルチソース ETL プロジェクトテンプレート追加
- `TEMPLATE_GALLERY` に `"multi-source"` エントリ追加（6 エントリ）
- `v37700_tests` 3 テスト追加

---
```

### Step 2: `driver.rs` — `create_multi_source_etl_project` 追加

`create_data_contract_project` の閉じ `}` の直後（`// ── module loading ──` の前）に挿入。

spec.md §1 のコードブロックに従い実装。生成ファイル:
- `src/load_customers.fav`（Postgres ロード）
- `src/load_orders.fav`（CSV ロード）
- `src/main.fav`（`List.join_on` 結合パイプライン）
- `fav.toml`
- `README.md`
- `.github/workflows/ci.yml`

### Step 3: `driver.rs` — `try_cmd_new` 更新

1. `"data-contract"` アームの直後に `"multi-source"` アームを追加
2. `other` アームのエラーメッセージ末尾に `|multi-source` を追記

**挿入位置の確認:** `"data-contract" => create_data_contract_project(&root, name),` の行を Read で特定してから Edit。

### Step 4: `driver.rs` — `TEMPLATE_GALLERY` 更新

`("data-contract", ...)` エントリの直後に追加:
```rust
("multi-source", "マルチソース ETL（複数 DB/CSV 結合）"),  // v37.7.0
```

### Step 5: `driver.rs` — `cmd_new_list` 更新

`cmd_new_list` には `"data-contract"` 行が欠落している（TEMPLATE_GALLERY には登録済みだが未追加）。
`"distributed-etl"` 行の直後に `"data-contract"` と `"multi-source"` の 2 行を同時に追加する:

```rust
println!("  {:<17} {}", "data-contract",   "Data Contract スキーマ定義プロジェクト");
println!("  {:<17} {}", "multi-source",    "マルチソース ETL（Postgres + CSV 結合）");
```

### Step 6: `driver.rs` — `v248000_tests::template_gallery_has_5_entries` のスタブ化

既存コメント行（`// v36.5.0 で...`）と `assert_eq!(TEMPLATE_GALLERY.len(), 5, ...)` の 2 行の
合計 3 行を、スタブコメント 1 行に置き換える。名前確認アサーション群は維持。
spec.md §5 の形式に従う（3 行 → 1 行）。

### Step 7: `driver.rs` — `v37600_tests::cargo_toml_version_is_37_6_0` をスタブ化

ライブアサーション → `// Stubbed: version bumped to 37.7.0 — assertion intentionally removed` に変更。

**注意:** `changelog_has_v37_6_0` / `lineage_dot_contains_digraph` / `lineage_svg_contains_svg_tag` はスタブ化しない。

### Step 8: `driver.rs` — `v37700_tests` モジュール追加

`v37600_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §6 のコードブロックに従う:
- `use super::try_cmd_new`
- `cargo_toml_version_is_37_7_0`
- `changelog_has_v37_7_0`
- `fav_new_multi_source_ok`（tempdir + try_cmd_new + ファイル存在確認 + main.fav 内容確認）

### Step 9: Cargo.toml バージョン更新

Step 1〜8 完了後に `37.6.0` → `37.7.0` に更新。

### Step 10: ドキュメント更新

- `versions/roadmap/roadmap-v37.1-v38.0.md` の v37.7.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v37.7.0（最新安定版）・v37.8.0（次バージョン）に更新

## 依存関係

- `create_multi_source_etl_project` は `write_text_file` ヘルパーを使用（既存）
- `tempfile` は `[dev-dependencies]` に既登録
- `try_cmd_new` は `fn`（非 `pub`）だが同ファイル内テストから `super::` でアクセス可能
- `List.join_on` は v37.3.0 で実装済みの VM ビルトイン — template コードに記述するのみ（実行はしない）

## リスク

| リスク | 対処 |
|---|---|
| `v248000_tests::fav_new_unknown_template_errors` が `etl-csv-to-db` を期待している | エラーメッセージの先頭部分は変更しないため引き続きパス |
| `template_gallery_has_5_entries` の len アサーション以外がスタブ化される | Step 6 では len 行のみ除去し名前確認は維持 |
| `src/main.fav` のパスが `proj.join("src/main.fav")` で正しいか | `write_text_file` は `create_dir_all` で多段ディレクトリを自動作成するため問題なし |
| `v37600_tests` の閉じ `}` 行番号の特定 | T0 で確認し記録してから Edit |
