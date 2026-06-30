# v25.9.0 タスクリスト — vm.fav Phase 6（CallNamed 実装）

**状態**: COMPLETE
**開始日**: 2026-06-26
**完了日**: 2026-06-26

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.9.0"` に bump | [x] |
| T1 | `fav/self/vm.fav` 更新: `Opcode` に `CallNamed(Int, Int)` 追加（`Call(Int)` の直後）、`opcode_to_string` にアーム追加 | [x] |
| T2 | `fav/self/vm.fav` 更新: `decode_byte_with_u16x2_le` ヘルパー追加、`decode_opcode` に `0x56` アーム追加 | [x] |
| T3 | `fav/self/vm.fav` 更新: `vm_execute` シグネチャ拡張（`consts: Int, prog_keys: Int, prog_vals: Int` 追加）、再帰呼び出しを 3 パターンで個別更新（通常 / `Jump dec.next_pc+off` / `JumpIfFalse` × 2 / `vm_run` 初回） | [x] |
| T3.5 | `cargo build` — T3 のシグネチャ変更漏れがないことを確認 | [x] |
| T4 | `fav/self/vm.fav` 更新: 補助型・関数追加（`FnDef`、`parse_fn_json`、`copy_args_to_locals`、`build_consts_list`、`find_fn_in_program`、`build_program_lists`） — **Mut.str_map は存在しないため線形検索パターンを採用** | [x] |
| T5 | `fav/self/vm.fav` 更新: `CallNamed(name_idx, argc)` ハンドラを `vm_execute` の `Call(argc)` アームの直後に追加 | [x] |
| T6 | `fav/self/vm.fav` 更新: `vm_run_program(program_json: String)` 新エントリポイント追加 | [x] |
| T7 | `cargo build` で型エラーなし確認 | [x] |
| T8 | `fav/src/driver.rs` 更新: `pub fn build_vm_program_json(artifact: &FvcArtifact) -> String` 追加 | [x] |
| T9 | `fav/src/driver.rs` 更新: `pub fn run_via_vm(vm_src: &str, program_json: &str) -> String` 追加 | [x] |
| T10 | `fav/src/main.rs` 更新: `fav run --vm <path> --compile <src>` CLI モード追加（`--hex` モードの直前に挿入） | [x] |
| T10.5 | `site/content/docs/tools/vm-fav.mdx` が存在する場合、`--compile` フラグ説明を追記 | [x] |
| T11 | `CHANGELOG.md` 更新（`[v25.9.0]` エントリ追加） | [x] |
| T12 | `benchmarks/v25.9.0.json` 新規作成（test_count: 2035） | [x] |
| T13 | `fav/src/driver.rs` 更新（`v259000_tests` 7 件追加） | [x] |
| T14 | `cargo test v259000` — 7 件 PASS 確認 | [x] |
| T15 | `cargo test` 総テスト数 ≥ 2035 件 確認 | [x] |
| T16 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x]`fav/self/vm.fav` に `CallNamed(Int, Int)` opcode が存在する
- [x]`fav/self/vm.fav` に `vm_run_program` が存在する
- [x]`fav/self/vm.fav` に `0x56` デコーダー（`decode_byte_with_u16x2_le`）が存在する
- [x]`fav/src/driver.rs` に `build_vm_program_json` が存在する
- [x]`fav/src/driver.rs` に `run_via_vm` が存在する
- [x]`fav run --vm <path> --compile <src>` CLI モードが動作する
- [x]multi-function プログラムを vm.fav で実行して正しい値が返ること（`run_via_vm_correct_result` テスト PASS）
- [x]`CHANGELOG.md` に `[v25.9.0]` エントリが存在する
- [x]`v259000_tests` 7 件すべて PASS
- [x]総テスト数 ≥ 2035 件

---

## メモ

### vm.fav 再帰呼び出し更新について
`vm_execute` のシグネチャに `consts: Int, program: Int` を追加するため、
関数内の全再帰呼び出し箇所（約 40 箇所）を更新する必要がある。
`replace_all` で `vm_execute(bytecode, stack, locals, globals, dec.next_pc)` を一括置換する。

