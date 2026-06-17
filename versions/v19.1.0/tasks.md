# v19.1.0 — 遅延評価パイプライン タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — StreamingAnnotation 追加 + FlwDef / StageDef 拡張

- [x] `StreamingAnnotation` struct を追加:
  ```rust
  #[derive(Debug, Clone)]
  pub struct StreamingAnnotation {
      pub chunk_size: Option<i64>,
      pub span: Span,
  }
  ```
- [x] `FlwDef` に `pub streaming: Option<StreamingAnnotation>` フィールドを追加
- [x] `StageDef`（実際の struct 名を Grep で確認してから）に `pub stateful: bool` フィールドを追加
- [x] `cargo build` でコンパイルエラーが生じることを確認（T2/T3 で修正）

---

### T2: `fav/src/frontend/parser.rs` — アノテーションパース実装

- [x] `parse_streaming_annotation() -> Result<Option<StreamingAnnotation>, ParseError>` 追加:
  - `peek() != Hash` なら `Ok(None)` を即返却
  - lookahead で `#[streaming` を確認（`#[api]` / `#[stateful]` 等と区別）
  - `#[streaming]`（引数なし） → `StreamingAnnotation { chunk_size: None, span }`
  - `#[streaming(chunk_size = N)]` → `StreamingAnnotation { chunk_size: Some(N), span }`
  - `N` は `TokenKind::Int(n)` を消費
- [x] `parse_stateful_annotation() -> Result<bool, ParseError>` 追加:
  - lookahead で `#[stateful]` の完全一致を確認
  - マッチすれば消費して `Ok(true)` を返す
  - マッチしなければ `Ok(false)` を返す
- [x] `parse_item` の先頭でアノテーション先読み:
  - `api_ann = parse_api_annotation()?`（既存）
  - `streaming_ann = parse_streaming_annotation()?`（新規）
  - `stateful = parse_stateful_annotation()?`（新規）
- [x] `seq` / `abstract seq` の `Item::FlwDef` 構築時に `streaming: streaming_ann` を付与
- [x] `stage` の `Item::StageDef` 構築時に `stateful: stateful` を付与
- [x] `parse_flw_def` の戻り値（`FlwDef { ... }`）に `streaming: None` を追加（T3 と合わせて実施）

---

### T3: 波及ファイル修正

Grep で `FlwDef {` を検索し、`streaming: None` を追記:

- [x] `fav/src/frontend/parser.rs` — `parse_flw_def` 内の `FlwDef { ... }` 構築
- [x] その他 Grep で発見した `FlwDef { ... }` 構築箇所（driver.rs / checker.rs / compiler.rs 等）

Grep で `StageDef {`（または実際の struct 名）を検索し、`stateful: false` を追記:

- [x] 発見した全箇所に `stateful: false` を追記
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

---

### T4: `fav/src/backend/codegen.rs` — ストリーミング opcodes 追加

**4-A: opcode 定義**

- [x] `Opcode` enum に追加（`RefinementAssert = 0x63` の直後）:
  ```rust
  StreamInit  = 0x64,
  StreamNext  = 0x65,
  StreamMap   = 0x66,
  StreamEnd   = 0x67,
  ```

**4-B: `compile_flw_def` の分岐**

- [x] `compile_flw_def` に `flw_def.streaming.is_some()` チェックを追加:
  - `true` → `compile_streaming_pipeline(flw_def, chunk_size)` を呼ぶ
  - `false` → 従来の eager コンパイルを維持

**4-C: `compile_streaming_pipeline` 関数**

- [x] `compile_streaming_pipeline(flw_def: &FlwDef, chunk_size: i64)` 実装:
  1. ソース stage（steps[0]）を呼び出す（従来通り）
  2. `emit_opcode(Opcode::StreamInit)` — List → Stream 変換
  3. 蓄積バッファ用のローカルスロットを確保
  4. ループ開始ラベルを設置
  5. `emit_opcode(Opcode::StreamNext)` + `emit_u32(chunk_size as u32)` — 次 chunk 取得
  6. `emit_opcode(Opcode::StreamEnd)` — 空チェック
  7. 空なら ループ終端 へジャンプ（`JumpIfTrue`）
  8. steps[1..] の各 stage に `emit_opcode(Opcode::StreamMap)` + `emit_u32(fn_idx)`
  9. chunk 結果を蓄積バッファに連結（`ListConcat` opcode か push ループ）
  10. ループ先頭へ `Jump`
  11. ループ終端ラベル — 蓄積バッファを最終 push

