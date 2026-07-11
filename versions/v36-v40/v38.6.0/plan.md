# v38.6.0 実装計画 — RAG パイプラインテンプレート

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 変更 | `create_rag_pipeline_project` 追加 / `try_cmd_new` に `"rag-pipeline"` アーム追加 / `cmd_new_list` 更新 / エラーメッセージ更新 / `v38500_tests` スタブ化 / `v38600_tests` 追加（4 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.5.0"` → `"38.6.0"` |
| `CHANGELOG.md` | 追記 | `[v38.6.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.6.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.6.0、次バージョン v38.7.0 |
| `versions/v36-v40/v38.6.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T7 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.6.0] エントリ追加

`## [v38.5.0]` の直前に挿入（spec.md §5 のコードブロックに従う）。

### Step 2: `driver.rs` — `create_rag_pipeline_project` 関数追加

`create_multi_source_etl_project` の `Ok(())` / `}` (line 741〜742) の直後・`// ── module loading ──` セクション区切りコメント（line 744）の**前**に挿入する（spec.md §1 のコードブロックに従う）。

生成ファイル:
- `src/main.fav`: Ingest / Embed / Retrieve / Generate 4 ステージ + pipeline 宣言
- `fav.toml`: `[runes] llm = "1.0.0"`, `csv = "1.0.0"`
- `data/documents.csv`: サンプルデータ 2 行
- `README.md`: 使い方説明

### Step 3: `driver.rs` — `try_cmd_new` に `"rag-pipeline"` アーム追加

`"multi-source"` アームの直後、`other =>` の直前に追加:
```rust
"rag-pipeline"     => create_rag_pipeline_project(&root, name),
```

エラーメッセージの `multi-source` の直後・`)` の直前に `|rag-pipeline` を追加（`multi-source|rag-pipeline)`）。

### Step 4: `driver.rs` — `cmd_new_list` に `rag-pipeline` 追加

`"multi-source"` 行の直後に追加（spec.md §3 のコードブロックに従う）。

### Step 5: `driver.rs` — `v38500_tests::cargo_toml_version_is_38_5_0` スタブ化

```rust
// Stubbed: version bumped to 38.6.0 — assertion intentionally removed
```

**注意**: `changelog_has_v38_5_0` / `explain_verbose_*` テストはスタブ化しない。

### Step 6: `driver.rs` — `v38600_tests` モジュール追加（Step 1 完了後）

T0 で記録した `v38500_tests` の閉じ `}` の行番号（現時点の参考値: 43826 行目）を Read で特定してから Edit。
`v38500_tests` の閉じ `}` の直後に追加（spec.md §4 のコードブロックに従う）。

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.5.0` → `38.6.0` に更新。

### Step 8: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2769 passed, 0 failed

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.6.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v38.6.0（最新安定版）・v38.7.0（次バージョン）に更新
- `versions/v36-v40/v38.6.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 6 (driver tests, changelog_has_v38_6_0)
Step 2 (create_rag_pipeline_project) ───────────► Step 3 (try_cmd_new アーム、コンパイル通過)
                                     ───────────► Step 6 (driver tests, rag_pipeline_fn_exists)
                                     ───────────► Step 6 (driver tests, rag_pipeline_has_four_stages)
                                     ───────────► Step 8 (cargo test, コンパイル通過)
Step 3 (try_cmd_new) ───────────────────────────► Step 8 (cargo test)
Step 4 (cmd_new_list) ──────────────────────────► Step 8 (cargo test)
Step 5 (stub v38500) ───────────────────────────► Step 8 (cargo test)
Step 6 (v38600_tests) ──────────────────────────► Step 7 (Cargo.toml bump)
Step 6 (v38600_tests) ──────────────────────────► Step 8 (cargo test)
Step 7 (Cargo.toml) ────────────────────────────► Step 8 (cargo test)
                    ※ cargo_toml_version_is_38_6_0 が pass するのは Step 7 完了後の Step 8 実行時
Step 8 (all pass) ──────────────────────────────► Step 9 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `rag_pipeline_has_four_stages` テストが既存テキストの偶然一致でパスしてしまう | `rag_pipeline_fn_exists` と組み合わせて「関数が存在し、かつ 4 ステージ名が含まれる」ことを担保 |
| `\x20\x20\x20\x20` が意図したインデントにならない | raw string `r#"..."#` を使うか、テスト実行で `fav run` が通ることを確認（v38.6.0 ではスモークテストのみ）|
| `gen` 予約語（Rust 2024） | `create_rag_pipeline_project` では `root`・`name` のみ使用 |
| エラーメッセージの末尾更新を忘れる | Step 3 のチェックリストに明示 |
