# v23.6.0 — vm.fav Phase 3（制御フロー・ローカル変数）

Date: 2026-06-22

## 目標

vm.fav に制御フロー（Jump / JumpIfFalse）とローカル変数アクセス（LoadLocal / StoreLocal）を追加し、
基本的な単一フレーム実行を完成させる。残りの比較演算・論理演算・除算も同時に実装する。

## スコープ

### Favnir（fav/self/vm.fav への変更）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| シグネチャ変更 | `fn vm_execute` | `locals: Int` パラメータを追加 |
| シグネチャ変更 | `fn vm_run` | `Mut.map()` でローカル変数マップを生成し `vm_execute` に渡す |
| 再帰呼び出し更新 | Phase 2 全アーム | `vm_execute(bytecode, stack, dec.next_pc)` → `(bytecode, stack, locals, dec.next_pc)` |
| 新オペコード追加 | `vm_execute` | 12 件（下表参照） |

### Phase 3 対応オペコード（12 件）

| オペコード | バイト | 動作 |
|---|---|---|
| `Jump(off)` | `0x30 u16LE` | `pc = next_pc + off`（相対前方ジャンプ） |
| `JumpIfFalse(off)` | `0x31 u16LE` | 条件 pop → false なら `pc = next_pc + off` |
| `LoadLocal(slot)` | `0x10 u16LE` | `Mut.get(locals, slot)` → push |
| `StoreLocal(slot)` | `0x11 u16LE` | pop → `Mut.set(locals, slot, val)` |
| `Ne` | `0x25` | pop b, pop a: push VMBool(a != b)（VMInt のみ） |
| `Lt` | `0x26` | pop b, pop a: push VMBool(a < b) |
| `Le` | `0x27` | pop b, pop a: push VMBool(a <= b) |
| `Gt` | `0x28` | pop b, pop a: push VMBool(a > b) |
| `Ge` | `0x29` | pop b, pop a: push VMBool(a >= b) |
| `And` | `0x2a` | pop b, pop a: push VMBool(a && b)（VMBool のみ） |
| `Or` | `0x2b` | pop b, pop a: push VMBool(a \|\| b)（VMBool のみ） |
| `Div` | `0x23` | pop b, pop a: push VMInt(a / b)（b=0 → err） |

> **Jump オフセット注記**: codegen.rs の `patch_jump` より、オフセットは「Jump 命令直後の pc（= next_pc）からの相対前方距離」。
> `target = next_pc + off`。

### Rust（変更なし）

Phase 3 に必要な Rust primitive は全て実装済み。

| 使用 primitive | 追加バージョン | 用途 |
|---|---|---|
| `Mut.map()` | v23.3.0 | ローカル変数マップ生成 |
| `Mut.set(h, k, v)` | v23.3.0 | StoreLocal — `k: Int` がキーとして動作（MutMap は `VMValue` 型のキーを使用） |
| `Mut.get(h, k)` | v23.3.0 | LoadLocal — 同上 |

---

## 型・関数変更

```favnir
// Phase 3 シグネチャ（Phase 2 から変更）
// locals: Int — Mut<Map<VMVal, VMVal>> の opaque handle
fn vm_execute(bytecode: Bytes, stack: Int, locals: Int, pc: Int) -> Result<VMVal, String>

// vm_run は変わらず公開 API（ローカル変数を内部で作成）
fn vm_run(bytecode: Bytes) -> Result<VMVal, String>
```

---

## 設計上の注意点

| # | 注意点 | 対応 |
|---|---|---|
| 1 | `vm_execute` 全再帰呼び出しに `locals` を追加 | Phase 2 の全アームを一括更新（検索: `vm_execute(bytecode, stack, dec.next_pc)`） |
| 2 | `Mut.set` は `Result<Unit, String>` を返す | `bind _ <- Mut.set(locals, slot, val)` で sequencing |
| 3 | `Mut.get` の Int キー | MutMap は `VMValue` をキーとして比較するため `slot: Int` が `VMValue::Int(n)` として正しく照合される |
| 4 | `JumpIfFalse` の条件値 pop | JumpIfFalse はスタックから条件値を pop した後にジャンプ判定する |
| 5 | `And`/`Or` 演算子 | Favnir では `&&` / `\|\|` が使用可能（`AmpAmp` / `PipePipe` トークン） |
| 6 | `Div` のゼロ除算 | `VMInt(bi)` パターン内で `match bi { 0 => err ... _ => ... }` で処理 |
| 7 | `locals: Int` 型宣言 | MutMap handle は `Type::Unknown` → `Int` に unify（v23.5.0 と同じパターン） |

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_6_0` | Cargo.toml に `version = "23.6.0"` | — |
| `vm_fav_phase3_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `execute_locals` | hex `"012a0011000010000016"` → vm_run | `"VMInt(42)"` |
| `execute_jump` | hex `"0431060001010030030001020016"` → vm_run | `"VMInt(2)"` |
| `changelog_has_v23_6_0` | CHANGELOG.md に `[v23.6.0]` | — |

### バイトコード詳細

**`execute_locals`**: `"012a0011000010000016"`
```
pc=0: 01 2A 00  Const(42)      → push VMInt(42)
pc=3: 11 00 00  StoreLocal(0)  → pop VMInt(42), locals[0] = VMInt(42)
pc=6: 10 00 00  LoadLocal(0)   → push locals[0] = VMInt(42)
pc=9: 16        Return          → pop → VMInt(42)
```

**`execute_jump`**: `"0431060001010030030001020016"`
```
pc=0:  04        ConstFalse      → push VMBool(false)
pc=1:  31 06 00  JumpIfFalse(6)  → pop false, next_pc=4, target=4+6=10 → jump
pc=4:  01 01 00  Const(1)        → (スキップ) VMInt(1) true path
pc=7:  30 03 00  Jump(3)         → (スキップ) next_pc=10, target=10+3=13
pc=10: 01 02 00  Const(2)        → push VMInt(2) ← false path
pc=13: 16        Return           → pop → VMInt(2)
```

---

## ロードマップとの関係

ロードマップ v23.6 は「CallFrame / VMState レコード型によるスタックフレーム管理」を示しているが、
Favnir の型チェッカーが `Mut<T>` をレコードフィールド型として扱えないため（v23.5.0 で確認済み）、
Phase 3 では「単一フレームで locals を持つ実行」に留め、CallFrame / call stack は Phase 4（v23.7）以降に持ち越す。

---

## 完了条件

- [ ] `vm_execute` に `locals: Int` パラメータが追加される
- [ ] Phase 2 全アームの再帰呼び出しが `locals` を渡すよう更新される
- [ ] `vm_run` が `Mut.map()` でローカル変数マップを生成する
- [ ] Jump / JumpIfFalse / LoadLocal / StoreLocal / Ne / Lt / Le / Gt / Ge / And / Or / Div（計 12 件）が追加される
- [ ] `cargo test v236000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）
- [ ] `CHANGELOG.md` に v23.6.0 エントリ
- [ ] `benchmarks/v23.6.0.json` 作成済み
- [ ] `site/content/docs/tools/vm-fav.mdx` に Phase 3 セクション追記
