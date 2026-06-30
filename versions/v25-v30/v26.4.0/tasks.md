# v26.4.0 タスクリスト — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.3.0`、テスト数 2062 件、`runes/stream/` が未存在、`ast.rs` に `backpressure` がないこと確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.4.0"` に bump | [x] |
| T2 | `fav/src/ast.rs` 更新: `StreamingAnnotation` に `backpressure: Option<bool>` フィールド追加 + 初期化箇所を全件更新 | [x] |
| T3 | `fav/src/frontend/parser.rs` 更新: `#[streaming(...)]` パース時に `backpressure: true/false` キーを処理 | [x] |
| T4 | `fav/src/backend/vm.rs` 更新: `VMStream` に 4 バリアント（FlatMap / Window / Merge / Split）追加 + 4 primitive 追加 + **`materialize_stream` 関数の match に 4 アーム追加**（`Stream.to_list` が正しく評価できること） | [x] |
| T4.5 | `cargo build` — T2〜T4 のコンパイルエラーなし確認（`non-exhaustive patterns` 含む） | [x] |
| T5 | `runes/stream/stream.fav` 新規作成（6 関数: map / filter / flat_map / window / merge / split） | [x] |
| T6 | `site/content/docs/runes/stream.mdx` 新規作成（`#[streaming]` 全オプション / API リファレンス / ウィンドウ例） | [x] |
| T7 | `CHANGELOG.md` 更新: 先頭に `[v26.4.0]` エントリ追加 | [x] |
| T8 | `benchmarks/v26.4.0.json` 新規作成（test_count: 2070） | [x] |
| T8.5 | `driver.rs` 内の前バージョン `version_is_X` テストが存在する場合は削除（不要なら skip） | [x] |
| T9 | `fav/src/driver.rs` 更新: `v264000_tests`（8 件）を `v263000_tests` の直後に追加 | [x] |
| T9.5 | `cargo test v264000 --bin fav` — 8/8 PASS 確認 | [x] |
| T10 | `cargo test --bin fav` — 2070 件 PASS 確認（リグレッションなし） | [x] |
| T11 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.4.0"` であること
- [x] `fav/src/ast.rs` の `StreamingAnnotation` に `backpressure: Option<bool>` フィールドが存在すること
- [x] `fav/src/frontend/parser.rs` が `backpressure` キーをパースすること
- [x] `fav/src/backend/vm.rs` に `"Stream.flat_map"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Stream.window"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Stream.merge"` primitive が存在すること
- [x] `fav/src/backend/vm.rs` に `"Stream.split"` primitive が存在すること
- [x] `runes/stream/stream.fav` が存在すること
- [x] `runes/stream/stream.fav` に `fn map` が含まれること
- [x] `runes/stream/stream.fav` に `fn filter` が含まれること
- [x] `runes/stream/stream.fav` に `fn flat_map` が含まれること
- [x] `runes/stream/stream.fav` に `fn window` が含まれること
- [x] `runes/stream/stream.fav` に `fn merge` が含まれること
- [x] `runes/stream/stream.fav` に `fn split` が含まれること
- [x] `site/content/docs/runes/stream.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.4.0]` エントリが存在すること
- [x] `benchmarks/v26.4.0.json` が存在すること（test_count: 2070）
- [x] `v264000_tests` 8 件すべて PASS
- [x] 総テスト数 ≥ 2070 件

---

## メモ

### 既存 `Stream.*` primitive の配置（vm.rs ~4491〜4601）

以下は実装済みのため v26.4.0 では変更しない:
- `"Stream.from"` / `"Stream.of"` / `"Stream.gen"` / `"Stream.map"` / `"Stream.filter"` / `"Stream.take"` / `"Stream.to_list"`

新規 4 primitive は `"Stream.to_list"` の直後に追加する。

### `VMStream` 列挙体変更後の網羅性確認

`cargo build` で `non-exhaustive patterns` エラーが出た場合は、
`vm.rs` 内の `match stream_val { VMStream::...` 式を全件確認し、
`FlatMap` / `Window` / `Merge` / `Split` アームを追加する。

### `streaming_annotation_supports_backpressure` テストの include_str! パス

```rust
include_str!("ast.rs")
```
`driver.rs` と `ast.rs` は両方 `fav/src/` 内にある — 同じディレクトリなので `"ast.rs"` で参照する（`"../ast.rs"` は `fav/ast.rs` を指す誤り）。

### `stream.fav` の fn map / fn filter について

`runes/stream/stream.fav` の `fn map` / `fn filter` は vm.rs の既存 primitive を呼ぶラッパー。
`Stream.map` は vm.rs に既に `VMStream::Map` として実装済み。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] `Stream.split` が `VMValue::Stream` を返すと意味論が壊れる（`Stream.map` 等との合成が無効） | primitive を即時実体化に変更 → `VMValue::List([trues_list, falses_list])` を直接返すよう修正 |
| [HIGH] FlatMap/Window/Split に無限ストリームを渡すとスタックオーバーフロー | 既存 Map/Filter と同等リスク、`materialize_stream(Gen)` は既に Err を返す。iterative 化は v27.x |
| [MED] unknown key skip が 1 トークンのみ — 複合値への前方互換性に限界 | 現状の値型（Int/Bool）に限り安全。TODO として記録 |
| [MED] `*s.clone()` の可読性 — `s.as_ref().clone()` に変更 | 修正済み |
| [LOW] `backpressure` コメントを `TODO(v27.x):` 形式にすべき | 記録のみ（v27.x 対応時に統一） |
| [LOW] `stream.fav` 公開関数に型注釈なし | Stream<T> のジェネリクス構文がパーサー未対応のため意図的に省略 |