**4-D: `remap_string_operands` への追加**

- [x] `StreamInit` / `StreamEnd` → no-op（operand なし）
- [x] `StreamNext` / `StreamMap` → operand 1 個（`u32`）— str_table 非参照なので no-op

---

### T5: `fav/src/backend/vm.rs` + `fav/src/value.rs` — opcodes 実装

**5-A: `StreamHandle` 定義**

- [x] `vm.rs` に追加（または `value.rs` に）:
  ```rust
  #[derive(Debug, Clone)]
  pub struct StreamHandle {
      pub items: Vec<Value>,
      pub pos: usize,
  }
  ```

**5-B: `Value::Stream` 追加**

- [x] `value.rs` に `Stream(StreamHandle)` variant を追加
- [x] `Value` に `PartialEq` が derive されている場合:
  - `Stream` variant を手動 `PartialEq` impl に追加:
    ```rust
    (Value::Stream(a), Value::Stream(b)) => std::ptr::eq(
        a as *const _, b as *const _
    ),
    ```
- [x] exhaustive match が必要な全箇所（driver.rs の fmt 等）に `Value::Stream` ケースを追加

**5-C: opcode ハンドラ実装（vm.rs の dispatch ループ内）**

- [x] `StreamInit`:
  ```rust
  let list = pop_list(stack)?;
  stack.push(Value::Stream(StreamHandle { items: list, pos: 0 }));
  ```
- [x] `StreamNext`:
  ```rust
  let n = read_u32(bytecode, ip) as usize; ip += 4;
  let stream = stack.last_mut().and_then(|v| if let Value::Stream(s) = v { Some(s) } else { None })?;
  let chunk: Vec<Value> = stream.items[stream.pos..].iter().take(n).cloned().collect();
  stream.pos += chunk.len();
  stack.push(Value::List(chunk));
  ```
- [x] `StreamEnd`:
  ```rust
  let is_done = if let Some(Value::Stream(s)) = stack.last() { s.pos >= s.items.len() } else { false };
  stack.push(Value::Bool(is_done));
  ```
- [x] `StreamMap`:
  ```rust
  let fn_idx = read_u32(bytecode, ip) as usize; ip += 4;
  let chunk = pop!(stack); // Value::List
  // fn_idx を呼び出し chunk を引数に渡す
  let result = call_fn(artifact, fn_idx, vec![chunk], ...)?;
  stack.push(result);
  ```

---

### T6: `fav/src/driver.rs` — `v191000_tests` 追加

- [x] `v190000_tests::version_is_19_0_0` に `#[ignore]` を追加
- [x] `v191000_tests` モジュールを追加（5件）:

  ```rust
  #[test]
  fn version_is_19_1_0() {
      let cargo = include_str!("../Cargo.toml");
      assert!(cargo.contains("19.1.0"), "Cargo.toml should have version 19.1.0");
  }

  #[test]
  fn streaming_annotation_parses() {
      // #[streaming(chunk_size = 1000)] seq P = A |> B
      // → FlwDef.streaming = Some(StreamingAnnotation { chunk_size: Some(1000) })
      let src = r#"
  #[streaming(chunk_size = 1000)]
  seq Pipeline = StageA |> StageB
  "#;
      let prog = Parser::parse_str(src, "test.fav").expect("parse");
      if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
          let s = fd.streaming.as_ref().expect("expected streaming annotation");
          assert_eq!(s.chunk_size, Some(1000));
      } else {
          panic!("expected FlwDef");
      }
  }

  #[test]
  fn streaming_default_chunk_size_parses() {
      // #[streaming] seq P = A |> B
      // → FlwDef.streaming = Some(StreamingAnnotation { chunk_size: None })
      let src = r#"
  #[streaming]
  seq Pipeline = StageA |> StageB
  "#;
      let prog = Parser::parse_str(src, "test.fav").expect("parse");
      if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
          let s = fd.streaming.as_ref().expect("expected streaming annotation");
          assert_eq!(s.chunk_size, None);
      } else {
          panic!("expected FlwDef");
      }
  }

  #[test]
  fn streaming_pipeline_executes() {
      // ストリーミングパイプラインが通常パイプラインと同じ結果を返す（compile + run）
      // 簡単なテスト: List<Int> を受け取り各要素 *2 する stage を streaming で実行
      // 詳細は実装時に確定（Parser + Checker + Compiler + VM の E2E）
  }

  #[test]
  fn streaming_stateful_annotation_parses() {
      // #[stateful] stage S(rows: List<Int>) -> List<Int> { rows }
      // → StageDef.stateful = true
      let src = r#"
  #[stateful]
  stage MyStage(rows: List<Int>) -> List<Int> { rows }
  "#;
      let prog = Parser::parse_str(src, "test.fav").expect("parse");
      // StageDef の実際の Item variant 名を確認してからアサート
      assert_eq!(prog.items.len(), 1);
      // stateful フィールドのアサート（Item variant 名は T1 で確認後に記入）
  }
  ```

