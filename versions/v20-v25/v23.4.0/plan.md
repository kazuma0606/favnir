# v23.4.0 実装計画 — vm.fav Phase 1（バイトコードデコード）

## フェーズ概要

vm.fav を `fav/self/vm.fav` として新規作成し、Opcode 型定義と decode_opcode 関数を実装する。
実行ループは含まない（Phase 2 = v23.5 以降のスコープ）。

---

## T0: `Bytes.read_u16_le` / `Bytes.read_u24_le` を vm.rs に追加

`codegen.rs` は u16 LE でバイトコードを生成するが、既存の `Bytes.read_u16` は BE。
vm.fav で正しくデコードするための LE バージョンを追加する。

### T0-1: vm.rs — vm_call_builtin に 2 ハンドラ追加

`"Bytes.read_u32"` アームの直後に追加:

```rust
"Bytes.read_u16_le" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u16_le: arg0 not Bytes".to_string()) };
    let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u16_le: arg1 not Int".to_string()) };
    match bytes_get_arc(id) {
        Some(arc) if off + 2 <= arc.len() => {
            let v = arc[off] as i64 | (arc[off + 1] as i64) << 8;
            Ok(ok_vm(VMValue::Int(v)))
        }
        _ => Ok(err_vm(VMValue::Str("Bytes.read_u16_le: out of bounds".into()))),
    }
}
"Bytes.read_u24_le" => {
    let mut it = args.into_iter();
    let id  = match it.next() { Some(VMValue::Bytes(id)) => id, _ => return Err("Bytes.read_u24_le: arg0 not Bytes".to_string()) };
    let off = match it.next() { Some(VMValue::Int(n))    => n as usize, _ => return Err("Bytes.read_u24_le: arg1 not Int".to_string()) };
    match bytes_get_arc(id) {
        Some(arc) if off + 3 <= arc.len() => {
            let v = arc[off] as i64 | (arc[off + 1] as i64) << 8 | (arc[off + 2] as i64) << 16;
            Ok(ok_vm(VMValue::Int(v)))
        }
        _ => Ok(err_vm(VMValue::Str("Bytes.read_u24_le: out of bounds".into()))),
    }
}
```

`cargo check --bin fav` でエラー 0 を確認。

---

## T1: `fav/self/vm.fav` 新規作成

### T1-1: ファイル構造（全体）

