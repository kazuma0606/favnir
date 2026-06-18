# v20.2.0 — スーパー命令（Superinstruction） タスク

## ステータス: DONE

---

## タスク一覧

### T1: `codegen.rs` — Opcode enum に 10 variants 追加

- [x] `RefinementAssert = 0x63,` の直後に `AddLL` (0xA0) 〜 `MoveLocal` (0xA9) を追加
  - `AddLL = 0xA0` — stack[base+a] + stack[base+b]
  - `SubLL = 0xA1` — stack[base+a] - stack[base+b]
  - `MulLL = 0xA2` — stack[base+a] * stack[base+b]
  - `AddLC = 0xA3` — stack[base+a] + constants[k]
  - `SubLC = 0xA4` — stack[base+a] - constants[k]
  - `LeLC = 0xA5` — stack[base+a] <= constants[k]
  - `LtLC = 0xA6` — stack[base+a] < constants[k]
  - `EqLC = 0xA7` — stack[base+a] == constants[k]
  - `GetFieldL = 0xA8` — stack[base+a].field[str_table[f]]
  - `MoveLocal = 0xA9` — stack[base+dst] = stack[base+src]
- [x] `cargo check` でコンパイルエラー 0 を確認（この時点では VM dispatch なし → `_ =>` が吸収）

---

### T2: `codegen.rs` — emit_expr / emit_stmt 融合パターン追加

- [x] `emit_expr::IRExpr::BinOp` を書き換え:
  - `Local(a) + Local(b)` で Add/Sub/Mul → AddLL/SubLL/MulLL を出力
  - `Local(a) + Lit(Int(k))` で Add/Sub/LtEq/Lt/Eq → AddLC/SubLC/LeLC/LtLC/EqLC を出力
  - それ以外はフォールバック（既存の emit_expr + opcode）
- [x] `emit_expr::IRExpr::FieldAccess` を書き換え:
  - `FieldAccess(Local(a), field)` → GetFieldL(a, f_idx) を出力
  - それ以外はフォールバック（既存の emit_expr(obj) + GetField）
- [x] `emit_stmt::IRStmt::Bind` を書き換え:
  - `Bind(dst, Local(src))` → MoveLocal(src, dst) を出力
  - それ以外はフォールバック（既存の emit_expr(expr) + StoreLocal）
- [x] `cargo check` でコンパイルエラー 0 を確認

---

### T3: `codegen.rs` — remap_string_operands 更新

- [x] `_ => break` の直前に以下を追加:
  - 9 opcodes（AddLL/SubLL/MulLL/AddLC/SubLC/LeLC/LtLC/EqLC/MoveLocal）: `ip += 5`
  - GetFieldL: `remap_u16_at(code, ip + 3, str_remap); ip += 5`
- [x] `cargo check` でコンパイルエラー 0 を確認

---

### T4: `vm.rs` — resume ループに 10 opcode の dispatch 追加

- [x] `RefinementAssert` ハンドラの直後（または末尾の `_ =>` 直前）に追加:
  - `AddLL` — read(a, b), push `stack[base+a] + stack[base+b]`（apply_numeric_binop 使用）
  - `SubLL` — read(a, b), push `stack[base+a] - stack[base+b]`
  - `MulLL` — read(a, b), push `stack[base+a] * stack[base+b]`
  - `AddLC` — read(a, k_idx), push `stack[base+a] + constants[k_idx]`（constant_to_value 使用）
  - `SubLC` — read(a, k_idx), push `stack[base+a] - constants[k_idx]`
  - `LeLC` — read(a, k_idx), push `compare_pair((va, vk), |a,b| a<=b, ...)` （compare_pair 再利用）
  - `LtLC` — read(a, k_idx), push `compare_pair((va, vk), |a,b| a<b, ...)`
  - `EqLC` — read(a, k_idx), push `Bool(va == vk)` （VMValue::PartialEq、vmvalue_eq は不存在）
  - `GetFieldL` — read(a, f_idx), match Record/Builtin/VariantCtor（GetField と同一分岐）
  - `MoveLocal` — read(src, dst), `stack[base+dst] = stack[base+src]`（push/pop なし）
- [x] `EqLC` は `va == vk`（VMValue::PartialEq）で実装（`vmvalue_eq` は存在しないため使わない）
- [x] `GetFieldL` の match に `Builtin` / `VariantCtor` 分岐を含む（GetField と同一）
- [x] `cargo check` でコンパイルエラー 0 を確認

