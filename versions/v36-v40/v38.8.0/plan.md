# v38.8.0 実装計画 — AI 支援 cookbook 3 本

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/cookbook/sql-to-favnir.mdx` | 新規作成 | `fav generate --from sql` 使用例 |
| `site/content/cookbook/rag-pipeline.mdx` | 新規作成 | `llm.embed` + RAG テンプレート使用例 |
| `site/content/cookbook/llm-streaming.mdx` | 新規作成 | `llm.stream` 使用例・v38.7.0 制約説明 |
| `fav/src/driver.rs` | 変更 | `v38700_tests` スタブ化 / `v38800_tests` 追加（3 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.7.0"` → `"38.8.0"` |
| `CHANGELOG.md` | 追記 | `[v38.8.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.8.0 を完了済みにマーク（✅）・テスト件数を 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.8.0、次バージョン v38.9.0 |
| `versions/v36-v40/v38.8.0/tasks.md` | 更新 | COMPLETE ステータスに更新 |

## 実装順序

### Step 1: CHANGELOG.md に [v38.8.0] エントリ追加

`## [v38.7.0]` の直前に挿入（spec.md §5 のコードブロックに従う）。

### Step 2: `site/content/cookbook/sql-to-favnir.mdx` 新規作成

`fav generate --from sql` のコード例を含む MDX ファイルを作成（spec.md §1 に従う）。
必須キーワード: `"fav generate"`

### Step 3: `site/content/cookbook/rag-pipeline.mdx` 新規作成

`llm.embed` + `fav new --template rag-pipeline` のコード例を含む MDX ファイルを作成（spec.md §2 に従う）。
必須キーワード: `"llm.embed"`

### Step 4: `site/content/cookbook/llm-streaming.mdx` 新規作成

`llm.stream` のコード例と v38.7.0 collect-all 制約説明を含む MDX ファイルを作成（spec.md §3 に従う）。
必須キーワード: `"llm.stream"`

### Step 5: `driver.rs` — `v38700_tests::cargo_toml_version_is_38_7_0` スタブ化

```rust
// Stubbed: version bumped to 38.8.0 — assertion intentionally removed
```

### Step 6: `driver.rs` — `v38800_tests` モジュール追加（Step 1 完了後）

`v38700_tests` の閉じ `}` の直後に追加（spec.md §4 のコードブロックに従う）。

`v38700_tests` の閉じ `}` の行番号確認:
```
grep -n "v38700_tests\|v38800_tests\|llm_test_fav_has_new_functions" fav/src/driver.rs
```

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.7.0` → `38.8.0` に更新。

### Step 8: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2777 passed, 0 failed

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.8.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v38.8.0（最新安定版）・v38.9.0（次バージョン）に更新
- `versions/v36-v40/v38.8.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 6 (v38800_tests: changelog_has_v38_8_0)
Step 2 (sql-to-favnir.mdx) ──────────────────────► Step 6 (v38800_tests: ai_cookbook_files_exist)
Step 3 (rag-pipeline.mdx) ───────────────────────► Step 6 (v38800_tests: ai_cookbook_files_exist)
Step 4 (llm-streaming.mdx) ──────────────────────► Step 6 (v38800_tests: ai_cookbook_files_exist)
Step 2 + 3 + 4 ──────────────────────────────────► Step 8 (cargo test: include_str! がコンパイル通過)
Step 5 (stub v38700) ────────────────────────────► Step 7 (Cargo.toml bump)
Step 6 (v38800_tests) ───────────────────────────► Step 7 (Cargo.toml bump)
Step 7 (Cargo.toml) ─────────────────────────────► Step 8 (cargo test)
Step 8 (all pass) ───────────────────────────────► Step 9 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `include_str!("../../site/content/cookbook/xxx.mdx")` のパスが誤り | MDX ファイル作成後に `cargo build --tests` でコンパイルエラーを確認 |
| `rag-pipeline.mdx` が既存の `pinecone-rag.mdx` と混同される | 別ファイル（`rag-pipeline.mdx`）であることをテスト名と MDX frontmatter title で区別 |
| MDX のコードブロック内にバックティック 3 個が入れ子になって frontmatter が破損 | spec.md のコードブロックはそのまま使用（Markdown のコードブロック入れ子は ` 数で区別） |
