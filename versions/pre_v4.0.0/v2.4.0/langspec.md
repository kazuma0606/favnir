# Favnir v2.4.0 Langspec

更新日: 2026-05-13

## 1. Runtime stack trace

ランタイムエラー時は関数スタックを表示する。

```text
RuntimeError: division by zero
  at divide (main.fav:12)
  at process (main.fav:34)
  at main (main.fav:5)
```

仕様:

- 先頭行は `RuntimeError: <message>`
- 各フレームは `at <fn_name> (<file>:<line>)`
- 行番号は `TrackLine` opcode で更新される最新行を使う

内部表現:

- `CallFrame` は `line: u32` を持つ
- `VMError` は `stack_trace: Vec<TraceFrame>` を持つ
- `TraceFrame` は `fn_name` と `line` を保持する

## 2. Unknown bind warning

`fav check` は `bind name <- expr` の推論結果が `Unknown` のまま残った場合に warning を出す。

```text
warning[W001]: type of `x` could not be resolved (Unknown)
  --> main.fav:5:10
```

ルール:

- warning なので exit code は 0 のまま
- エラーがある場合は従来通り非 0 で終了する
- この warning は plain bind に対して出る

補足:

- 既存の namespace/path 不一致 warning は `W012` に整理した

## 3. Variant / record destructuring parity

`legacy_vm_test_bind_record_destruct` と `legacy_vm_test_bind_variant_destruct` の `#[ignore]` を解除した。

record bind:

```favnir
bind { x, y } <- point
```

variant bind:

```favnir
bind Val(v) <- wrap
```

variant bind は compiler で payload 抽出の `match` に lowering する。

## 4. Compatibility

- `v2.3.0` の record destructuring bind はそのまま有効
- `v2.3.0` の戻り型推論構文もそのまま有効
- 既存の VM 実行結果は維持しつつ、エラー表示だけ詳細化した