```favnir
// v23.4.0: vm.fav — Favnir セルフホスト VM Phase 1（バイトコードデコード）
// 対応 opcode: codegen.rs の主要バリアント 27 件（Phase 1）
// Phase 2 以降: 実行ループ / CallFrame / VMValue / builtin ディスパッチ

type Opcode =
  | Const(Int)
  | ConstUnit
  | ConstTrue
  | ConstFalse
  | LoadLocal(Int)
  | StoreLocal(Int)
  | LoadGlobal(Int)
  | Pop
  | Dup
  | Call(Int)
  | Return
  | Add
  | Sub
  | Mul
  | Div
  | Eq
  | Ne
  | Lt
  | Le
  | Gt
  | Ge
  | And
  | Or
  | Jump(Int)
  | JumpIfFalse(Int)
  | GetField(Int)
  | Unknown(Int)

type DecodeResult = {
  op:      Opcode
  next_pc: Int
}

fn read_operand_u16(bytes: Bytes, pc: Int) -> Result<Int, String> {
  Bytes.read_u16(bytes, pc)
}

fn decode_byte_no_operand(byte: Int, pc: Int) -> Result<DecodeResult, String> {
  match byte {
    0x02 => ok({ op: ConstUnit,   next_pc: pc + 1 })
    0x03 => ok({ op: ConstTrue,   next_pc: pc + 1 })
    0x04 => ok({ op: ConstFalse,  next_pc: pc + 1 })
    0x13 => ok({ op: Pop,         next_pc: pc + 1 })
    0x14 => ok({ op: Dup,         next_pc: pc + 1 })
    0x16 => ok({ op: Return,      next_pc: pc + 1 })
    0x20 => ok({ op: Add,         next_pc: pc + 1 })
    0x21 => ok({ op: Sub,         next_pc: pc + 1 })
    0x22 => ok({ op: Mul,         next_pc: pc + 1 })
    0x23 => ok({ op: Div,         next_pc: pc + 1 })
    0x24 => ok({ op: Eq,          next_pc: pc + 1 })
    0x25 => ok({ op: Ne,          next_pc: pc + 1 })
    0x26 => ok({ op: Lt,          next_pc: pc + 1 })
    0x27 => ok({ op: Le,          next_pc: pc + 1 })
    0x28 => ok({ op: Gt,          next_pc: pc + 1 })
    0x29 => ok({ op: Ge,          next_pc: pc + 1 })
    0x2a => ok({ op: And,         next_pc: pc + 1 })
    0x2b => ok({ op: Or,          next_pc: pc + 1 })
    _    => ok({ op: Unknown(byte), next_pc: pc + 1 })
  }
}

fn decode_byte_with_u16(bytes: Bytes, byte: Int, pc: Int) -> Result<DecodeResult, String> {
  bind r <- Bytes.read_u16_le(bytes, pc + 1)
  match r {
    ok(operand) => match byte {
      0x01 => ok({ op: Const(operand),      next_pc: pc + 3 })
      0x10 => ok({ op: LoadLocal(operand),  next_pc: pc + 3 })
      0x11 => ok({ op: StoreLocal(operand), next_pc: pc + 3 })
      0x12 => ok({ op: LoadGlobal(operand), next_pc: pc + 3 })
      0x15 => ok({ op: Call(operand),       next_pc: pc + 3 })
      0x30 => ok({ op: Jump(operand),       next_pc: pc + 3 })
      0x31 => ok({ op: JumpIfFalse(operand), next_pc: pc + 3 })
      0x40 => ok({ op: GetField(operand),   next_pc: pc + 3 })
      _    => ok({ op: Unknown(byte),       next_pc: pc + 1 })
    }
    err(e) => err(e)
  }
}

fn decode_opcode(bytes: Bytes, pc: Int) -> Result<DecodeResult, String> {
  bind get_result <- Bytes.get(bytes, pc)
  match get_result {
    ok(byte) => match byte {
      0x01 | 0x10 | 0x11 | 0x12 | 0x15 | 0x30 | 0x31 | 0x40 =>
        decode_byte_with_u16(bytes, byte, pc)
      _ =>
        decode_byte_no_operand(byte, pc)
    }
    err(e) => err(e)
  }
}

fn opcode_to_string(op: Opcode) -> String {
  match op {
    Const(idx)        => f"Const({idx})"
    ConstUnit         => "ConstUnit"
    ConstTrue         => "ConstTrue"
    ConstFalse        => "ConstFalse"
    LoadLocal(s)      => f"LoadLocal({s})"
    StoreLocal(s)     => f"StoreLocal({s})"
    LoadGlobal(g)     => f"LoadGlobal({g})"
    Pop               => "Pop"
    Dup               => "Dup"
    Call(argc)        => f"Call({argc})"
    Return            => "Return"
    Add               => "Add"
    Sub               => "Sub"
    Mul               => "Mul"
    Div               => "Div"
    Eq                => "Eq"
    Ne                => "Ne"
    Lt                => "Lt"
    Le                => "Le"
    Gt                => "Gt"
    Ge                => "Ge"
    And               => "And"
    Or                => "Or"
    Jump(off)         => f"Jump({off})"
    JumpIfFalse(off)  => f"JumpIfFalse({off})"
    GetField(idx)     => f"GetField({idx})"
    Unknown(b)        => f"Unknown({b})"
  }
}
```

### T1-2: 実装上の注意事項

#### Favnir 構文ルール（v23.3.0 で確認済み）
- `let` キーワードは存在しない。`bind x <- expr` を使う
- `bind` は Result をアンラップしない（単純代入）
- `Bytes.get` / `Bytes.read_u16` は `Result<Int, String>` を返す → `bind` + `match ok/err` で展開

#### or パターン（`0x01 | 0x10 | ...`）
- Favnir の match は or パターンをサポートしている（parser.rs `parse_pattern` の `|` 処理）
- `0x01 | 0x10 | ... => decode_byte_with_u16(...)` は有効

#### 16 進数リテラル
- `0x01`, `0x10` 等は v23.2.0 で lexer に追加済み → vm.fav で使用可

#### 関数の分割設計
- `decode_opcode` を直接大きな match で書くと、`Bytes.read_u16` の Result 展開が各アームで必要になる
- ヘルパー関数 `decode_byte_no_operand` / `decode_byte_with_u16` に分割して構造化

---

## T2: `fav/src/driver.rs` — `v233000_tests::version_is_23_3_0` に `#[ignore]` 追加 + `v234000_tests` 追加

### T2-1: `#[ignore]` 追加

`fn version_is_23_3_0` に `#[ignore]` を追加（T3-1 より前に実施）。

### T2-2: `v234000_tests` モジュール追加（5 テスト）

