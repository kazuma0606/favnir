# Roadmap v23.1.0 〜 v24.0.0 — VM in Favnir

Date: 2026-06-18

## 目標

v23.0「Distributed Scale」で「スケールアウトできる」を達成した。
残る最大の課題は「**VM が唯一の Rust 依存**」であることだ。

セルフホストは compiler / checker / CLI まで達成したが、
VM だけは「Rust（恒久）」として保留してきた。
vm.fav を作ることは生産性向上を目的にしない。
「**Favnir の表現力が VM 実装に足りるほど成熟した**」という証明である。

**完了条件:**
1. `Bytes` 型 + ビット演算 + `Mut<T>` が動作する
2. vm.fav が全 opcode をデコード・実行できる
3. `fav run --vm=self/vm.fav` で hello.fav が動作する
4. vm.fav での実行結果が Rust VM の結果と一致する（500 件以上）
5. `fav test self/vm.fav` 全件 PASS

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| `Bytes` の表現 | VM 内では `VMValue::Bytes(Arc<Vec<u8>>)`。Favnir 側では `Bytes` 型 |
| ビット演算の実装 | `Int.bit_and / bit_or / bit_xor / bit_not / shift_left / shift_right`（prefix 記法）|
| `Mut<T>` のスコープ | 線形型 `-o` と組み合わせ。スコープ外持ち出しはコンパイルエラー |
| dispatch テーブル | `Mut<Map<Int, OpHandler>>` — `fn(VMState, Int) -> VMState` 型の関数値を格納 |
| vm.fav のフェーズ分割 | 5フェーズ（バイトコードデコード / 実行ループ / 関数呼び出し / builtin / 自己実行） |
| vm.fav の実行方法 | `fav run --vm=self/vm.fav <target.fav>` フラグで切り替え |
| Rust VM の位置付け | 実行エンジン（バイトコード dispatch ループ）は永続的に Rust で維持。設計上の意図 |

---

## 前提条件

vm.fav を書くためには、Favnir に現時点で不足しているプリミティブが必要。
これらを先行して整備する（v23.1〜v23.3）。

```
- Bytes 型（生バイト列）                                   → v23.1
- ビット演算（& | ^ >> <<）                                → v23.2
- 可変配列（VMValue を push/pop できるスタック）            → v23.3
- ファーストクラス関数値の Map 格納（dispatch テーブル用）  → v23.3（付記）
```

---

## バージョン計画

### v23.1 — `Bytes` 型

**テーマ**: 生バイト列を Favnir から直接操作できる型。

```favnir
// Bytes 型の基本操作
bind data  <- Bytes.from_hex("464f4f")    // "FOO"
bind byte  <- Bytes.get(data, 0)          // Int: 70 ('F')
bind slice <- Bytes.slice(data, 1, 3)     // Bytes: "OO"
bind merged <- Bytes.concat(data, data)   // Bytes: "FOOFOO"
bind s     <- Bytes.to_utf8(data)         // Result<String, String>
bind hex   <- Bytes.to_hex(data)          // "464f4f"

// バイナリファイルの読み書き
bind raw <- Bytes.read_file("data.bin")
Bytes.write_file("out.bin", raw)
```

#### 実装要件

- `VMValue::Bytes(Arc<Vec<u8>>)` バリアントを追加
- `is_known_builtin_namespace` / checker namespace list / compiler builtins list に `"Bytes"` を追加
- `Bytes.read_u16 / read_u24 / read_u32`（vm.fav のバイトコードデコードで使用）

---

### v23.2 — ビット演算

**テーマ**: 整数のビットレベル操作を追加する。

```favnir
// ビット演算
bind a <- Int.bit_and(0xFF, 0x0F)    // 0x0F
bind b <- Int.bit_or(0xF0, 0x0F)     // 0xFF
bind c <- Int.bit_xor(0xFF, 0x0F)    // 0xF0
bind d <- Int.bit_not(0x00)          // 0xFFFFFFFF
bind e <- Int.shift_left(1, 4)       // 16
bind f <- Int.shift_right(256, 4)    // 16

// opcode デコード（VM 実装で使う）
bind opcode  <- Int.bit_and(Int.shift_right(word, 24), 0xFF)
bind operand <- Int.bit_and(word, 0x00FFFFFF)
```

#### 実装要件

- `vm_call_builtin` の `"Int"` namespace に 6 関数を追加
- checker の `"Int"` 組み込み関数リストに追加
- 16進数リテラル（`0xFF`）のパース対応（lexer / parser）

---

### v23.3 — 可変コレクション（`Mut<T>`）

**テーマ**: 安全な可変性を限定的に導入する。VM のスタック・ヒープ操作に不可欠。

```favnir
// Mut<List<T>>: 可変配列
bind stack <- Mut.list<VMValue>()
Mut.push(stack, VMValue.Int(42))
bind top <- Mut.pop(stack)

// Mut<Map<K, V>>: 可変マップ（VM のローカル変数テーブル）
bind locals <- Mut.map<String, VMValue>()
Mut.set(locals, "x", VMValue.Int(10))
bind x <- Mut.get(locals, "x")
```