---

### T5: `driver.rs` — `v202000_tests` 追加

- [x] `v201000_tests` モジュールの `}` 直後に `v202000_tests` モジュールを追加
  - `compile_and_run_si(name, src)` ヘルパー関数を定義
  - `version_is_20_2_0` テスト
  - `addll_opcode_value` テスト（`Opcode::AddLL as u8 == 0xA0`）
  - `getfieldl_opcode_value` テスト（`Opcode::GetFieldL as u8 == 0xA8`）
  - `superinsn_add_local_local` テスト（`add(3, 4) == 7`）
  - `superinsn_tight_loop` テスト（`tight_loop(100, 0) == 5050`）
- [x] `cargo test v202000` — 5/5 PASS を確認

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.1.0"` → `"20.2.0"` に変更
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T7: `fav/CHANGELOG.md` 更新

- [x] v20.2.0 エントリを追加:
  ```
  ## [v20.2.0] — 2026-06-18 — スーパー命令（Superinstruction）

  ### Added
  - `Opcode::AddLL / SubLL / MulLL / AddLC / SubLC / LeLC / LtLC / EqLC / GetFieldL / MoveLocal`
    (0xA0〜0xA9) — IR レベルスーパー命令 10 種
  - `emit_expr / emit_stmt` が Local×Local・Local×Int リテラルのパターンで自動融合
  - `GetFieldL` が `FieldAccess(Local(a), field)` を 6→5 bytes に圧縮
  - `MoveLocal` が `Bind(dst, Local(src))` を 6→5 bytes に圧縮

  ### Performance
  - `tight_loop_10m_iter`: ディスパッチ回数削減（+20〜30% 期待）
  - `record_transform_1m`: フィールドアクセスパターン改善（+10〜15% 期待）
  ```

> **site/ MDX**: 新構文・新コマンドの追加なし（VM 内部実装のみ）のため、
> site/content/ への新規 MDX ページ追加は不要。

---

### T8: 事後ベンチマーク計測 — `benchmarks/v20.2.0.json` 生成

- [x] release ビルドで `tight_loop_10m_iter_ms` を計測（v20.0.0 比 +20〜30% 確認）
- [x] `record_transform_1m_ms` を計測（v20.0.0 比 +10〜15% 確認）
- [x] `benchmarks/v20.2.0.json` を保存（ロードマップの「Verify + Document」原則）

---

## テスト（v202000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_2_0` | Cargo.toml に `"20.2.0"` が含まれる |
| `addll_opcode_value` | `Opcode::AddLL as u8 == 0xA0` |
| `getfieldl_opcode_value` | `Opcode::GetFieldL as u8 == 0xA8` |
| `superinsn_add_local_local` | `add(3, 4) == 7`（AddLL の正確性） |
| `superinsn_tight_loop` | `tight_loop(100, 0) == 5050`（SubLC + AddLL + LeLC の連携） |

---

## 完了条件チェックリスト

- [x] `Opcode` enum に `AddLL`〜`MoveLocal`（0xA0〜0xA9）が追加されている
- [x] `emit_expr::BinOp` が 8 パターンのスーパー命令を出力する
- [x] `emit_expr::FieldAccess(Local)` が `GetFieldL` を出力する
- [x] `emit_stmt::Bind(Local)` が `MoveLocal` を出力する
- [x] `remap_string_operands` が新オペコードを正しくスキップ（GetFieldL は str remap）
- [x] `resume` ループが 10 opcode すべてを dispatch する
- [x] `fav/Cargo.toml` version が `20.2.0`
- [x] `cargo test v202000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（全既存テストが PASS）
- [x] `CHANGELOG.md` に v20.2.0 エントリが追加されている
- [x] site/ MDX 追加: 不要（VM 内部実装のみ）
- [x] `benchmarks/v20.2.0.json` が生成されている（事後計測 Verify + Document）

---

## 優先度

```
T1（Opcode 追加）     ← 他すべての前提
T2（emit 融合）       ← T1 完了後
T3（remap 更新）      ← T1 完了後（T2 と並列可）
T4（VM dispatch）     ← T1 完了後（T2/T3 と並列可）
T5（driver テスト）   ← T1〜T4 完了後
T6（Cargo.toml）      ← 任意
T7（CHANGELOG）       ← T5 完了後
```
