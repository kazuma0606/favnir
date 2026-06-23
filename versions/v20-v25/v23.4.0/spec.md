# v23.4.0 仕様書 — vm.fav Phase 1（バイトコードデコード）

## 概要

`vm.fav` を書き始める最初のフェーズ。`fav/self/vm.fav` を新規作成し、
Favnir バイトコードの **Opcode 型定義** と **decode_opcode 関数** を実装する。

実行ループは Phase 2（v23.5）以降のスコープ。このフェーズでは「デコードが正しく動く」ことを証明する。

---

## 背景

`codegen.rs` が生成するバイトコード形式:
- 各命令は **1 バイトのオペコード** + **可変長オペランド** で構成
- オペランドは Little-Endian `u16`（2 バイト）が基本
- 例: `Const(0x01)` + `u16` const_idx、`Return(0x16)` はオペランドなし

`Bytes.read_u16` / `Bytes.read_u24` は v23.1.0 で実装済み。

---

## 既存リソース確認

| リソース | 場所 | ステータス |
|---|---|---|
| `Opcode` 定義 | `fav/src/backend/codegen.rs` | Rust 実装済み |
| `Bytes.read_u16/read_u24/read_u32` | `vm.rs` vm_call_builtin | 実装済み（**ビッグエンディアン**） |
| `Bytes.get(bytes, offset)` | vm.rs | 実装済み（`Result<Int, String>` を返す） |
| `Bytes.from_hex(hex_str)` | vm.rs | 実装済み（`Result<Bytes, String>` を返す） |
| `fav/self/compiler.fav` | fav/self/ | 既存 — vm.fav はここに追加 |

### 重要: エンディアン問題

`codegen.rs` は `emit_u16(value.to_le_bytes())` で **リトルエンディアン（LE）** でバイトコードを生成する。
しかし `Bytes.read_u16` は **ビッグエンディアン（BE）** で実装されており（既存テスト `bytes_read_u16_big_endian` で確認済み）、`Bytes.read_u16` の変更は既存テストを壊す。

**対応**: `Bytes.read_u16_le` / `Bytes.read_u24_le` を v23.4.0 で新規追加し、vm.fav はこれを使用する。

### ロードマップからの変更点

| 項目 | ロードマップ | v23.4.0 実装 | 理由 |
|---|---|---|---|
| 戻り値型 | `-> Opcode` | `-> Result<DecodeResult, String>` | `Bytes.get` が `Result` を返すため |
| バリアント名 | `LoadConst(Int)` | `Const(Int)` | codegen.rs の `Opcode::Const = 0x01` に合わせる |
| read 関数 | `read_u24` | `read_u16_le` | codegen.rs は u16 LE でオペランドをエンコード |

---

## 実装スコープ

### T0: `Bytes.read_u16_le` / `Bytes.read_u24_le` 新規追加（vm.rs + checker.rs）

| 関数 | 動作 | 用途 |
|---|---|---|
| `Bytes.read_u16_le(bytes, off)` | `[lo, hi]` を LE u16 として読む → `Result<Int, String>` | codegen.rs バイトコードのオペランド読み取り |
| `Bytes.read_u24_le(bytes, off)` | `[b0, b1, b2]` を LE u24 として読む → `Result<Int, String>` | 将来の 3 バイトオペランド用（Phase 1 では不要だが一緒に追加） |

checker.rs: `Bytes` namespace が `Type::Unknown` なので追加不要（namespace 登録のみで全メソッドが通過）。
vm.rs: vm_call_builtin に 2 ハンドラを追加（`"Bytes"` アーム内、既存 `read_u32` の直後）。

### `fav/self/vm.fav` — 新規ファイル

#### 1. `Opcode` 型定義（Phase 1 対象: 27 バリアント）

codegen.rs の `Opcode` enum（`#[repr(u8)]`）から Phase 1 で扱う主要 opcode を Favnir 型として定義する。

```favnir
type Opcode =
  // 定数ロード
  | Const(Int)         // 0x01: +u16 const_idx
  | ConstUnit          // 0x02: オペランドなし
  | ConstTrue          // 0x03: オペランドなし
  | ConstFalse         // 0x04: オペランドなし
  // ローカル・グローバル
  | LoadLocal(Int)     // 0x10: +u16 slot
  | StoreLocal(Int)    // 0x11: +u16 slot
  | LoadGlobal(Int)    // 0x12: +u16 global_idx
  | Pop                // 0x13: オペランドなし
  | Dup                // 0x14: オペランドなし
  | Call(Int)          // 0x15: +u16 argc
  | Return             // 0x16: オペランドなし
  // 算術
  | Add                // 0x20
  | Sub                // 0x21
  | Mul                // 0x22
  | Div                // 0x23
  // 比較
  | Eq                 // 0x24
  | Ne                 // 0x25
  | Lt                 // 0x26
  | Le                 // 0x27
  | Gt                 // 0x28
  | Ge                 // 0x29
  | And                // 0x2A
  | Or                 // 0x2B
  // ジャンプ
  | Jump(Int)          // 0x30: +u16 offset
  | JumpIfFalse(Int)   // 0x31: +u16 offset
  // レコード・フィールド
  | GetField(Int)      // 0x40: +u16 field_idx
  // 未知オペコード（フォールバック）
  | Unknown(Int)       // 上記以外のバイト値
```

#### 2. `DecodeResult` レコード型

`decode_opcode` の戻り値。次の PC（`next_pc`）を含む。