`Mut<T>` はスコープを抜けると自動解放（線形型 `-o` と組み合わせ）。
スコープ外への持ち出しはコンパイルエラー。

#### 付記: ファーストクラス関数値の Map 格納（dispatch テーブル用）

vm.fav の dispatch テーブルは「opcode → 処理関数」のマッピングである。
これは**永続化やバイト列化ではなく**、関数値を実行時に Map へ格納・参照できることを指す。

```favnir
// 関数値を Map に格納（dispatch テーブル）
type OpHandler = fn(VMState, Int) -> VMState

bind dispatch <- Mut.map<Int, OpHandler>()
Mut.set(dispatch, 0x01, handle_load_const)
Mut.set(dispatch, 0x02, handle_load_local)

// 実行ループ内で参照・呼び出し
bind handler <- Mut.get(dispatch, opcode)
match handler {
  ok(f)  => f(state, operand)
  err(_) => VMState.error(state, f"unknown opcode: {opcode}")
}
```

**実装要件**: `fn(A) -> B` 型の値を `Mut<Map<K, fn(A) -> B>>` に格納・取り出しできること。
checker / compiler 両方で「関数型を Map の値型として扱う」対応が必要。

---

### v23.4 — vm.fav Phase 1（バイトコードデコード + 基本 opcode）

**テーマ**: 実際に vm.fav を書き始める最初のフェーズ。

```favnir
// opcodes の定義
type Opcode =
  | LoadConst(Int)    // str_table からロード
  | LoadLocal(Int)    // ローカル変数ロード
  | StoreLocal(Int)   // ローカル変数ストア
  | Add | Sub | Mul | Div
  | Eq | Lt | Gt
  | Jump(Int) | JumpIfFalse(Int)
  | Return
  // ...

// バイトコードのデコード
fn decode_opcode(bytes: Bytes, pc: Int) -> Opcode {
  bind byte <- Bytes.get(bytes, pc)
  match byte {
    0x01 => Opcode.LoadConst(Bytes.read_u24(bytes, pc + 1))
    0x02 => Opcode.LoadLocal(Bytes.read_u16(bytes, pc + 1))
    // ...
  }
}
```

---

### v23.5 — vm.fav Phase 2（スタックベース実行ループ）

**テーマ**: opcode インタープリタの核心部分。

```favnir
// VM の実行状態
type VMState = {
  stack:    Mut<List<VMValue>>
  locals:   Mut<Map<Int, VMValue>>
  pc:       Int
  bytecode: Bytes
}

// メインループ
fn execute(state: VMState) -> Result<VMValue, String> {
  bind opcode <- decode_opcode(state.bytecode, state.pc)
  match opcode {
    Opcode.Add => {
      bind b <- Mut.pop(state.stack)
      bind a <- Mut.pop(state.stack)
      Mut.push(state.stack, vm_add(a, b))
      execute({ ...state, pc: state.pc + 1 })
    }
    Opcode.Return => {
      Mut.pop(state.stack)
    }
    // ...
  }
}
```

---

### v23.6 — vm.fav Phase 3（関数呼び出し・クロージャ）

**テーマ**: スタックフレーム管理とクロージャキャプチャの実装。

```favnir
type CallFrame = {
  locals:      Mut<Map<Int, VMValue>>
  return_addr: Int
  fn_def:      FnDef
}

type VMState = {
  call_stack:  Mut<List<CallFrame>>
  value_stack: Mut<List<VMValue>>
  pc:          Int
  // ...
}
```

---

### v23.7 — vm.fav Phase 4（stdlib・builtin 呼び出し）

**テーマ**: Rust で実装された builtin（`List.map` 等）を vm.fav から呼び出せるようにする。

この層は「Favnir ↔ Rust の境界」として永続化する。

```favnir
// builtin ディスパッチ
fn call_builtin(name: String, args: List<VMValue>) -> Result<VMValue, String> {
  match name {
    "List.map"    => builtin_list_map(args)
    "String.trim" => builtin_string_trim(args)
    // ...
    _ => Result.err(f"unknown builtin: {name}")
  }
}
```

---

### v23.8 — vm.fav Phase 5（テスト通過・自己実行）

**テーマ**: vm.fav で vm.fav 自体を実行できることを検証する。

```bash
# vm.fav で hello.fav を実行
fav run --vm=self/vm.fav hello.fav

# vm.fav 自体のテスト
fav test self/vm.fav
```

**完了条件**: 既存の `fav test` スイートの主要テスト（500 件以上）が
vm.fav 経由でも通ること。

---

## v24.0 — VM in Favnir マイルストーン宣言

**完了条件:**
1. `Bytes` 型 + ビット演算 + `Mut<T>` が動作する
2. vm.fav が全 opcode をデコード・実行できる
3. `fav run --vm=self/vm.fav` で hello.fav が動作する
4. vm.fav での実行結果が Rust VM の結果と一致する（500 件以上）
5. `fav test self/vm.fav` 全件 PASS

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v22.1-v23.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v24.1-v25.0.md`
- マスタースケジュール: `versions/roadmap-v20.1-v25.0.md`
