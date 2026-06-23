# v23.4.0 — vm.fav Phase 1（バイトコードデコード）タスク

## ステータス: COMPLETE（2026-06-22）

---

## タスク一覧

### T0: `fav/src/backend/vm.rs` — `Bytes.read_u16_le` / `Bytes.read_u24_le` 追加

- [x]**事前確認**: `grep -n "Bytes.read_u32\|read_u32" fav/src/backend/vm.rs | head -5` で挿入位置確認
- [x]`"Bytes.read_u32"` アームの直後に `"Bytes.read_u16_le"` / `"Bytes.read_u24_le"` を追加（plan.md T0-1 のコードに従う）
  - `read_u16_le`: `arc[off] | arc[off+1] << 8`（LE）
  - `read_u24_le`: `arc[off] | arc[off+1] << 8 | arc[off+2] << 16`（LE）
  - 既存の `read_u16`（BE）は変更しないこと
- [x]`cargo check --bin fav` でコンパイルエラー 0 を確認

---

### T1: `fav/self/vm.fav` 新規作成

- [x]**事前確認**: `ls fav/self/` で既存ファイルを確認（`vm.fav` が未存在であること）
- [x]**事前確認（or パターン）**: `grep -n "Pattern::Or\|parse_or_pattern\|Pipe.*pattern\|Pat.*Or" fav/src/frontend/parser.rs | head -10` で or パターンのサポートを確認
- [x]`fav/self/vm.fav` を新規作成（plan.md T1-1 のコードに従う）
  - `type Opcode` — 27 バリアント（Const〜Unknown）
  - `type DecodeResult` — `{ op: Opcode, next_pc: Int }`
  - `fn decode_byte_no_operand(byte: Int, pc: Int) -> Result<DecodeResult, String>`
  - `fn decode_byte_with_u16_le(bytes: Bytes, byte: Int, pc: Int) -> Result<DecodeResult, String>`（`Bytes.read_u16_le` を使用）
  - `fn decode_opcode(bytes: Bytes, pc: Int) -> Result<DecodeResult, String>`
  - `fn opcode_to_string(op: Opcode) -> String`
- [x]or パターンが不可の場合: `decode_opcode` の match を個別アームに展開
- [x]**Bytes.from_hex は Result<Bytes, String>**: テストコード・vm.fav 内部で使う場合は `match hex_r { ok(b) => ... err(e) => ... }` で展開すること
- [x]**事後確認**: `cargo check --bin fav` でコンパイルエラー 0（vm.fav はテストで include_str! されるため）

---

### T2: `fav/src/driver.rs` — `#[ignore]` + `v234000_tests` 追加

- [x]**事前確認**: `grep -n "fn version_is_23_3_0\|mod v233000_tests\|mod v234000_tests" fav/src/driver.rs | head -5`
- [x]**T3-1（Cargo.toml バージョン更新）より前に実施**: `v233000_tests::version_is_23_3_0` に `#[ignore]` を追加
- [x]`v234000_tests` モジュールを `v233000_tests` の直後に追加（5 件、plan.md T2-2 のコードに従う）
  - `version_is_23_4_0`
  - `vm_fav_file_exists`（include_str! で読み込みテスト）
  - `vm_fav_compiles`（parse + build_artifact）
  - `decode_const_opcode`（bytes=[0x01,0x03,0x00] → `"Const(3)"`）
  - `changelog_has_v23_4_0`
- [x]`cargo test v234000 --bin fav` — 5/5 PASS を確認
- [x]`cargo test --bin fav` — リグレッションなし（1905 件以上合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意（T3-1 より前）**: T2-1 の `#[ignore]` 追加が完了してから `Cargo.toml` を更新すること。

- [x]**事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリを確認
- [x]`fav/Cargo.toml` の `version = "23.3.0"` → `"23.4.0"` に変更
- [x]`CHANGELOG.md` 先頭（v23.3.0 エントリの上）に v23.4.0 エントリを追加（plan.md T3-2）
- [x]`benchmarks/v23.4.0.json` を新規作成（plan.md T3-3、test_count は実行後に確定）
- [x]`site/content/docs/tools/vm-fav.mdx` を新規作成（plan.md T3-4）
- [x]`cargo test v234000 --bin fav` — 最終確認 5/5 PASS
- [x]`cargo test --bin fav` — リグレッションなし再確認

---

