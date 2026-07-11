# v38.4.0 実装計画 — LSP AI 補完（オプション）

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/toml.rs` | 変更 | `LspAiConfig` struct + `parse_lsp_ai_config` + `parse_lsp_ai_enabled` 追加（末尾）|
| `fav/src/driver.rs` | 変更 | `v38300_tests::cargo_toml_version_is_38_3_0` スタブ化 / `v38400_tests` 追加（4 テスト） |
| `fav/Cargo.toml` | 更新 | `version = "38.3.0"` → `"38.4.0"` |
| `CHANGELOG.md` | 追記 | `[v38.4.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.4.0 を完了済みにマーク（✅）・テスト件数を 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.4.0、次バージョン v38.5.0 |
| `versions/v36-v40/v38.4.0/tasks.md` | 更新 | COMPLETE ステータスに更新（T0〜T9 全チェック）|

## 実装順序

### Step 1: CHANGELOG.md に [v38.4.0] エントリ追加

`## [v38.3.0]` の直前に挿入:

```markdown
## [v38.4.0] — 2026-07-10

### Added
- `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加
- `[lsp.ai] enabled = true` で LSP AI 補完を有効化（v38.7.0 で本実装）
- `v38400_tests` 4 テスト追加

---
```

### Step 2: `fav/src/toml.rs` — `LspAiConfig` + `parse_lsp_ai_config` 追加

Read で `toml.rs` の末尾行番号を確認 → `parse_lsp_ai_enabled` セクションを末尾に追加。
spec.md §1 のコードブロックに従う。

### Step 3: `driver.rs` — `v38300_tests::cargo_toml_version_is_38_3_0` スタブ化

```rust
// Stubbed: version bumped to 38.4.0 — assertion intentionally removed
```

### Step 4: `driver.rs` — `v38400_tests` モジュール追加（Step 1・2 完了後）

`v38300_tests` の閉じ `}` の直後に追加（spec.md §2 のコードブロックに従う）。

### Step 5: Cargo.toml バージョン更新

Step 1〜4 完了後に `38.3.0` → `38.4.0` に更新。

### Step 6: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2758 passed, 0 failed

### Step 7: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.4.0 を ✅ にマーク・テスト件数を 4 件に更新
- `versions/current.md` を v38.4.0（最新安定版）・v38.5.0（次バージョン）に更新
- `versions/v36-v40/v38.4.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ─────────────────────────────► Step 4 (driver tests, changelog_has_v38_4_0)
Step 2 (toml.rs) ───────────────────────────────► Step 4 (driver tests, lsp_ai_*)
                  ───────────────────────────────► Step 6 (cargo test, コンパイル通過)
Step 3 (stub v38300) ───────────────────────────► Step 6 (cargo test)
Step 4 (v38400_tests) ──────────────────────────► Step 5 (Cargo.toml bump)
                      ──────────────────────────► Step 6 (cargo test)
                      ※ Step 4 はテストコードの追加のみ。cargo_toml_version_is_38_4_0 が pass するのは Step 5 完了後の Step 6 実行時
Step 5 (Cargo.toml) ────────────────────────────► Step 6 (cargo test)
Step 6 (all pass) ──────────────────────────────► Step 7 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `[lsp.formatting]` 等の類似セクションで `in_lsp_ai` が誤って true になる | `trimmed.starts_with('[')` で確実にリセット |
| `crate::toml::parse_lsp_ai_config` がテストからアクセスできない | `lib.rs` に `pub mod toml;` が宣言済み（`lib.rs` 63行目確認済み）のため `crate::toml::` パスは解決される — 追加関数も `pub fn` にする |
| `gen` 予約語 | `in_lsp_ai` / `trimmed` を使用し `gen` は不使用 |
