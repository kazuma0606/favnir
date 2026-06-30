# v25.9.0 仕様書 — vm.fav Phase 6（CallNamed 実装）

## テーマ

`vm.fav` にユーザー定義関数呼び出し (`CallNamed` opcode 0x56) を実装し、
multi-function プログラムを vm.fav インタープリター上で実行できるようにする。

「Favnir で書いた VM が Favnir プログラム（関数呼び出しあり）を完全に実行できる」最後のピース。

---

## 背景と現状

`fav/self/vm.fav` は Phase 1〜5 完了:
- Phase 1: opcode デコード（27 件）
- Phase 2: スタックベース実行ループ（算術・比較・Jump 等）
- Phase 3: LoadLocal / StoreLocal / LoadGlobal
- Phase 4: call_builtin（組み込み関数ディスパッチ: `Int.to_string` / `String.length` 等）
- Phase 5: `collect_args` ヘルパー

**未実装**: `CallNamed` (0x56) — ユーザー定義関数の呼び出し。
現在の `Call(argc)` は builtin のみ対応（callee = `VMStr(name)` → `call_builtin`）。

---

## CallNamed opcode の仕様

```
opcode: 0x56 (CallNamed)
size:   5 バイト (1-byte opcode + 2-byte name_const_idx LE + 2-byte argc LE)
動作:
  1. 現在関数の constants[name_const_idx] → 呼び出すユーザー定義関数名 (String)
  2. program テーブルから対象関数のバイトコードと定数リストを取得
  3. スタックから argc 個の引数を収集
  4. 対象関数を fresh locals で vm_execute 再帰呼び出し
  5. 結果をスタックに push
```

これは自己ホスト型コンパイラ (`compiler.fav`) が出力するバイトコードで使用される。

---

## プログラム JSON フォーマット

vm.fav は multi-function プログラムを JSON 文字列として受け取る:

```json
{
  "main":   {"code": "01000012...", "consts": ["helper_fn", "String.length"]},
  "helper_fn": {"code": "10000016", "consts": []}
}
```

- `"code"`: バイトコードの hex 文字列
- `"consts"`: 関数内の定数プール（全 `Constant` バリアントを文字列化した配列）
  - `Constant::Int(n)` → `"42"`、`Constant::Float(f)` → `"3.14"`、`Constant::Str(s)` → `"s"`、`Constant::Name(s)` → `"s"`
  - `CallNamed(name_idx, argc)` の `name_idx` は `Constant::Name(fn_name)` のインデックスを指す
  - **注意**: `Constant::Name` は `Constant::Str` とは別のバリアント。`codegen.rs` に `Int / Float / Str / Name` の 4 バリアントが存在する

エントリポイントはキー `"main"` の関数。

---

## 変更対象

### `fav/self/vm.fav`

1. **`Opcode` に `CallNamed(Int, Int)` 追加**（fn_name_const_idx, argc）
2. **5バイトデコーダー** `decode_byte_with_u16x2_le`: 0x56 → `CallNamed(name_idx, argc)`
3. **`vm_execute` シグネチャ拡張**:
   ```favnir
   fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, globals: Int,
                 consts: Int, program: Int, pc: Int) -> Result<VMVal, String>
   ```
   - `consts: Int` — 現在関数の定数プール (`Mut.list` ハンドル、String の配列)
   - `program: Int` — 全関数テーブル (`Mut.map` ハンドル、fn_name → JSON 文字列)
4. **`CallNamed(name_idx, argc)` ハンドラ**
5. **新エントリポイント**:
   - `vm_run_program(program_json: String) -> Result<VMVal, String>`
   - JSON をパース → program マップ構築 → `"main"` 関数を実行

### `fav/src/driver.rs`

6. **`pub fn build_vm_program_json(artifact: &FvcArtifact) -> String`**
   — FvcArtifact の全関数を program JSON 形式にシリアライズ
7. **`pub fn run_via_vm(vm_src: &str, program_json: &str) -> String`**
   — vm.fav 経由でプログラムを実行
8. **`v259000_tests`** 7 件追加

### `fav/src/main.rs`

9. **`fav run --vm <vm_path> --compile <src_path>` CLI モード追加**
   — ソースをコンパイルして build_vm_program_json → run_via_vm

---

## NOT in scope (v26.x 予定)

- クロージャのキャプチャ (close-over)
- `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` の完全動作 E2E（bootstrap テスト #[ignore] は残す）
- 末尾再帰最適化（TCO）
- `Json.get_arr_field` 等の完全 JSON パーサーは未実装のため、`parse_fn_json` は関数名にカンマ・ダブルクォートを含まない前提の単純文字列解析で実装する（compiler.fav 出力の関数名はアルファベット・ドット・アンダースコアのみ）

---

## ロードマップ位置付け

v25.9.0 は `versions/roadmap/roadmap-v25.1-v26.0.md` の `v25.9` エントリ（vm.fav Phase 6）に対応。
v25.0.0「Practical Self-Hosting」完了後の追加フェーズとして位置付けられ、v26.0.0（Rune Foundation マイルストーン）の前提となる。

---

## テスト数

- v25.8.0 完了時: 2028 件
- v25.9.0 追加: 7 件
- **目標**: ≥ 2035 件