## テスト一覧（v234000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_4_0` | Cargo.toml に `version = "23.4.0"` が含まれる |
| `vm_fav_file_exists` | include_str!("../self/vm.fav") が成功し空でない |
| `vm_fav_compiles` | vm.fav を parse + build_artifact し、エラーなく完了 |
| `decode_const_opcode` | bytes=[0x01,0x03,0x00] → decode_opcode → `"Const(3)"` |
| `changelog_has_v23_4_0` | CHANGELOG.md に `[v23.4.0]` が含まれる |

---

## 完了条件チェックリスト

- [x]`v233000_tests::version_is_23_3_0` に `#[ignore]` が追加済み（T3-1 より前）
- [x]`Bytes.read_u16_le` / `Bytes.read_u24_le` が vm.rs に追加される（LE バイトコード読み取り）
- [x]`fav/self/vm.fav` が新規作成される
- [x]`type Opcode` — 27 バリアント（Const〜Unknown）が定義される
- [x]`type DecodeResult` — `{ op: Opcode, next_pc: Int }` が定義される
- [x]`fn decode_byte_no_operand` / `fn decode_byte_with_u16_le` ヘルパーが定義される
- [x]`fn decode_opcode` / `fn opcode_to_string` が定義される
- [x]`cargo test v234000 --bin fav` — 5/5 PASS
- [x]`cargo test --bin fav` — リグレッションなし（1905 件以上合格）
- [x]`CHANGELOG.md` に v23.4.0 エントリ
- [x]`benchmarks/v23.4.0.json` 作成済み
- [x]`site/content/docs/tools/vm-fav.mdx` 作成済み

---

## 優先度

```
T0（Bytes.read_u16_le）  ← 最初（vm.fav より前）
T1（vm.fav 作成）        ← T0 完了後
T2-1（#[ignore]）        ← T3-1 より前
T2-2（tests）            ← T0 + T1 完了後
T3-1（version）          ← T2-1 完了後
T3-2〜4（docs）          ← T3-1 完了後
```

---

## 実装時の注意事項

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `let` キーワード不在 | `bind x <- expr` を使う（`let x = expr` は構文エラー） |
| 2 | `Bytes.get` / `Bytes.read_u16_le` / `Bytes.from_hex` は Result | `bind r <- ...\nmatch r { ok(v) => ... err(e) => err(e) }` パターン |
| 3 | `Bytes.read_u16` は BE（既存テストで確認済み） | vm.fav では `Bytes.read_u16_le`（T0 で追加）を使う |
| 4 | or パターン未確認 | parser.rs で確認後、不可なら個別アームに展開 |
| 5 | `type Opcode` のバリアントに `And`/`Or` を含む | Favnir のキーワードでないため OK（lexer.rs で未定義） |
| 6 | `f"Const({idx})"` のフォーマット | Favnir f-string は Int を自動展開 |
| 7 | `#[ignore]` 追加順序 | Cargo.toml 更新前に必ず追加すること |

---

## 実装完了メモ（2026-06-22）

- 1909 tests pass（0 failures）
- v234000_tests: 5/5 PASS
- **重要な発見**: Favnir で Result 値を構築する式位置では `Result.ok(...)` / `Result.err(...)` を使う必要がある。
  bare `ok(...)` / `err(...)` は globals に未登録のため、コンパイラが `LoadGlobal(65535)` を生成し、実行時クラッシュ（"global index out of bounds"）となる。
- record リテラルは型名プレフィックスが必須: `DecodeResult { op: ..., next_pc: ... }` (bare `{ ... }` は Parser エラー)
- or パターン（`0x01 | 0x10 | ...`）は実際に使用可能だが、個別アームに展開して実装


---

## コードレビュー対応（2026-06-22）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [HIGH] | `decode_byte_with_u16_le` `_` アームの `next_pc: pc + 1` バグ（正しくは `pc + 3`） | `Result.err(f"...unknown 3-byte opcode: {byte}")` に変更して問題を回避 |
| [HIGH] | `decode_byte_with_u16_le` `_` アームが到達不能 + 将来バグのリスク | 上記と同時に `Result.err(...)` に変更（防御的エラー化） |
| [MED] | `vm_fav_compiles` テストが実行を検証しない | Phase 2 で実行テスト追加予定（現状は `decode_const_opcode` が実行カバレッジを担保） |
| [LOW] | `Unknown` バリアントの Phase 2 仕様コメント不足 | Phase 2 spec で対応 |
| [LOW] | OOB エッジケーステスト不足 | Phase 2 前に追加予定 |