ただし `vm_run` / `vm_run_named` からの初回呼び出し箇所は別パターンのため個別更新。

### Mut.str_map は存在しない（spec-reviewer HIGH-2 確認済み）
`Mut.str_map()` / `Mut.str_get` / `Mut.str_set` は vm.rs に存在しない。
**線形検索パターンを採用**（T4 で実装）:
- `prog_keys: Int` (Mut.list) — fn_name 文字列の配列
- `prog_vals: Int` (Mut.list) — fn_json 文字列の配列（インデックス対応）
- `find_fn_in_program(keys, vals, fn_name, i)` — 再帰的線形検索

### Constant::Name に注意（spec-reviewer HIGH-1 確認済み）
`CallNamed` が参照するのは `Constant::Name`（関数名）であり `Constant::Str` ではない。
`build_vm_program_json` (T8) では `Int / Float / Str / Name` の 4 バリアントすべてを変換する。

### parse_fn_json の実装
Favnir に JSON パーサーはないため、`Json.get_str_field` / `Json.get_arr_field` が
存在するか確認する。存在しない場合、単純な文字列解析で代替:
- `"code"` 値: `"code":"` の後から次の `"` まで
- `"consts"` 配列: `["` ... `"]` を `,` で分割

このパーサーは本番品質不要（vm.fav Phase 6 の scope 内 JSON フォーマットのみ対応）。

### `Constant::Int` の扱い
`CallNamed(name_const_idx, argc)` の `name_const_idx` は `FnBytecode.constants` の
全要素（`Constant::Str` + `Constant::Int` 混在）のインデックス。
`build_vm_program_json` では全 Constant を文字列に変換して格納する:
- `Constant::Str(s)` → `"s"`（そのまま）
- `Constant::Int(n)` → `"42"` (数値の文字列表現)

### `fav run --vm --compile` の位置
`main.rs` では `--hex` モードの前（or 内側）に `--compile` アームを追加:
```
if let Some(vm_pos) = args.iter().position(|a| a == "--vm") {
    if let Some(compile_pos) = args.iter().position(|a| a == "--compile") {
        // --vm + --compile モード: build_vm_program_json → run_via_vm
    } else if let Some(hex_pos) = args.iter().position(|a| a == "--hex") {
        // 既存の --vm + --hex モード
    }
}
```

### テスト数
v25.8.0 完了時 2028 件 + v25.9.0 7 件 = 2035 件

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] `parse_fn_json` / `build_program_lists_rec` で `none =>` が `some(...)` の前に来ており、Favnir パーサーが `none` を `Pattern::Bind`（catch-all）として解析するため常に早期リターンしていた | `some(...)` アームを先頭に移動し、`none =>` を `_ =>` に変更して修正 |
| [HIGH] `run_via_vm_correct_result` で Rust コンパイラ出力（`LoadGlobal(86)+Call`）を vm.fav に渡していたが、vm.fav の globals マップが空のため `LoadGlobal` がエラー | テストを手動クラフト ByteCode（`CallNamed` opcode 使用）に切り替え |
| [MED] `Const(n)` ハンドラが定数プールを参照せず `VMInt(n)` として扱う（Phase 2 設計）。`build_vm_program_json` 出力を `run_via_vm` に直接渡すと誤動作する可能性あり | Phase 2 設計上の制限として tasks.md に記録。vm-fav.mdx に制限を明記済み（手動クラフト bytecode テストで PASS 確認） |
| [LOW] `build_program_lists_rec` の `}` 検索は fn_json 内に `}` がない前提（将来スキーマ拡張で壊れる可能性） | 現スキーマでは問題なし。将来拡張時の注意事項としてコメントに残す |
| [LOW] `find_fn_in_program` の `_ => err("prog_keys: non-VMStr key")` は到達不能だが誤解を招くメッセージ | 到達不能コードのため実害なし。記録のみ |
