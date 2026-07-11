# v38.7.0 実装計画 — Llm Rune 強化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/backend/vm.rs` | 変更 | `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` ディスパッチ追加 + `llm_embed` ヘルパー関数追加 |
| `fav/src/driver.rs` | 変更 | primitives テーブルに 3 エントリ追加 / `v38600_tests` スタブ化 / `v38700_tests` 追加（3 テスト） |
| `runes/llm/client.fav` | 変更 | `stream` / `function_call` / `embed` 公開関数追加 |
| `runes/llm/llm.fav` | 変更 | `use client.{...}` に `stream, function_call, embed` を追加 |
| `runes/llm/llm.test.fav` | 変更 | `llm_stream_no_key_is_err` / `llm_function_call_no_key_is_err` / `llm_embed_no_provider_is_err` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "38.6.0"` → `"38.7.0"` |
| `CHANGELOG.md` | 追記 | `[v38.7.0]` エントリ追加 |
| `versions/roadmap/roadmap-v38.1-v39.0.md` | 更新 | v38.7.0 を完了済みにマーク（✅）・テスト件数を 3 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v38.7.0、次バージョン v38.8.0 |
| `versions/v36-v40/v38.7.0/tasks.md` | 更新 | COMPLETE ステータスに更新 |

## 実装順序

### Step 1: CHANGELOG.md に [v38.7.0] エントリ追加

`## [v38.6.0]` の直前に挿入（spec.md §7 のコードブロックに従う）。

### Step 2: `vm.rs` — `Llm.stream_raw` / `Llm.function_call_raw` / `Llm.embed_raw` ディスパッチ追加

`Llm.extract_raw` ブロックの末尾 `}` の直後、`// ── Snowflake` セクションコメントの前に挿入（spec.md §1 のコードブロックに従う）。

行番号の確認方法:
```
grep -n "Snowflake.execute_raw" fav/src/backend/vm.rs | head -3
```

### Step 3: `vm.rs` — `llm_embed` ヘルパー関数追加

`llm_call_chat` 関数の終端 `}` の直後（line 6677 付近、`// ── Snowflake helpers (v10.2.0)` コメントの前）に追加（spec.md §1 のコードブロックに従う）。

行番号の確認方法:
```
grep -n "^fn llm_call_chat\|Snowflake helpers" fav/src/backend/vm.rs | head -5
```

**注意**:
- `#[cfg(not(target_arch = "wasm32"))]` は `fn llm_embed` の**直前のみ**付与すること。
- ディスパッチアーム（Step 2 の `"Llm.embed_raw" => { ... }` ブロック）には付与しない。

### Step 4: `driver.rs` — primitives テーブルに 3 エントリ追加

`Llm.extract_raw` エントリの直後に追加（spec.md §2 のコードブロックに従う）。

行番号の確認方法:
```
grep -n "Llm.extract_raw" fav/src/driver.rs
```

### Step 5: `runes/llm/client.fav` — 3 公開関数追加

`extract<T>` 関数の末尾 `}` の直後に追加（spec.md §3 のコードブロックに従う）。

### Step 6: `runes/llm/llm.fav` — use 宣言を更新

```favnir
use client.{ complete, chat, extract, stream, function_call, embed }
```

### Step 7: `runes/llm/llm.test.fav` — 3 テスト追加

既存 3 テストの直後に追加（spec.md §5 のコードブロックに従う）。

### Step 8: `driver.rs` — `v38600_tests::cargo_toml_version_is_38_6_0` スタブ化

```rust
// Stubbed: version bumped to 38.7.0 — assertion intentionally removed
```

### Step 9: `driver.rs` — `v38700_tests` モジュール追加

`v38600_tests` の閉じ `}` の直後に追加（spec.md §6 のコードブロックに従う）。

`v38600_tests` の閉じ `}` の行番号確認:
```
grep -n "v38600_tests\|v38700_tests" fav/src/driver.rs
```

### Step 10: Cargo.toml バージョン更新

Step 1〜9 完了後に `38.6.0` → `38.7.0` に更新。

### Step 11: `cargo test` 実行・全通過確認

```
cd /c/Users/yoshi/favnir/fav && cargo test 2>&1 | grep "test result"
```

期待: ≥ 2773 passed, 0 failed（`llm.test.fav` の Favnir テスト 3 件は `cargo test` カウントに含まれないため、Rust テスト +3 件のみカウント増加）

### Step 12: ドキュメント更新

- `versions/roadmap/roadmap-v38.1-v39.0.md` の v38.7.0 を ✅ にマーク・テスト件数を 3 件に更新
- `versions/current.md` を v38.7.0（最新安定版）・v38.8.0（次バージョン）に更新
- `versions/v36-v40/v38.7.0/tasks.md` を COMPLETE ステータスに更新

## 依存関係

```
Step 1 (CHANGELOG) ──────────────────────────────► Step 9 (v38700_tests: changelog_has_v38_7_0)
Step 2 (vm.rs dispatch) ─────────────────────────► Step 3 (llm_embed helper, コンパイル依存)
Step 2 + Step 3 ─────────────────────────────────► Step 11 (cargo test, コンパイル通過)
Step 4 (driver primitives) ──────────────────────► Step 9 (v38700_tests: primitives_exist に間接関係)
Step 5 (client.fav) ─────────────────────────────► Step 6 (llm.fav use 宣言)
Step 5 + Step 6 ─────────────────────────────────► Step 7 (llm.test.fav)
Step 7 + existing llm_rune_test_file_passes ─────► Step 11 (cargo test, Favnir テスト通過)
Step 8 (stub v38600) ────────────────────────────► Step 10 (Cargo.toml bump, スタブ前に 38.6.0 を検証しない)
Step 9 (v38700_tests) ───────────────────────────► Step 10 (Cargo.toml bump)
Step 10 (Cargo.toml) ────────────────────────────► Step 11 (cargo test)
Step 11 (all pass) ──────────────────────────────► Step 12 (docs)
```

## リスク

| リスク | 対処 |
|---|---|
| `include_str!("../backend/vm.rs")` のパスが誤り | T2 でコンパイル後にパスを確認（コンパイルエラーで即判明） |
| `#[cfg(not(target_arch = "wasm32"))]` 忘れ | `llm_call_chat` の定義行を Read で確認し、同様のアノテーションをコピー |
| `llm_embed_no_provider_is_err` テストが Favnir テストファイルで失敗 | `ANTHROPIC_API_KEY` を unset する既存 `llm_rune_test_file_passes` の前処理に依存（確認必要） |
| `llm.fav` の `use` 宣言更新漏れ | Step 6 をチェックリストで明示 |
| `function_call_raw` の `tools_json` 引数が `format!` でパニック | `tools_json` は文字列として扱うだけで展開しない（`format!` の `{}` ではなく `{}` プレースホルダーのみ） |
