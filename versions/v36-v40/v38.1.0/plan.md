# v38.1.0 実装計画 — `fav suggest`

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/suggest.rs` | 新規作成 | `cmd_suggest` / `builtin_hint` / `llm_suggest` 実装 |
| `fav/src/main.rs` | 変更 | `mod suggest;` 追加 + `Some("suggest")` ディスパッチアーム追加 |
| `fav/src/driver.rs` | 変更 | `v38000_tests::cargo_toml_version_is_38_0_0` スタブ化 / `v38100_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "38.0.0"` → `"38.1.0"` |
| `CHANGELOG.md` | 追記 | `[v38.1.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.1.0 を完了済みにマーク（✅）・テスト件数を 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.1.0、次バージョン v38.2.0 |
| `versions/v36-v40/v38.1.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T9 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.1.0] エントリ追加

`## [v38.0.0]` の `---` セパレータ直後に挿入:

```markdown
## [v38.1.0] — 2026-07-10

### Added
- `fav/src/suggest.rs` — `fav suggest <error-code> <file:line>` コマンド追加
- `builtin_hint`: E0001 / E0007 / E0008 の組み込みヒント
- `ANTHROPIC_API_KEY` 設定時は LLM 提案（v38.7.0 で本実装予定、現在スタブ）
- `v38100_tests` 3 テスト追加

---
```

**注意**: セパレータは `—`（全角ダッシュ U+2014）

### Step 2: `fav/src/suggest.rs` 新規作成

spec.md §1 の内容で作成。以下を含む:
- `pub fn cmd_suggest(error_code: &str, location: &str) -> Result<(), String>`
- `fn read_source(location: &str) -> Result<String, String>`
- `fn builtin_hint(error_code: &str) -> String`（E0001 / E0007 / E0008 組み込みヒント）
- `fn llm_suggest(...) -> String`（現在スタブ: `builtin_hint` にフォールバック）

### Step 3: `fav/src/main.rs` — `mod suggest;` 追加 + `Some("suggest")` アーム追加

Read で `mod rune_cmd;` の行番号を確認 → `mod suggest;` を直後に追加。

Read で `Some("registry")` の行番号を確認 → `Some("suggest")` アームを同じ match ブロック内に追加:

```rust
Some("suggest") => {
    let error_code = args.get(2).map(|s| s.as_str()).unwrap_or("E0001");
    let location   = args.get(3).map(|s| s.as_str()).unwrap_or("main.fav:1");
    if let Err(e) = suggest::cmd_suggest(error_code, location) {
        eprintln!("fav suggest error: {}", e);
        std::process::exit(1);
    }
}
```

**注意**: Step 2 で `suggest.rs` を作成してから Step 3 の `mod suggest;` を追加すること（コンパイルが通る順序）。

### Step 4: `driver.rs` — `v38000_tests::cargo_toml_version_is_38_0_0` スタブ化

Read で `cargo_toml_version_is_38_0_0` の行番号を確認 → ライブアサーションを:
```rust
// Stubbed: version bumped to 38.1.0 — assertion intentionally removed
```
に変更。

**注意**: `changelog_has_v38_0_0` / `milestone_has_multi_source_etl_power` / `readme_mentions_multi_source_etl` はスタブ化しない。

### Step 5: `driver.rs` — `v38100_tests` モジュール追加（Step 4 完了後）

`v38000_tests` の閉じ `}` の行番号を Read で特定してから Edit。
spec.md §3 のコードブロックに従う:
- imports 不要（`include_str!` のみ使用）
- 3 テスト: `cargo_toml_version_is_38_1_0` / `changelog_has_v38_1_0` / `suggest_fn_exists`

`suggest_fn_exists` は `include_str!("suggest.rs")` を使用 — Step 2 完了後に追加すること。

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `38.0.0` → `38.1.0` に更新。

### Step 7: `cargo test` 実行・全通過確認

`cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | tail -5`

期待: ≥ 2744 passed, 0 failed

### Step 8: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.1.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v38.1.0（最新安定版）・v38.2.0（次バージョン）に更新
- `versions/v36-v40/v38.1.0/tasks.md` を COMPLETE ステータスに更新（T0〜T9 全チェックボックスを `[x]` に）

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 5 (driver tests, changelog_has_v38_1_0)
Step 2 (suggest.rs) ────────────────────────────► Step 3 (main.rs mod + dispatch)
                    ────────────────────────────► Step 5 (driver tests, suggest_fn_exists)
Step 3 (main.rs) ───────────────────────────────► Step 7 (cargo test)
Step 4 (stub v38000) ───────────────────────────► Step 5 (driver tests)
Step 5 (v38100_tests) ──────────────────────────► Step 6 (Cargo.toml bump)
Step 5 (v38100_tests) ──────────────────────────► Step 7 (cargo test)
Step 6 (Cargo.toml) ────────────────────────────► Step 7 (cargo test)
Step 7 (all pass) ──────────────────────────────► Step 8 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `suggest.rs` の `std::fs::read_to_string` が test 環境で失敗 | `cmd_suggest` はテストで直接呼ばず `include_str!` のみ検証 |
| `main.rs` dispatch アームの挿入位置がずれる | Read で `Some("registry")` 前後を確認してから Edit |
| `mod suggest;` を先に追加すると `suggest.rs` がなくてコンパイルエラー | Step 2 → Step 3 の順序を厳守 |
| `gen` 予約語（Rust 2024） | v38.1.0 テストでは `suggest` 系変数のみ使用 — 問題なし |
