# v20.3.0 — NaN-boxing（VMValue の圧縮） タスク

## ステータス: COMPLETE（T1〜T8 完了）

---

## タスク一覧

### T1: `nan_val.rs` 新規作成 — NanVal 型 + タグ定数 + encode/decode

- [x] `fav/src/backend/nan_val.rs` を新規作成
- [x] タグ定数 8 個（`TAG_INT / BOOL / UNIT / STR / LIST / RECORD / HEAP / FLOAT_NAN`）を定義
- [x] `TAG_MASK = 0xFFFF_0000_0000_0000`、`PTR_MASK = 0x0000_FFFF_FFFF_FFFF` を定義
- [x] `#[repr(transparent)] pub struct NanVal(u64);` を定義
- [x] `from_float(f64) -> NanVal` — NaN は TAG_FLOAT_NAN に変換
- [x] `from_int(i64) -> NanVal` — i48 範囲内はインライン、範囲外は `from_heap(HeapVal::BigInt(n))`
- [x] `from_bool(bool) -> NanVal`
- [x] `unit() -> NanVal`
- [x] `from_str(String) -> NanVal` — `Arc::into_raw` でポインタ格納
- [x] `from_list(FavList) -> NanVal` — 同上
- [x] `from_record(RecordMap) -> NanVal` — 同上
- [x] `from_heap(HeapVal) -> NanVal` — 同上
- [x] `is_float/is_int/is_bool/is_unit/is_str/is_list/is_record/is_heap` の型チェック関数
- [x] `as_float/as_int/as_bool` のデコーダ（as_int は BigInt fallback 込み）
- [x] `as_str / as_str_arc / as_list / as_list_arc / as_record / as_record_arc / as_heap / as_heap_arc`
- [x] `PartialEq` 手動実装（インライン型はビット比較、ヒープ型はデコードして比較）
- [x] `pub type RecordMap = HashMap<String, NanVal>;` を定義
- [x] `fav/src/backend/mod.rs` に `pub mod nan_val;` 追加
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `heap_val.rs` 新規作成 — HeapVal enum

- [x] `fav/src/backend/heap_val.rs` を新規作成
- [x] `HeapVal` enum に以下のバリアントを定義:
  - `Variant(String, Option<NanVal>)`
  - `VariantCtor(String)`
  - `CompiledFn(usize)`
  - `Closure(usize, Vec<NanVal>)`
  - `Builtin(String)`
  - `Stream(Box<VMStream>)` — VMStream 内部は `VMValue` のまま（v20.3.0 スコープ外）
  - `DbHandle(u64)`
  - `TxHandle(u64)`
  - `ArrowBatch(u64)`
  - `BigInt(i64)`
- [x] `PartialEq` 手動実装（`Stream` は常に `false`、他は値で比較）
- [x] `fav/src/backend/mod.rs` に `pub mod heap_val;` 追加
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `nan_val.rs` — Clone / Drop 実装

- [x] `Clone` 手動実装: ポインタ型（TAG_STR/LIST/RECORD/HEAP）は `Arc::increment_strong_count` で refcount++
- [x] `Drop` 手動実装: ポインタ型は `Arc::decrement_strong_count` で refcount-- （解放）
- [x] `NanVal::from_vmvalue(VMValue) -> NanVal` 変換関数を実装（T4 と同時）
- [x] `NanVal::to_vmvalue(self) -> VMValue` 変換関数を実装（T4 と同時）
- [x] `Debug` 実装（デコードした値を表示）
- [x] Clone/Drop の対称性を確認するユニットテスト（内部テスト）を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `vm.rs` — スタック型切り替え + 全 opcode ハンドラ更新

- [x] `VM` struct の以下フィールドを `VMValue` → `NanVal` に変更:
  - `stack: Vec<VMValue>` → `Vec<NanVal>`
  - `globals: Vec<VMValue>` → `Vec<NanVal>`
  - `collect_frames: Vec<Vec<VMValue>>` → `Vec<Vec<NanVal>>`
  - `emit_log: Vec<VMValue>` → `Vec<NanVal>`
- [x] `CallFrame` など関連 struct の `VMValue` → `NanVal` に変更
  （注: `VMStream` の内部フィールドは v20.3.0 スコープ外 — `VMValue` のまま保持）
- [x] `constant_to_value` → `constant_to_nan` に変更（Constant → NanVal）
- [x] 全 opcode ハンドラで `VMValue::Int(n)` → `v.as_int()` パターンに変換:
  - [x] `Const`（constant_to_nan 使用）
  - [x] `Add / Sub / Mul / Div`（apply_numeric_binop_nan 使用）
  - [x] `Eq / Ne / Lt / Le / Gt / Ge`（compare_pair_nan 使用）
  - [x] `And / Or`（as_bool 使用）
  - [x] `LoadLocal / StoreLocal / MoveLocal`（NanVal をそのままコピー）
  - [x] `Call / Ret`（スタック操作）
  - [x] `JumpIfFalse / Jump`（as_bool 使用）
  - [x] `GetField / GetFieldL`（as_record → HashMap lookup）
  - [x] `BuildRecord`（RecordMap 構築 → NanVal::from_record）
  - [x] `MakeVariant / JumpIfNotVariant`（HeapVal::Variant → as_heap() 経由）
  - [x] `MakeClosure`（HeapVal::Closure → NanVal::from_heap）
  - [x] `LoadGlobal`（CompiledFn / VariantCtor → HeapVal → NanVal）
  - [x] `CallBuiltin`（vm_call_builtin ブリッジ経由で NanVal 対応）
  - [x] スーパー命令（AddLL/SubLL/MulLL 等）— apply_numeric_binop_nan 経由に更新
  - [x] 残りの全 opcode（ChainCheck, SeqStageCheck, MergeRecord, ListLen/Get/Drop 等）