```rust
// ── v234000_tests (v23.4.0) — vm.fav Phase 1 ─────────────────────────────────
#[cfg(test)]
mod v234000_tests {
    use super::*;

    #[test]
    fn version_is_23_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"23.4.0\""));
    }

    #[test]
    fn vm_fav_file_exists() {
        // include_str! はコンパイル時にファイルが存在しないとエラーになる
        let _src = include_str!("../self/vm.fav");
        assert!(!_src.is_empty(), "vm.fav should not be empty");
    }

    #[test]
    fn vm_fav_compiles() {
        let src = include_str!("../self/vm.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "vm.fav")
            .tokenize().expect("lex vm.fav");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse vm.fav");
        let _artifact = build_artifact(&prog);
        // build_artifact でパニックしなければ OK
    }

    #[test]
    fn decode_const_opcode() {
        // bytes = [0x01, 0x03, 0x00]
        // 0x01 = Const opcode, u16 LE [0x03, 0x00] = 3
        // => DecodeResult { op: Const(3), next_pc: 3 }
        // Bytes.from_hex は Result<Bytes, String> を返す → match で展開が必要
        let vm_src = include_str!("../self/vm.fav");
        let src = format!(r#"{}
public fn main() -> String {{
  bind hex_result <- Bytes.from_hex("010300")
  match hex_result {{
    ok(bytes) => {{
      bind dec_result <- decode_opcode(bytes, 0)
      match dec_result {{
        ok(r) => opcode_to_string(r.op)
        err(e) => e
      }}
    }}
    err(e) => e
  }}
}}"#, vm_src);
        let tokens = crate::frontend::lexer::Lexer::new(&src, "vm_test.fav")
            .tokenize().expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program().expect("parse");
        let artifact = build_artifact(&prog);
        let result = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(result, crate::value::Value::Str("Const(3)".to_string()));
    }

    #[test]
    fn changelog_has_v23_4_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v23.4.0]"));
    }
}
```

---

## T3: Cargo.toml + CHANGELOG + benchmarks + MDX

### T3-1: Cargo.toml

> **注意**: T2-1（`version_is_23_3_0` への `#[ignore]` 追加）を完了してから更新すること。

```
version = "23.3.0" → "23.4.0"
```

### T3-2: CHANGELOG.md

v23.3.0 エントリの上に追加:

```markdown
## [v23.4.0] — 2026-06-22 — vm.fav Phase 1（バイトコードデコード）

### 追加
- `fav/self/vm.fav` を新規作成（Favnir セルフホスト VM Phase 1）
- `type Opcode` — codegen.rs の主要 opcode 27 件を Favnir 型として定義
- `type DecodeResult` — `{ op: Opcode, next_pc: Int }` レコード型
- `fn decode_opcode(bytes: Bytes, pc: Int) -> Result<DecodeResult, String>` — バイト列からオペコードをデコード
- `fn opcode_to_string(op: Opcode) -> String` — デバッグ・テスト用 opcode 文字列化
- `site/content/docs/tools/vm-fav.mdx` — vm.fav フェーズ計画ドキュメント

---
```

### T3-3: benchmarks/v23.4.0.json

```json
{
  "version": "23.4.0",
  "date": "2026-06-22",
  "feature": "vm.fav Phase 1（バイトコードデコード）",
  "test_count": XXXX,
  "notes": "fav/self/vm.fav 新規作成。Opcode 型定義 + decode_opcode 実装。実行ループは v23.5 以降。"
}
```

### T3-4: site/content/docs/tools/vm-fav.mdx

vm.fav の概要と Phase 1〜5 計画を記述する MDX ドキュメント。

---

## 実装順序

```
T1（vm.fav 作成）   ← 最初（Favnir ファイル本体）
T2-1（#[ignore]）   ← T3-1 より前に実施（必須）
T2-2（tests）       ← T1 完了後
T3-1（Cargo.toml）  ← T2-1 完了後
T3-2〜T3-4（docs）  ← T1〜T3-1 完了後
```

---

## or パターン構文の確認

Favnir の match では or パターンが使える:

```favnir
match byte {
  0x01 | 0x10 | 0x11 | 0x12 | 0x15 | 0x30 | 0x31 | 0x40 =>
    decode_byte_with_u16(bytes, byte, pc)
  _ =>
    decode_byte_no_operand(byte, pc)
}
```

parser.rs の `parse_pattern` で `|` を処理し `Pattern::Or` を生成する実装が存在するか事前確認すること。もし or パターンが動作しない場合は、各バイト値を個別の match アームに展開する。

---

## リスク事項

| リスク | 対応 |
|---|---|
| or パターンが match で動作しない | 個別アームに展開（`0x01 => decode_byte_with_u16(bytes, byte, pc)` × 8 件） |
| `Bytes.get` が checker で `Type::Unknown` → `match ok/err` で型エラー | `Bytes` は `Type::Unknown` namespace なので checker スキップ。実際は ok(Int)/err(String) を返すので実行時には動作する |
| レコード型 `{ op: Const(3), next_pc: 3 }` の生成が失敗 | 既存の `BuildRecord` opcode で生成されるはずだが、checker が検出しない場合は cargo test で確認 |
| `f"Const({idx})"` f-string 内の Int 展開 | Favnir の f-string は Int を自動展開する（既存コードで確認済み）|
