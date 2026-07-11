# v39.9.0 実装計画 — v40.0 前調整・安定化 + 全スプリント振り返り

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/docs/enterprise-governance.mdx` | 新規作成 | v39 スプリント振り返り + Enterprise Governance 概要 |
| `fav/src/driver.rs` | 変更 | `v39800_tests::cargo_toml_version_is_39_8_0` スタブ化 / `v39900_tests` 追加（2 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "39.8.0"` → `"39.9.0"` |
| `CHANGELOG.md` | 追記 | `[v39.9.0]` エントリ追加（`### Added` セクション使用） |
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.9.0 を完了済みにマーク（✅） |
| `versions/current.md` | 更新 | 最新安定版 v39.9.0、次に切る版 v40.0.0 |
| `versions/v36-v40/v39.9.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T6 全チェック） |

> **新規 Rust ソースファイル・main.rs 変更なし・MILESTONE.md 変更なし**: v39.9.0 は MDX 1 件と meta テスト 2 件のみ。

## 注記: Step 番号と tasks.md T 番号の対応

| plan.md Step | tasks.md T 番号 | 内容 |
|---|---|---|
| Step 1 | T1 | CHANGELOG 追加 |
| Step 2 | T2 | enterprise-governance.mdx 作成 |
| Step 3 | T3 | driver.rs スタブ化 |
| Step 4 | T4 | driver.rs v39900_tests 追加 |
| Step 5 | T5 | Cargo.toml バージョン更新 |
| Step 6 | T6 | cargo test 実行 + ドキュメント更新 |

tasks.md には T0（事前確認）が先頭に挿入されているため、Step N は T(N) に対応する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.9.0] エントリ追加（tasks.md: T1）

`## [v39.8.0]` ヘッダ行の直前に挿入（`### Added` セクション使用）。

### Step 2: `site/content/docs/enterprise-governance.mdx` 作成（tasks.md: T2）

Write ツールで新規作成。内容:
- frontmatter（title / description）
- v39 スプリント達成サマリー（8 バージョンの機能一覧テーブル）
- v40.0 宣言文（暫定）
- 各ドキュメントへの参照リンク（docs/governance/ 3 件、cookbook 3 件）

### Step 3: `driver.rs` — `v39800_tests::cargo_toml_version_is_39_8_0` スタブ化（tasks.md: T3）

Grep で `cargo_toml_version_is_39_8_0` の行番号を確認 →
NOTE コメントとライブアサーションを:
```rust
// Stubbed: version bumped to 39.9.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_8_0` / `site_has_governance_docs` はスタブ化しない。

### Step 4: `driver.rs` — `v39900_tests` モジュール追加（tasks.md: T4、Step 1 完了後）

`v39800_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §2 のコードブロックに従う:
- 2 テスト: `cargo_toml_version_is_39_9_0`（NOTE コメント付き）/ `changelog_has_v39_9_0`

### Step 5: Cargo.toml バージョン更新（tasks.md: T5）

Step 1〜4 完了後に `39.8.0` → `39.9.0` に更新。

### Step 6: `cargo test` 実行 + ドキュメント更新（tasks.md: T6）

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2810 passed, 0 failed

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 4 (driver tests, changelog_has_v39_9_0)
Step 2 (enterprise-governance.mdx) ──────────────► Step 6 (目視確認)
Step 3 (stub v39800) ────────────────────────────► Step 4 (driver tests)
Step 4 (v39900_tests) ───────────────────────────► Step 5 (Cargo.toml bump)
Step 5 (Cargo.toml) ─────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ───────────────────────────────► docs 更新
```

## リスク

| リスク | 対処 |
|---|---|
| `enterprise-governance.mdx` の内容が v39.1〜v39.8 と不整合 | spec.md §1 の機能一覧テーブルを参照して作成 |
| MILESTONE.md を誤って変更する | MILESTONE.md は v40.0.0 スコープ — 本バージョンでは触れない |
| `changelog_has_v39_8_0` / `site_has_governance_docs` を誤ってスタブ化する | T3 の注意事項を必ず確認 |
| `gen` 予約語（Rust 2024）| v39.9.0 テストでは `cargo`/`src` 変数のみ使用 — 問題なし |