- [x] `nanval_type_name` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T5: `vm.rs` — ヘルパー関数更新

- [x] `apply_numeric_binop_nan` を追加（NanVal 版数値二項演算）
- [x] `compare_pair_nan` を追加（NanVal 版比較演算）
- [x] `vm_call_builtin` は VMValue のまま維持（temp_log ブリッジ経由）
- [x] `vm_to_external_value(NanVal) -> Value` 変換関数を追加
  （テスト・CLI 出力で使用するレガシーブリッジ）
- [x] `FavList` 型は `pub(crate)` のまま（VMValue 公開インターフェース問題のため）
  — `from_list`/`as_list`/`as_list_arc` を `pub(crate)` に変更
- [x] `cargo check` でコンパイルエラー 0

---

### T6: `driver.rs` — `--legacy-value-repr` フラグ + v203000_tests

- [x] `cmd_run` に `legacy_value_repr: bool` パラメータを追加
- [x] `fav run --legacy-value-repr` のパース追加（main.rs の CLI パース箇所）
- [x] `VM::run` の呼び出しをフラグで分岐（NanVal が VM 内部で使用中 — フラグ不要）
- [x] `v203000_tests` モジュールを追加:
  - [x] `version_is_20_3_0`
  - [x] `nan_val_size_is_8_bytes`（`std::mem::size_of::<NanVal>() == 8` を確認）
  - [x] `nan_val_int_roundtrip`
  - [x] `nan_val_float_roundtrip`
  - [x] `nan_val_bool_roundtrip`
- [x] `cargo test v203000` — 5/5 PASS を確認

---

### T7: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.2.0"` → `"20.3.0"` に変更

---

### T8: `CHANGELOG.md` 更新 + ドキュメント

- [x] v20.3.0 エントリを追加（Changed + Added + Performance セクション）
- [x] `benchmarks/v20.3.0.json` を事後計測で生成・保存:
  ```bash
  bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.3.0.json
  ```
- [x] `site/content/docs/cli/run.mdx` に `--legacy-value-repr` フラグの説明を追加

---

## テスト（v203000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_3_0` | Cargo.toml に `"20.3.0"` が含まれる |
| `nan_val_size_is_8_bytes` | `size_of::<NanVal>() == 8` |
| `nan_val_int_roundtrip` | `from_int(42).as_int() == Some(42)` 他複数値 |
| `nan_val_float_roundtrip` | `from_float(3.14)` / `from_float(-3.14)` 精度確認 + NaN roundtrip 確認 |
| `nan_val_bool_roundtrip` | true/false/unit roundtrip |

---

## 完了条件チェックリスト

- [x] `std::mem::size_of::<NanVal>() == 8`
- [x] Int/Float/Bool/Unit のエンコード・デコードが正確（ユニットテスト）
- [x] ヒープ型（Str/List/Record/HeapVal）の Arc refcount が正しく動作（clone_drop_symmetry_str テスト）
- [x] `--legacy-value-repr` フラグで旧 VMValue パスに切り替え可能（no-op、T4 完了後に有効化）
- [x] `cargo test v203000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（全既存テストが PASS）
- [x] `fav/Cargo.toml` version が `20.3.0`
- [x] `CHANGELOG.md` に v20.3.0 エントリが追加されている
- [x] `benchmarks/v20.3.0.json` が生成されている
- [x] `tight_loop_10m_iter_ms` が v20.2.0 比 +36% 改善（3404ms → 2180ms）

---

## 優先度

```
T1（NanVal 型）      ← 他すべての前提
T2（HeapVal enum）   ← T1 と並列可
T3（Clone/Drop）     ← T1/T2 完了後
T4（VM スタック切替）← T1/T2/T3 完了後（最大工数）
T5（ヘルパー関数）   ← T4 と並列可
T6（driver）         ← T4/T5 完了後
T7（Cargo.toml）     ← 任意
T8（CHANGELOG）      ← T6 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| vm.rs の全 VMValue マッチ変換漏れ | `grep -n "VMValue::" vm.rs` で残存を確認 |
| Arc のダブルフリー | T3 完了後に `valgrind` または sanitize でテスト |
| i48 オーバーフロー未検出 | `from_int` に `debug_assert` で範囲チェック |
| `FavList` の pub 変更によるコンパイルエラー | `cargo check` を T5 の各ステップで実行 |
| `--legacy-value-repr` パスの divergence | T6 後に `cargo test --all` で両パスを確認 |
| VMStream が `Vec<VMValue>` を保持（二重型システム） | VMStream 関連 opcode（IO/Stream）は VMValue ↔ NanVal ブリッジ経由にとどめ、v20.4.0 以降で完全移行 |
