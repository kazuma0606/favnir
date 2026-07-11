# v38.9.0 実装計画 — v39.0 前調整・安定化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/docs/ai-overview.mdx` | 新規作成 | v38.x AI 支援機能概要ドキュメント |
| `fav/src/driver.rs` | 変更 | `v38800_tests::cargo_toml_version_is_38_8_0` のアサーション部分のみスタブ化 / `v38900_tests` 追加（4 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.8.0"` → `"38.9.0"` |
| `CHANGELOG.md` | 追記 | `[v38.9.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.9.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.9.0、次バージョン v39.0.0 |
| `versions/v36-v40/v38.9.0/tasks.md` | 更新 | COMPLETE ステータスに更新 |

## 実装順序

### Step 1: CHANGELOG.md に [v38.9.0] エントリ追加

`## [v38.8.0]` の直前に挿入（spec.md §3 のコードブロックに従う）。

### Step 2: `site/content/docs/ai-overview.mdx` 新規作成

v38.x AI 支援機能を一覧化する MDX ファイルを作成（spec.md §1 に従う）。
必須キーワード: `"fav suggest"`

### Step 3: `driver.rs` — `v38800_tests::cargo_toml_version_is_38_8_0` スタブ化

```rust
// Stubbed: version bumped to 38.9.0 — assertion intentionally removed
```

### Step 4: `driver.rs` — `v38900_tests` モジュール追加（Step 1〜2 完了後）

`v38800_tests` の閉じ `}` の行番号確認:
```
grep -n "v38800_tests\|v38900_tests\|ai_cookbook_files_exist" fav/src/driver.rs
```

`v38800_tests` の閉じ `}` の直後に追加（spec.md §2 のコードブロックに従う）。

### Step 5: Cargo.toml バージョン更新

Step 1〜4 完了後に `38.8.0` → `38.9.0` に更新。

### Step 6: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2781 passed, 0 failed

### Step 7: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.9.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v38.9.0（最新安定版）・v39.0.0（次バージョン）に更新
- `versions/v36-v40/v38.9.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 4 (v38900_tests: changelog_has_v38_9_0)
Step 2 (ai-overview.mdx) ───────────────────────► Step 4 (v38900_tests: ai_overview_doc_exists)
Step 2 ─────────────────────────────────────────► Step 6 (cargo test: include_str! がコンパイル通過)
Step 3 (stub v38800) ────────────────────────────► Step 5 (Cargo.toml bump)
Step 4 (v38900_tests) ───────────────────────────► Step 5 (Cargo.toml bump)
Step 5 (Cargo.toml) ─────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ───────────────────────────────► Step 7 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `include_str!("suggest.rs")` のパス誤り | `driver.rs` と同ディレクトリなので `"suggest.rs"` が正しい。`cargo build --tests` でパスを確認 |
| `include_str!("../../site/content/docs/ai-overview.mdx")` のパス誤り | MDX ファイル作成後に `cargo build --tests` でコンパイルエラーを確認 |
| MDX コードブロック内のバックティックネスト | Write ツールで直接書き込むため spec.md の外側バッククォートは無視する |
