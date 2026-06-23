# v23.5.0 — vm.fav Phase 2（スタックベース実行ループ）

Date: 2026-06-22

## 目標

vm.fav に `VMVal` 型と `vm_execute` ループを追加し、基本オペコードを実際に実行できるようにする。
Phase 1 のバイトコードデコーダを活用し、スタックベースの評価エンジンを Favnir で実装する。

## スコープ

### Favnir（fav/self/vm.fav への追記）

| 追加要素 | 内容 |
|---|---|
| `type VMVal` | スタック値 sum type（VMInt / VMBool / VMUnit） |
| `fn vmval_to_string` | デバッグ用文字列化 |
| `fn vm_execute` | 実行ループ（再帰）— Phase 2 オペコードを dispatch |
| `fn vm_run` | エントリポイント（Mut.list でスタック生成 → vm_execute） |

### Phase 2 対応オペコード（11 件）

| オペコード | バイト | 動作 |
|---|---|---|
| `ConstUnit` | `0x02` | push VMUnit |
| `ConstTrue` | `0x03` | push VMBool(true) |
| `ConstFalse` | `0x04` | push VMBool(false) |
| `Const(n)` | `0x01 u16LE` | push VMInt(n)（Phase 2 簡略: index を整数値として扱う） |
| `Pop` | `0x13` | top 破棄 |
| `Dup` | `0x14` | top 複製 |
| `Return` | `0x16` | pop → Result.ok(v) |
| `Add` | `0x20` | pop b, pop a: push VMInt(a+b) |
| `Sub` | `0x21` | pop b, pop a: push VMInt(a-b) |
| `Mul` | `0x22` | pop b, pop a: push VMInt(a*b) |
| `Eq` | `0x24` | pop b, pop a: push VMBool(a==b)（VMInt のみ） |

その他オペコードは `Result.err("vm_execute: unimplemented opcode")` を返す。

### Rust（変更なし）

Phase 2 に必要な Rust primitive は v23.3.0 / v23.4.0 で整備済み。追加不要。

| 使用 primitive | 追加バージョン | 備考 |
|---|---|---|
| `Mut.list()` | v23.3.0 | スタック生成 |
| `Mut.push(h, v)` | v23.3.0 | → `Result<Unit, String>` |
| `Mut.pop(h)` | v23.3.0 | → `Result<Unknown, String>` |
| `Mut.peek(h)` | v23.3.0 | → `Result<Unknown, String>` |
| `Mut.len(h)` | v23.3.0 | → `Int` |
| `Bytes.*` | v23.1.0 | バイトコード読み取り |

---

## 型定義

```favnir
// スタック上の値
type VMVal =
  | VMInt(Int)
  | VMBool(Bool)
  | VMUnit

// デバッグ用
fn vmval_to_string(v: VMVal) -> String

// エントリポイント
fn vm_run(bytecode: Bytes) -> Result<VMVal, String>

// 実行ループ（再帰）
// stack: Int — Mut<List<VMVal>> の opaque handle（Type::Unknown が Int に unify されるため宣言は Int）
fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String>
```

---

## 設計上の注意点

| # | 注意点 | 対応 |
|---|---|---|
| 1 | `Mut.push` は `Result<Unit, String>` を返す | `bind _ <- Mut.push(stack, val)` で sequencing |
| 2 | `Mut.pop` は `Result<Unknown, String>` を返す | `bind r <- Mut.pop(stack)` → `match r { ok(v) => ... err(e) => ... }` |
| 3 | `stack` の型宣言 | `Int` と宣言。型チェッカーは `Unknown`（MutList）が `Int` に unify する（Type::Unknown は全型にマッチ） |
| 4 | `bind _ <-` でユニット捨て | Favnir の bind は単純代入。`_` パターンで戻り値を無視する |
| 5 | `Const(n)` の意味 | Phase 2 では u16 オペランドを定数プールインデックスではなく整数値として push（簡略） |
| 6 | `Result.ok(...)` / `Result.err(...)` | bare `ok`/`err` は LoadGlobal(65535) クラッシュ。必ず名前空間付きで使う |
| 7 | レコードリテラルは型名プレフィックス必須 | `DecodeResult { ... }`（vm.fav Phase 1 と同じルール）※ Phase 2 では使わない |
| 8 | `Mut.push(stack, VMVal)` の型安全性 | checker は `Mut.push` の引数型を検証しない（`Unknown` に unify）。VMVal 値が正しく push されることは vm.rs が保証する |
| 9 | ロードマップとの乖離（意図的） | ロードマップ v23.5 は `VMState` レコード型を示しているが、Favnir の型チェッカーが `Mut<List<T>>` をレコードフィールド型として扱えないため、Phase 2 では引数分離方式を採用する。`VMState` レコードへの移行は Phase 3 以降で検討する |

---

## テスト（5 件）

`fav/src/driver.rs` の `v235000_tests` モジュールに追加。

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_5_0` | Cargo.toml に `version = "23.5.0"` が含まれる | — |
| `vm_fav_phase2_compiles` | vm.fav（Phase 1 + Phase 2）を parse + build_artifact | エラーなし |
| `execute_const_unit` | hex `"0216"` → vm_run → vmval_to_string | `"VMUnit"` |
| `execute_add` | hex `"0103000104002016"` → vm_run → vmval_to_string | `"VMInt(7)"` |
| `changelog_has_v23_5_0` | CHANGELOG.md に `[v23.5.0]` が含まれる | — |

### バイトコード詳細

**`execute_const_unit`**: `"0216"`
- `02` = ConstUnit → push VMUnit
- `16` = Return → pop → VMUnit

**`execute_add`**: `"0103000104002016"`
- `01 03 00` = Const(3) → push VMInt(3)
- `01 04 00` = Const(4) → push VMInt(4)
- `20` = Add → pop 4, pop 3, push VMInt(7)
- `16` = Return → pop → VMInt(7)

---

## 完了条件

- [ ] `type VMVal` + `fn vmval_to_string` が vm.fav に追加される
- [ ] `fn vm_execute` / `fn vm_run` が vm.fav に追加される（11 オペコード対応）
- [ ] `cargo test v235000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1909 件以上合格）
- [ ] `CHANGELOG.md` に v23.5.0 エントリ
- [ ] `benchmarks/v23.5.0.json` 作成済み
- [ ] `site/content/docs/tools/vm-fav.mdx` に Phase 2 セクション追記
