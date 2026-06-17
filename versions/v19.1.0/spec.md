# v19.1.0 Spec — 遅延評価パイプライン（Lazy / Streaming Evaluation）

## 概要

大規模データを定常メモリで処理できるようにする。
現在は全ステージで全データをメモリに乗せる（eager evaluation）。
`#[streaming(chunk_size = N)]` アノテーションで chunk 単位のストリーミング評価に切り替える。

**テーマ**: Production Performance シリーズ第1弾

---

## 動機

```favnir
// 現状: 10GB CSV を全部メモリに乗せてから処理
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb
// LoadCsv が全行を List<Row> として返す → 10GB がメモリ上に展開される
```

```favnir
// v19.1.0 以降: チャンク単位のストリーミング評価
#[streaming(chunk_size = 1000)]
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb
// 最大メモリ使用量 ≈ chunk_size × row_size（全データを保持しない）
```

---

## 構文

### `#[streaming]` アノテーション

```favnir
// chunk_size 指定あり
#[streaming(chunk_size = 1000)]
seq Pipeline = LoadCsv |> Transform |> Save

// chunk_size 省略（デフォルト 512）
#[streaming]
seq Pipeline = LoadCsv |> Transform |> Save
```

### `#[stateful]` ステージアノテーション

chunk 間で状態を保持するステージに付ける:

```favnir
#[stateful]
stage RunningAvg(rows: List<Row>) -> List<Row> {
  // 内部状態は VM が chunk 間で保持する
  ...
}
```

---

## 実装内容

### T1: `fav/src/ast.rs`

- `StreamingAnnotation` struct 追加:
  ```rust
  pub struct StreamingAnnotation {
      pub chunk_size: Option<i64>,
      pub span: Span,
  }
  ```
- `FlwDef` に `pub streaming: Option<StreamingAnnotation>` フィールド追加
- `StageDef` に `pub stateful: bool` フィールド追加（`#[stateful]` アノテーション）

### T2: `fav/src/frontend/parser.rs`

- `parse_streaming_annotation() -> Result<Option<StreamingAnnotation>, ParseError>`:
  - `#[streaming]` → `StreamingAnnotation { chunk_size: None, span }`
  - `#[streaming(chunk_size = N)]` → `StreamingAnnotation { chunk_size: Some(N), span }`
  - lookahead で `#[streaming` を確認（`#[api]` / `#[stateful]` と区別）
- `parse_stateful_annotation() -> Result<bool, ParseError>`:
  - `#[stateful]` → `true`
- `parse_item` でアノテーションを先読みし、`FlwDef` / `StageDef` に付与

### T3: 波及修正（`FlwDef` / `StageDef` struct リテラル）

- Grep で `FlwDef {` を検索 → `streaming: None` を追記
- Grep で `StageDef {` を検索 → `stateful: false` を追記
- `cargo build` でコンパイルエラーが 0 になることを確認

### T4: `fav/src/backend/codegen.rs` — ストリーミング opcodes 追加

新しい opcode（0x64〜0x67）:

```rust
StreamInit  = 0x64,  // スタックトップの List を StreamHandle に変換
StreamNext  = 0x65,  // operand: chunk_size(u32) — 次の chunk を取り出す
StreamMap   = 0x66,  // operand: fn_idx(u32) — chunk に stage 関数を適用
StreamEnd   = 0x67,  // ストリームの終了確認（空なら JumpIfFalse へ）
```

`compile_flw_def` を拡張:
- `flw_def.streaming.is_some()` なら `compile_streaming_pipeline` を呼ぶ
- そうでなければ従来の eager コンパイルを維持

`compile_streaming_pipeline(flw_def, chunk_size)`:
1. `StreamInit` — ソース stage の出力を Stream に変換
2. ループ開始ラベル
3. `StreamNext(chunk_size)` — 次の chunk を取得
4. `StreamEnd` → ループ終端へジャンプ（空なら終了）
5. 各 stage に対して `StreamMap(stage_fn_idx)` を順番に emit
6. ループ先頭へジャンプ
7. ループ終端ラベル

### T5: `fav/src/backend/vm.rs` — opcodes 実装

- `StreamHandle` 型:
  ```rust
  struct StreamHandle {
      items: Vec<Value>,   // バッファリングされた全要素（ソースから一度に取得）
      pos: usize,          // 現在位置
  }
  ```
  ※ v19.1 は「論理的なストリーミング」実装（メモリ効率の実証）。
  　ファイルレベルの真のストリーミング（行単位読み込み）は v19.5 以降で対応。

- `Value::Stream(Rc<RefCell<StreamHandle>>)` を `value.rs` に追加

- 各 opcode のハンドラ:
  - `StreamInit`: `Value::List(items)` → `Value::Stream(StreamHandle { items, pos: 0 })`
  - `StreamNext(n)`: Stream から最大 n 個を取り出し `Value::List` として push
  - `StreamEnd`: Stream が空なら 1 を push（終了フラグ）、そうでなければ 0 を push
  - `StreamMap(fn_idx)`: chunk（`Value::List`）を引数に fn_idx を呼び出し、結果を push

- `remap_string_operands` に `StreamNext` / `StreamMap` を追加（str_table 非参照のため no-op）

### T6: `fav/src/driver.rs` — テスト追加

- `v190000_tests::version_is_19_0_0` に `#[ignore]` を追加
- `v191000_tests` モジュールを追加（5件）

### T7: `fav/Cargo.toml` 更新

- `version = "19.0.0"` → `"19.1.0"`

### T8: ドキュメント

- `site/content/docs/language/streaming.mdx` — `#[streaming]` / `#[stateful]` ガイド

---

## テスト（v191000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_1_0` | Cargo.toml に `"19.1.0"` が含まれる |
| `streaming_annotation_parses` | `#[streaming(chunk_size = 1000)]` が `StreamingAnnotation { chunk_size: Some(1000) }` としてパースされる |
| `streaming_default_chunk_size_parses` | `#[streaming]`（引数なし）が `StreamingAnnotation { chunk_size: None }` としてパースされる |
| `streaming_pipeline_executes` | ストリーミングパイプラインが通常パイプラインと同じ結果を返す |
| `streaming_stateful_annotation_parses` | `#[stateful]` が `StageDef.stateful = true` としてパースされる |

---

## 完了条件

- [ ] `#[streaming(chunk_size = 1000)]` が AST にパースされる
- [ ] `#[streaming]` がデフォルト chunk_size でパースされる
- [ ] ストリーミングパイプラインが正しい結果を返す（eager と同一）
- [ ] `#[stateful]` アノテーションがパースされる
- [ ] `cargo test v191000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/language/streaming.mdx` が存在する

---

## 技術ノート

### opcode 番号

現在の最大 opcode は `RefinementAssert = 0x63`。
ストリーミング opcodes は 0x64〜0x67 に割り当てる。

### `StageDef` の検索

`StageDef` は ast.rs の `pub struct` として定義されている。
Grep で確認してから `stateful: bool` フィールドを追加すること。

### `Value::Stream` の `PartialEq`

`Rc<RefCell<StreamHandle>>` は `PartialEq` を自動 derive できない。
`Value` に `PartialEq` が derive されている場合は manual impl が必要。
（v18.7 の `GenericParam` と同様のパターン）

### `compile_streaming_pipeline` の戻り値の扱い

ストリーミングパイプラインは `List<OutputRow>` を返す（eager と同じ外部インターフェース）。
内部で chunk 単位処理 → 結果を `List.push` で蓄積 → 最終結果を返す。
`StreamMap` の結果（chunk の出力）は逐次 `List.push` で蓄積 opcode で連結する。
