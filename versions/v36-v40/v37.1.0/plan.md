# v37.1.0 実装計画 — 境界付きジェネリクス実用強化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/middle/checker.rs` | 変更 | `type_implements_bound` に `Deserialize` を明示追加 |
| `fav/src/driver.rs` | 変更 | `v37000_tests` スタブ化 / `v37100_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.0.0"` → `"37.1.0"` |
| `CHANGELOG.md` | 追記 | `[v37.1.0]` エントリ追加 |
| `runes/generic/generic.fav` | 新規作成 | Generic Rune — 型パラメータ付き汎用関数 |
| `runes/generic/rune.toml` | 新規作成 | Generic Rune メタデータ |
| `versions/current.md` | 更新 | 最新安定版 v37.1.0、次バージョン v37.2.0 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.1.0 完了済みにマーク（✅） |

## 実装順序

### Step 1: CHANGELOG.md に [v37.1.0] エントリ追加

`## [v37.0.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

```markdown
## [v37.1.0] - 2026-07-09

### Added
- `Deserialize` 型制約を `type_implements_bound` に明示追加（`middle/checker.rs`）
- `T with Deserialize` が型チェックと実行を通ることをテストで保証
- Generic Rune (`runes/generic/`) — 型パラメータ付き汎用 ETL 関数の参照実装

---
```

### Step 2: `middle/checker.rs` — `Deserialize` を明示的な有効制約に追加

実装箇所: `fav/src/middle/checker.rs` の `type_implements_bound` 関数（行 7596〜7609）

```bash
# 実装箇所の確認
# "Eq" | "Serialize" | "Clone" => true の行
grep -n '"Serialize"' fav/src/middle/checker.rs
```

**変更前（行 7599 付近）:**
```rust
"Eq" | "Serialize" | "Clone" => true,
```

**変更後:**
```rust
"Eq" | "Serialize" | "Deserialize" | "Clone" => true,
```

変更は 1 行のみ。コンパイルエラーなし（既存の `_ => true` フォールスルーを明示パターンに格上げするだけ）。

### Step 3: runes/generic/ 新規作成

`runes/generic/generic.fav`（spec.md §2 の内容）と `runes/generic/rune.toml` を作成する。

`runes/generic/` ディレクトリは Write ツールで直接作成可能（mkdir 不要）。

### Step 4: driver.rs — `v37000_tests::cargo_toml_version_is_37_0_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.1.0` に変更。

### Step 5: driver.rs — `v37100_tests` モジュール追加（T3 完了後）

**前提: Step 3（runes/generic/generic.fav 作成）完了後に実行する。**
`include_str!("../../runes/generic/generic.fav")` はファイル不存在だとコンパイルエラーになるため。

`v37000_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する。

追加内容は spec.md §3 のコードブロックに従う（`use super::*;` を忘れないこと）。

### Step 6: Cargo.toml バージョン更新

Step 2〜5 完了・コンパイルエラー解消後に `37.0.0` → `37.1.0` に更新。

## 依存関係

- `Deserialize` 追加は `middle/checker.rs` の 1 行変更のみ → コンパイルエラーリスク極小
- `runes/generic/generic.fav` はスタブファイル（実行時処理不要）
- **Step 5 は Step 3 完了後** — `include_str!` がコンパイル時にファイル存在を要求するため
- `v37100_tests` は `run()` / `Value` を使うため `use super::*;` が必要

## リスク

| リスク | 対処 |
|---|---|
| `middle/checker.rs` の `type_implements_bound` 行番号がずれている | T0 で grep して実際の行番号を確認してから Edit |
| `runes/generic/` 作成前に Step 5 を実行してコンパイルエラー | Step 3 → Step 5 の順序を守る（tasks.md T3/T5 の順序依存を明示） |
| `deserialize_constraint_type_checks` で `run()` の戻り値型が不一致 | 既存 `bounded_generic_serialize_all_types`（行 31215）と同じパターンで実装 |
