# v19.1.0 実装計画 — 遅延評価パイプライン

## 実装順序

```
T1（ast.rs 型追加）              ← 最初
T2（parser.rs パース実装）       ← T1 完了後
T3（波及修正）                   ← T2 完了後（cargo build が通るまで）
T4（codegen.rs opcodes 追加）    ← T3 完了後
T5（vm.rs opcodes 実装）         ← T4 完了後
T6（driver.rs テスト追加）       ← T5 完了後
T7（Cargo.toml）                 ← T6 と並列可
T8（ドキュメント）               ← T7 と並列可
```

---

## T1: ast.rs

追加する型:

```rust
#[derive(Debug, Clone)]
pub struct StreamingAnnotation {
    pub chunk_size: Option<i64>,
    pub span: Span,
}
```

`FlwDef` に追加:
```rust
pub streaming: Option<StreamingAnnotation>,
```

`StageDef` の構造確認（Grep で検索してから追加）:
```rust
pub stateful: bool,
```

---

## T2: parser.rs

### `parse_streaming_annotation`

lookahead: `tokens[pos] == Hash && tokens[pos+1] == LBracket && tokens[pos+2] == Ident("streaming")`

パースパターン:
- `#[streaming]` → `chunk_size: None`
- `#[streaming(chunk_size = N)]` → `chunk_size: Some(N)`

### `parse_stateful_annotation`

lookahead: `#[stateful]` の完全一致チェック

### `parse_item` の修正

`parse_item` の先頭でアノテーションを先読み:
```
api_ann  = parse_api_annotation()?
streaming_ann = parse_streaming_annotation()?
stateful = parse_stateful_annotation()?
```

- `seq` / `abstract seq` の Item に `streaming_ann` を付与
- `stage` の Item に `stateful` を付与

---

## T3: 波及修正

`FlwDef {` を Grep → `streaming: None` を追加（parser.rs 内の構築箇所 + その他）
`StageDef {` を Grep → `stateful: false` を追加

---

## T4: codegen.rs

### opcode 定義（Opcode enum に追加）

```rust
StreamInit  = 0x64,
StreamNext  = 0x65,
StreamMap   = 0x66,
StreamEnd   = 0x67,
```

### `compile_streaming_pipeline` 関数

```
StreamInit
LOOP_START:
  StreamNext(chunk_size)
  StreamEnd → JumpIfTrue LOOP_END
  StreamMap(stage0_idx)
  StreamMap(stage1_idx)
  ...
  StreamMap(stageN_idx)
  // 結果を蓄積バッファに追記
  Jump LOOP_START
LOOP_END:
  // バッファを最終結果として返す
```

実装上の簡略化:
- ソース stage 呼び出し後の戻り値（`Value::List`）に `StreamInit` を適用
- 蓄積は `ListAppendAll` opcode（既存）を使うか、chunk ごとに flatten

### `remap_string_operands` への追加

`StreamNext` / `StreamMap` / `StreamInit` / `StreamEnd` は str_table 非参照。
`remap_string_operands` に no-op ケースとして追加。

---

## T5: vm.rs

### `value.rs` への追加

```rust
Stream(crate::backend::vm::StreamHandle),
```

`StreamHandle` は vm.rs 内に定義:
```rust
pub struct StreamHandle {
    pub items: Vec<Value>,
    pub pos: usize,
}
```

`Value::PartialEq` が derive の場合、`Stream` variant を手動 impl に変更。

### opcode ハンドラ

`StreamInit`:
- stack pop: `Value::List(items)`
- push: `Value::Stream(StreamHandle { items, pos: 0 })`

`StreamNext(n)`:
- stack peek (mut): `Value::Stream` の pos を n 進める
- push: `Value::List(chunk)`

`StreamEnd`:
- stack peek: `Value::Stream` の `pos >= items.len()` を確認
- push: `Value::Bool(is_done)`

`StreamMap(fn_idx)`:
- stack pop: `Value::List(chunk)`
- VM: `fn_idx` を呼び出し chunk を引数に渡す
- push: 戻り値（`Value::List` の想定）

---

## T6: driver.rs

```rust
// v190000_tests に #[ignore] 追加
#[test]
#[ignore]
fn version_is_19_0_0() { ... }

// v191000_tests モジュール追加（5件）
```

---

## 注意点

### `StageDef` の実際の名前を確認

`ast.rs` では `stage` の定義体が `FlwStageDef` や他の名前かもしれない。
実装前に Grep で確認すること。

### ストリーミングの「論理的」実装

v19.1 では以下の制約を設ける:
- ソース stage は `List<T>` を一度に全部返す（ファイル行単位の読み込みは v19.5 以降）
- `StreamInit` は `List<T>` をメモリに持ったまま chunk 単位でスライス
- 「論理的ストリーミング」として、chunk 単位の処理パターンを確立する

### `compile_streaming_pipeline` の結果蓄積

各 chunk の `StreamMap` 結果を蓄積するには、既存の `ListAppend` primitive か、
新しい蓄積変数（ローカルスロット）を使う。
`ListConcat` opcode（既存確認が必要）を使って chunk 結果を連結するのが最もシンプル。