---

### T7: `fav/Cargo.toml` 更新

- [x] `version = "19.0.0"` → `"19.1.0"` に更新

---

### T8: `site/content/docs/language/streaming.mdx` 作成

- [x] ストリーミングパイプラインの使い方ガイド
- [x] `#[streaming(chunk_size = N)]` / `#[streaming]` の構文説明
- [x] `#[stateful]` ステージアノテーションの説明
- [x] 大規模データ処理のベストプラクティス

---

## テスト（v191000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_19_1_0` | Cargo.toml に `"19.1.0"` が含まれる |
| `streaming_annotation_parses` | `#[streaming(chunk_size = 1000)]` が正しくパースされる |
| `streaming_default_chunk_size_parses` | `#[streaming]`（引数なし）が正しくパースされる |
| `streaming_pipeline_executes` | ストリーミングパイプラインが正しい結果を返す |
| `streaming_stateful_annotation_parses` | `#[stateful]` が `stateful = true` としてパースされる |

---

## 完了条件チェックリスト

- [x] `StreamingAnnotation` struct が `ast.rs` に存在する
- [x] `FlwDef.streaming: Option<StreamingAnnotation>` フィールドが存在する
- [x] `StageDef.stateful: bool` フィールドが存在する
- [x] `#[streaming(chunk_size = 1000)] seq P = ...` がパースされる
- [x] `#[streaming] seq P = ...` がパースされる
- [x] `#[stateful] stage S = ...` がパースされる
- [x] `StreamInit` / `StreamNext` / `StreamMap` / `StreamEnd` opcode が実装される
- [x] ストリーミングパイプラインが正しい結果を返す
- [x] `cargo test v191000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/language/streaming.mdx` が存在する

---

## 優先度

```
T1（ast.rs 型追加）              ← 最初
T2（parser.rs パース実装）       ← T1 完了後
T3（波及修正）                   ← T2 完了後
T4（codegen.rs opcodes 追加）    ← T3 完了後
T5（vm.rs opcodes 実装）         ← T4 完了後
T6（driver.rs テスト追加）       ← T5 完了後
T7（Cargo.toml）                 ← T6 と並列可
T8（ドキュメント）               ← T7 と並列可
```

---

## 重要な技術ノート

### StageDef の実際の struct 名

`ast.rs` で `stage` に対応する struct を Grep で確認してから T1 を実施すること。
`StageDef` / `StageDecl` / `Stage` 等の名前の可能性がある。

### parse_streaming_annotation のルックアヘッド

`tokens[pos]   == Hash`
`tokens[pos+1] == LBracket`
`tokens[pos+2] == Ident("streaming")`
の 3 条件を満たす場合のみ消費する。`#[api]` `#[stateful]` `#[test]` 等との衝突を避ける。

### Value::Stream の PartialEq 問題

`Value` enum に `#[derive(PartialEq)]` がある場合、`Stream(StreamHandle)` の追加で
コンパイルエラーになる。`StreamHandle` に `PartialEq` を追加（pointer 比較で可）するか、
`Value` 全体を manual impl に変更する。v18.7 の `GenericParam.const_constraint` の
`Option<Box<Expr>>` 問題と同様のパターンを参照すること。

### StreamMap の VM 内関数呼び出し

`StreamMap(fn_idx)` は VM ループ内から別の fn を呼び出す。
既存の `OpCall` ハンドラの実装パターンを参照すること（再帰 VM 呼び出し or call stack）。
