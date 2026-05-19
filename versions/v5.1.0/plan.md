# Favnir v5.1.0 実装計画

作成日: 2026-05-19

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---------|---------|
| `fav/src/middle/checker.rs` | A: sum type 再帰制約緩和、B/C/E: 型シグネチャ登録 |
| `fav/src/backend/vm.rs` | B/C/E: VM primitive 実装 |
| `fav/src/middle/compiler.rs` | B/C: namespace 登録（`Int` は登録済み、`IO` も登録済み）|
| `docs/bytecode-spec.md` | D: 新規作成 |

`Int` と `IO` は compiler.rs の namespace リストに登録済みのため、
vm.rs と checker.rs への追加だけで関数が有効になる。

---

## A. 再帰的 sum type の許容

### 現状確認が必要な点

`error_catalog.rs` の E0251 は「recursive type without indirection」だが、
checker.rs の E0251 は「abstract stage の直接呼び出し禁止」に使われている。
実際に `type Expr = | Add(Expr, Expr)` を書いてエラーになるか最初に確認する。

### 実装方針

checker.rs の型定義検査（`TypeBody::Sum` のバリアント型検査部分）を探し、
sum type バリアントの自己参照を許容するよう条件を変更する。

record type（`TypeBody::Record`）の直接再帰は引き続きエラー。

```rust
// checker.rs — 変更箇所の方針
// TypeBody::Sum のバリアント内の型参照に対し、
// 親の型名への再帰参照は許可する（record の再帰はエラーのまま）
```

---

## B. ファイル I/O VM primitive

### vm.rs への追加（`call_builtin` の dispatch match に追加）

```rust
"IO.read_file_raw" => {
    // args[0]: Str(path)
    // std::fs::read_to_string(path)
    // Ok → ok_vm(VMValue::Str(content))
    // Err → err_vm(VMValue::Str(e.to_string()))
}
"IO.write_file_raw" => {
    // args[0]: Str(path), args[1]: Str(content)
    // std::fs::write(path, content)
}
"IO.write_bytes_raw" => {
    // args[0]: Str(path), args[1]: List(bytes)
    // 各 VMValue::Int を u8 にキャスト（& 0xFF）
    // std::fs::write(path, byte_vec)
}
"IO.file_exists_raw" => {
    // args[0]: Str(path)
    // std::path::Path::new(path).is_file()
    // Ok → VMValue::Bool(result)
}
```

### checker.rs への追加（`check_builtin_call` の型シグネチャ）

```rust
"IO.read_file_raw"   => (vec![Type::Str], Type::Result(Box::new(Type::Str), Box::new(Type::Str)))
"IO.write_file_raw"  => (vec![Type::Str, Type::Str], Type::Result(Box::new(Type::Unit), Box::new(Type::Str)))
"IO.write_bytes_raw" => (vec![Type::Str, Type::List(Box::new(Type::Int))], Type::Result(Box::new(Type::Unit), Box::new(Type::Str)))
"IO.file_exists_raw" => (vec![Type::Str], Type::Bool)
```

エフェクト: 全て `!Io`（既存 BUILTIN_EFFECTS に登録済み）。

---

## C. ビット演算 VM primitive

### vm.rs への追加

```rust
"Int.shl"  => (Int(x), Int(n)) → Int(x << n)
"Int.shr"  => (Int(x), Int(n)) → Int(x >> n)   // 算術右シフト（Rust の >> は符号付き）
"Int.band" => (Int(x), Int(y)) → Int(x & y)
"Int.bor"  => (Int(x), Int(y)) → Int(x | y)
"Int.bxor" => (Int(x), Int(y)) → Int(x ^ y)
"Int.bnot" => (Int(x))         → Int(!x)
"Int.to_byte" => (Int(x))      → Int(x & 0xFF)
```

### checker.rs への追加

```rust
"Int.shl"     => (vec![Type::Int, Type::Int], Type::Int)
"Int.shr"     => (vec![Type::Int, Type::Int], Type::Int)
"Int.band"    => (vec![Type::Int, Type::Int], Type::Int)
"Int.bor"     => (vec![Type::Int, Type::Int], Type::Int)
"Int.bxor"    => (vec![Type::Int, Type::Int], Type::Int)
"Int.bnot"    => (vec![Type::Int], Type::Int)
"Int.to_byte" => (vec![Type::Int], Type::Int)
```

エフェクト: なし（純粋関数）。

`Int` は compiler.rs の namespace リストに登録済みのため compiler.rs の変更不要。

---

## D. バイトコード仕様書

`docs/bytecode-spec.md` を新規作成。内容は `spec.md` の「D. バイトコード仕様書」セクションをそのまま使用。

`artifact.rs` と `codegen.rs` を参照して定数エントリ形式とオペコード一覧を確定。
作成後は **変更禁止（凍結）** とし、ファイル先頭に凍結宣言を記載。

---

## E. `String.chars`

### vm.rs への追加

```rust
"String.chars" => {
    // args[0]: Str(s)
    // s.chars().map(|c| VMValue::Str(c.to_string())).collect()
    // → VMValue::List(chars)
}
```

### checker.rs への追加

```rust
"String.chars" => (vec![Type::Str], Type::List(Box::new(Type::Str)))
```

`String` は compiler.rs の namespace リストに登録済みのため compiler.rs の変更不要。

---

## 実装順序

```
1. A: 再帰的 sum type — 動作確認 → 必要なら checker.rs 修正
2. E: String.chars — 変更量が少なく warming up に最適
3. C: ビット演算 — vm.rs + checker.rs に一括追加
4. B: ファイル I/O — vm.rs + checker.rs に一括追加
5. D: bytecode-spec.md — artifact.rs / codegen.rs から抽出して作成
```

---

## テスト方針

各追加について `fav/src/backend/vm_stdlib_tests.rs` にテストを追加:

```rust
// B: ファイル I/O
fn test_io_read_write_file()
fn test_io_write_bytes()
fn test_io_file_exists()

// C: ビット演算
fn test_int_shl()
fn test_int_shr()
fn test_int_band_bor_bxor()
fn test_int_bnot()
fn test_int_to_byte()

// E: String.chars
fn test_string_chars()
fn test_string_chars_empty()
fn test_string_chars_unicode()
```

A（再帰 sum type）は checker tests (`fav/src/middle/checker.rs` の test モジュール）に追加:

```rust
fn test_recursive_sum_type_ok()
fn test_recursive_record_type_err()  // 引き続き E0251
```
