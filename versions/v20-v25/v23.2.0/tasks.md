# v23.2.0 — ビット演算 タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1898 passed / 0 failed（v232000_tests 5/5 PASS）

---

## タスク一覧

### T1: `fav/src/frontend/lexer.rs` — 16 進数リテラル対応

- [x] **事前確認**: `grep -n "lex_number\|'0'..='9'" fav/src/frontend/lexer.rs | head -10` で呼び出し位置と `lex_number` 冒頭を確認
- [x] `lex_number()` の先頭に `0x`/`0X` チェックを追加（plan.md T1 のコードに従う）
  - `next_token()` は advance しないため、`lex_number()` 入場時に `'0'` は pos にある
  - `self.peek() == Some('0') && matches!(self.peek2(), Some('x') | Some('X'))` → 16 進モード
  - `is_ascii_hexdigit()` で桁を収集 → `i64::from_str_radix(&hex, 16)` → `TokenKind::Int`
  - 桁 0 文字（`0x` のみ）→ `LexError`
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/vm.rs` — vm_call_builtin に 6 アーム追加

- [x] **事前確認**: `grep -n '"Int\.bnot"' fav/src/backend/vm.rs | head -5` で挿入位置を確認
- [x] `"Int.bnot"` ブロック直後に 6 つの flat literal アームを追加（plan.md T2 のコードに従う）
  - `args.into_iter().next()` パターンを使う（`pop()` は使わない）
  - `"Int.bit_and"` / `"Int.bit_or"` / `"Int.bit_xor"` — 2 引数、`into_iter()`
  - `"Int.bit_not"` — 1 引数、`into_iter()`
  - `"Int.shift_left"` / `"Int.shift_right"` — シフト量 `0..=63` 範囲チェック付き（`Err` を返す）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/middle/checker.rs` — builtin_ret_ty 更新

- [x] **事前確認**: `grep -n '"bnot"\|"band"\|// Math' fav/src/middle/checker.rs | head -5` で挿入位置確認
- [x] `("Int", "bnot") | ("Int", "to_byte")` アームの直後（`// Math` の前）に 6 エントリを追加（plan.md T3 のコードに従う）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/driver.rs` — `#[ignore]` + `v232000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_1_0\|mod v231000_tests\|mod v232000_tests" fav/src/driver.rs | head -5`
- [x] **T5-1（Cargo.toml バージョン更新）より前に実施**: `v231000_tests::version_is_23_1_0` に `#[ignore]` を追加
- [x] `v232000_tests` モジュールを `v231000_tests` の直後に追加（5 件、plan.md T4-2 のコードに従う）
  - テストは `Lexer → Parser → build_artifact → exec_artifact_main(&artifact, None)` パターン
  - `version_is_23_2_0`
  - `int_bit_and_with_hex_literals`
  - `int_shift_left_correct`
  - `int_bit_not_is_minus_one`
  - `changelog_has_v23_2_0`
- [x] `cargo test v232000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1898 件合格）を確認

---

### T5: Cargo.toml + CHANGELOG + benchmarks + MDX

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "23.1.0"` → `"23.2.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.1.0 エントリの上）に v23.2.0 エントリを追加（plan.md T5-2）
- [x] `benchmarks/v23.2.0.json` を新規作成（test_count: 1898）
- [x] `site/content/docs/primitives/` ディレクトリを新規作成し `bit-ops.mdx` を配置（plan.md T5-4）
- [x] `cargo test v232000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1898 件合格）を再確認

---

## テスト一覧（v232000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_2_0` | Cargo.toml に `version = "23.2.0"` が含まれる |
| `int_bit_and_with_hex_literals` | `Int.bit_and(0xFF, 0x0F)` → `15` |
| `int_shift_left_correct` | `Int.shift_left(1, 4)` → `16` |
| `int_bit_not_is_minus_one` | `Int.bit_not(0)` → `-1`（i64 全ビット反転）|
| `changelog_has_v23_2_0` | CHANGELOG.md に `[v23.2.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `v231000_tests::version_is_23_1_0` に `#[ignore]` が追加済み（T5-1 より前）
- [x] `lexer.rs` が `0xFF` 等の 16 進リテラルを `TokenKind::Int` にパースできる
- [x] `lexer.rs` が `0x`（桁なし）で `LexError` を返す
- [x] `vm_call_builtin` に `"Int.bit_and"` 〜 `"Int.shift_right"` の 6 アームが追加される（`into_iter()` パターン）
- [x] `Int.shift_left` / `Int.shift_right` がシフト量範囲外（< 0 または >= 64）で `Err` を返す（パニックしない）
- [x] `checker.rs` の `builtin_ret_ty` に 6 エントリが追加される
- [x] `compiler.rs` の変更なし（`"Int"` namespace は既に登録済み）
- [x] `cargo test v232000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1898 件合格）
- [x] `CHANGELOG.md` に v23.2.0 エントリ
- [x] `benchmarks/v23.2.0.json` 作成済み
- [x] `site/content/docs/primitives/bit-ops.mdx` 作成済み（`bit_not` の i64 挙動を明示）

---

## 優先度

```
T1（lexer.rs）  ← 最初（hex リテラル。テストコードが 0xFF を使用）
T2（vm.rs）     ← T1 完了後（最大タスク）
T3（checker.rs）← T2 と並列可
T4（driver.rs） ← T2〜T3 完了後、T5-1 より前に #[ignore] を実施
T5（docs）      ← T4 完了後（#[ignore] 確認後にバージョン更新）
```