```favnir
type DecodeResult = {
  op:      Opcode
  next_pc: Int
}
```

#### 3. `fn decode_opcode(bytes: Bytes, pc: Int) -> Result<DecodeResult, String>`

指定位置のバイトを読み、対応する `Opcode` と次の PC を返す。

**デコード規則（主要 opcode）:**

| opcode byte | Opcode バリアント | next_pc |
|---|---|---|
| 0x01 | `Const(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x02 | `ConstUnit` | pc + 1 |
| 0x03 | `ConstTrue` | pc + 1 |
| 0x04 | `ConstFalse` | pc + 1 |
| 0x10 | `LoadLocal(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x11 | `StoreLocal(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x12 | `LoadGlobal(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x13 | `Pop` | pc + 1 |
| 0x14 | `Dup` | pc + 1 |
| 0x15 | `Call(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x16 | `Return` | pc + 1 |
| 0x20〜0x2B | `Add`/`Sub`/.../`Or` | pc + 1 |
| 0x30 | `Jump(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x31 | `JumpIfFalse(read_u16_le(bytes, pc+1))` | pc + 3 |
| 0x40 | `GetField(read_u16_le(bytes, pc+1))` | pc + 3 |
| その他 | `Unknown(byte_value)` | pc + 1 |

**実装注記（Favnir 固有）:**
- `Bytes.get(bytes, pc)` は `Result<Int, String>` を返す → `bind r <- Bytes.get(...)` で `r = ok(Int)` がバインドされる → `match r { ok(byte) => ... err(e) => ... }` で展開する
- `Bytes.read_u16_le(bytes, pc+1)` も同様（`Result<Int, String>`）
- `Bytes.from_hex(s)` も `Result<Bytes, String>` → `bind r <- Bytes.from_hex(...)` + `match r { ok(bytes) => ... }` が必要
- **`bind` は Result をアンラップしない**（単純代入）。`bind x <- expr` は `x = expr の評価結果` を代入するだけ。アンラップは `match` で行う
- `Bytes.read_u16` は BE。バイトコードデコードには `Bytes.read_u16_le`（v23.4.0 追加）を使う

#### 4. `fn opcode_to_string(op: Opcode) -> String`

テスト・デバッグ用表示関数。

```favnir
fn opcode_to_string(op: Opcode) -> String {
  match op {
    Const(idx) => f"Const({idx})"
    ConstUnit   => "ConstUnit"
    Return      => "Return"
    // ...
  }
}
```

---

## スコープ外（Phase 2 以降）

| 機能 | フェーズ |
|---|---|
| 実行ループ（execute/step 関数） | v23.5 |
| CallFrame・スタック管理 | v23.6 |
| builtin ディスパッチ | v23.7 |
| `fav run --vm=self/vm.fav` CLI フラグ | v23.5〜v23.8 |
| `VMValue` 型定義 | v23.5 |

---

## テスト（v234000_tests）

5 件。`include_str!("../self/vm.fav")` + main 関数追記パターン。

| テスト名 | 内容 |
|---|---|
| `version_is_23_4_0` | Cargo.toml に `version = "23.4.0"` が含まれる |
| `vm_fav_file_exists` | `self/vm.fav` が存在し include_str! で読み込める |
| `vm_fav_compiles` | vm.fav を parse + build_artifact し、エラー 0 |
| `decode_const_opcode` | bytes=[0x01, 0x03, 0x00] → `decode_opcode(bytes,0)` → `"Const(3)"` |
| `changelog_has_v23_4_0` | CHANGELOG.md に `[v23.4.0]` が含まれる |

**decode_const_opcode テスト詳細:**
- `Bytes.from_hex("010300")` → `Result<Bytes, String>`（`ok(bytes)` でアンラップ）
- bytes = [0x01, 0x03, 0x00]
- `0x01` = Const opcode、`read_u16_le(bytes, 1)` = LE([0x03, 0x00]) = 3
- 期待: `opcode_to_string(r.op)` == `"Const(3)"`
- テスト Favnir コード（from_hex の Result アンラップが必要）:
  ```favnir
  bind hex_result <- Bytes.from_hex("010300")
  match hex_result {
    ok(bytes) => {
      bind dec_result <- decode_opcode(bytes, 0)
      match dec_result {
        ok(r) => opcode_to_string(r.op)
        err(e) => e
      }
    }
    err(e) => e
  }
  ```

---

## 完了条件チェックリスト

- [ ] `Bytes.read_u16_le` / `Bytes.read_u24_le` が vm.rs に追加されている（LE バイトコード読み取り用）
- [ ] `fav/self/vm.fav` が新規作成される
- [ ] `type Opcode` — 27 バリアント（Const〜Unknown）が定義される
- [ ] `type DecodeResult` — `{ op: Opcode, next_pc: Int }` が定義される
- [ ] `fn decode_byte_no_operand` / `fn decode_byte_with_u16_le` ヘルパーが定義される
- [ ] `fn decode_opcode` / `fn opcode_to_string` が定義される
- [ ] `vm_fav_compiles` テストが PASS（parse + build_artifact エラーなし）
- [ ] `decode_const_opcode` テストが PASS（bytes=[0x01,0x03,0x00] → `"Const(3)"`）
- [ ] `cargo test v234000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1905 件以上合格）
- [ ] `CHANGELOG.md` に v23.4.0 エントリ
- [ ] `benchmarks/v23.4.0.json` 作成済み
- [ ] `site/content/docs/tools/vm-fav.mdx` 作成済み（vm.fav フェーズ計画の概要）
