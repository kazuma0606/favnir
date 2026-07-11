# v39.7.0 実装計画 — CI/CD ポリシーゲート

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 変更 | `generate_ci_yaml` に Policy check ステップ追加 / `v39600_tests::cargo_toml_version_is_39_6_0` スタブ化 / `v39700_tests` 追加（2 テスト）|
| `fav/Cargo.toml` | 更新 | `version = "39.6.0"` → `"39.7.0"` |
| `CHANGELOG.md` | 追記 | `[v39.7.0]` エントリ追加（`### Changed` セクション使用）|
| `versions/roadmap/roadmap-v39.1-v40.0.md` | 更新 | v39.7.0 を完了済みにマーク（✅）|
| `versions/current.md` | 更新 | 最新安定版 v39.7.0、次に切る版 v39.8.0 |
| `versions/v36-v40/v39.7.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T6 全チェック）|

> **新規ファイル・main.rs 変更なし**: v39.7.0 は `driver.rs` 内の変更のみ。

## 注記: Step 番号と tasks.md T 番号の対応

| plan.md Step | tasks.md T 番号 | 内容 |
|---|---|---|
| Step 1 | T1 | CHANGELOG 追加 |
| Step 2 | T2 | generate_ci_yaml 変更 |
| Step 3 | T3 | driver.rs スタブ化 |
| Step 4 | T4 | driver.rs v39700_tests 追加 |
| Step 5 | T5 | Cargo.toml バージョン更新 |
| Step 6 | T6 | cargo test 実行 + ドキュメント更新 |

tasks.md には T0（事前確認）が先頭に挿入されているため、Step N は T(N) に対応する（T0 を除く）。

## 実装順序

### Step 1: CHANGELOG.md に [v39.7.0] エントリ追加（tasks.md: T1）

`## [v39.6.0]` ヘッダ行の直前に挿入（`### Changed` セクション使用）。

### Step 2: `generate_ci_yaml` に Policy check ステップ追加（tasks.md: T2）

対象: `fav/src/driver.rs` の `pub fn generate_ci_yaml` 関数（行 15492 付近）

Read で現在の文字列リテラルの末尾を確認してから Edit。
`fav test` ステップの後に以下を追加:
```
           - name: Policy check\n\
             run: fav policy check --ci\n
```

**注意**: インデントはスペース 5 個 + `- name:` のパターンで既存行と統一すること。

### Step 3: `driver.rs` — `v39600_tests::cargo_toml_version_is_39_6_0` スタブ化（tasks.md: T3）

Grep で `cargo_toml_version_is_39_6_0` の行番号を確認 →
NOTE コメントとライブアサーションを:
```rust
// Stubbed: version bumped to 39.7.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v39_6_0` はスタブ化しない。

### Step 4: `driver.rs` — `v39700_tests` モジュール追加（tasks.md: T4、Step 1 完了後）

`v39600_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §2 のコードブロックに従う:
- 2 テスト: `cargo_toml_version_is_39_7_0`（NOTE コメント付き）/ `changelog_has_v39_7_0`

### Step 5: Cargo.toml バージョン更新（tasks.md: T5）

Step 1〜4 完了後に `39.6.0` → `39.7.0` に更新。

### Step 6: `cargo test` 実行 + ドキュメント更新（tasks.md: T6）

```
cargo test 2>&1 | grep "test result"
```

期待: ≥ 2805 passed, 0 failed

**追加確認**: 既存の `generate_ci_yaml_has_check_step` / `generate_ci_yaml_has_lint_step` / `generate_ci_yaml_has_test_step` が regression なく pass していることを確認。

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 4 (driver tests, changelog_has_v39_7_0)
Step 2 (generate_ci_yaml 変更) ──────────────────► Step 6 (既存テスト regression 確認)
Step 3 (stub v39600) ────────────────────────────► Step 4 (driver tests)
Step 4 (v39700_tests) ───────────────────────────► Step 5 (Cargo.toml bump)
Step 5 (Cargo.toml) ─────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ───────────────────────────────► docs 更新
```

## リスク

| リスク | 対処 |
|---|---|
| インデントのズレで YAML が不正になる | Read で既存行のインデントパターンを確認してから Edit |
| Policy check ステップ追加で `fav test` 行が重複 | Edit は追記のみ（`fav test` 行は変更しない） |
| 既存 `generate_ci_yaml_has_*` テストへの regression | `cargo test` 後に当該 3 テストの pass を確認 |
| `gen` 予約語（Rust 2024）| v39.7.0 テストでは `cargo`/`src` 変数のみ使用 — 問題なし |
