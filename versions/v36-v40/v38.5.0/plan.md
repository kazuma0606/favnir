# v38.5.0 実装計画 — `fav explain --verbose` LLM 拡張

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/explain_verbose.rs` | 新規作成 | `explain_verbose` + `base_explanation` 実装 |
| `fav/src/main.rs` | 変更 | `pub(crate) mod explain_verbose;` 追加 + `--verbose` 分岐追加 |
| `fav/src/driver.rs` | 変更 | `v38400_tests::cargo_toml_version_is_38_4_0` スタブ化 / `v38500_tests` 追加（4 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.4.0"` → `"38.5.0"` |
| `CHANGELOG.md` | 追記 | `[v38.5.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.5.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.5.0、次バージョン v38.6.0 |
| `versions/v36-v40/v38.5.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T7 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.5.0] エントリ追加

`## [v38.4.0]` の直前に挿入（spec.md §4 のコードブロックに従う）。

### Step 2: `fav/src/explain_verbose.rs` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `pub fn explain_verbose(error_code: &str, location: &str) -> String`
- `fn base_explanation(error_code: &str) -> String`（E0001 / E0007 / E0008 + デフォルト）

**キーワード確認**（テストが検索する文字列）:
- `explain_verbose_basic` テスト: `result.contains("E0001") && result.contains("Fix suggestion")`
- `explain_verbose_with_location` テスト: `result.contains("Context") && result.contains("main.fav:12")`

### Step 3: `fav/src/main.rs` — `pub(crate) mod explain_verbose;` 追加

Read で `pub(crate) mod generate_csv;` の行番号を確認 → 直後に `pub(crate) mod explain_verbose;` を追加。

**注意**: Step 2 で `explain_verbose.rs` を作成してから Step 3 を実施（コンパイルが通る順序）。

### Step 4: `fav/src/main.rs` — `--verbose` 分岐追加

Read で `Some("explain")` アーム内の `if args.get(2) ... == Some("compiler")` ブロックの `return;` の行番号を確認。
その**直後**（`if args.iter().any(|a| a == "--sla")` の直前）に `--verbose` チェックブロックを追加（spec.md §2 のコードブロックに従う）。

**挿入位置の根拠**: `compiler` チェックを先行させることで `fav explain compiler --verbose` が `compiler` パスを正しく実行する（spec.md §注意事項を参照）。

### Step 5: `driver.rs` — `v38400_tests::cargo_toml_version_is_38_4_0` スタブ化

```rust
// Stubbed: version bumped to 38.5.0 — assertion intentionally removed
```

**注意**: `changelog_has_v38_4_0` / `lsp_ai_*` テストはスタブ化しない。

### Step 6: `driver.rs` — `v38500_tests` モジュール追加（Step 1・2 完了後）

`v38400_tests` の閉じ `}` の行番号（参考値: 43784）を Read で特定してから Edit。
spec.md §3 のコードブロックに従い 4 テストを追加。

### Step 7: Cargo.toml バージョン更新

Step 1〜6 完了後に `38.4.0` → `38.5.0` に更新。

### Step 8: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2764 passed, 0 failed

### Step 9: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.5.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v38.5.0（最新安定版）・v38.6.0（次バージョン）に更新
- `versions/v36-v40/v38.5.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 6 (driver tests, changelog_has_v38_5_0)
Step 2 (explain_verbose.rs) ────────────────────► Step 3 (main.rs mod explain_verbose;)
                             ───────────────────► Step 4 (main.rs --verbose arm)
                             ───────────────────► Step 6 (driver tests, explain_verbose_*)
                             ───────────────────► Step 8 (cargo test, コンパイル通過)
Step 3 (main.rs mod) ───────────────────────────► Step 4 (main.rs arm, コンパイル通過)
                     ───────────────────────────► Step 8 (cargo test, mod 宣言がコンパイルに必須)
Step 4 (main.rs arm) ───────────────────────────► Step 8 (cargo test)
Step 5 (stub v38400) ───────────────────────────► Step 8 (cargo test)
Step 6 (v38500_tests) ──────────────────────────► Step 7 (Cargo.toml bump)
Step 6 (v38500_tests) ──────────────────────────► Step 8 (cargo test)
Step 7 (Cargo.toml) ────────────────────────────► Step 8 (cargo test)
                    ※ cargo_toml_version_is_38_5_0 が pass するのは Step 7 完了後の Step 8 実行時
Step 8 (all pass) ──────────────────────────────► Step 9 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `pub(crate) mod explain_verbose;` を先に追加すると `explain_verbose.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
| `--verbose` チェックを `Some("explain")` アームの後半に挿入すると既存フラグと干渉 | アームの**最初**に挿入（`if args.get(2) == Some("compiler")` の直前） |
| `location` をファイル読み込みに使うパス traversal の危険 | v38.5.0 ではファイル読み込みなし（スタブのため表示のみ）— v38.7.0 で `..` チェック追加 |
| `gen` 予約語（Rust 2024） | `explain_verbose.rs` では `error_code`・`location`・`base`・`context_note` を使用 |
| `lib.rs` に `pub mod explain_verbose` がないと `crate::explain_verbose` が解決できない | binary crate（`main.rs`）で `pub(crate) mod explain_verbose;` を宣言 → driver.rs テストからは `crate::explain_verbose` でアクセス可能（generate_csv と同構造） |
