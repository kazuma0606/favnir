# v23.2.0 — ビット演算 仕様書

## 概要

`Int.bit_and / bit_or / bit_xor / bit_not / shift_left / shift_right` の 6 関数と、
16 進数リテラル（`0xFF`）のパース対応を追加する。

vm.fav のバイトコードデコード実装（v23.4〜v23.8）で opcode/operand の抽出に直接使う。

---

## 背景と動機

- `Int.band` / `Int.bor` / `Int.bxor` / `Int.bnot` / `Int.shl` / `Int.shr` は v5.1.0 に実装済み
- 命名が vm.fav の意図（読みやすさ・明快さ）に合わない
- ロードマップ（roadmap-v23.1-v24.0.md）では `bit_` prefix / `shift_left/right` を明示

→ 旧名は維持（後方互換）し、新名を追加する。

---

## 追加関数（6 件）

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `Int.bit_and` | `Int, Int -> Int` | ビット AND |
| `Int.bit_or` | `Int, Int -> Int` | ビット OR |
| `Int.bit_xor` | `Int, Int -> Int` | ビット XOR |
| `Int.bit_not` | `Int -> Int` | ビット NOT（1 の補数）|
| `Int.shift_left` | `Int, Int -> Int` | 左シフト（シフト量 0〜63、範囲外は `Err`）|
| `Int.shift_right` | `Int, Int -> Int` | 算術右シフト（符号拡張、シフト量 0〜63、範囲外は `Err`）|

---

## 16 進数リテラル

`0x` または `0X` プレフィックス付き整数リテラルを `TokenKind::Int(i64)` として解析する。
パーサーへの変更は不要（既存の `TokenKind::Int` がそのまま使われる）。

```favnir
bind a <- Int.bit_and(0xFF, 0x0F)     // 15
bind b <- Int.bit_or(0xF0, 0x0F)      // 255
bind c <- Int.bit_xor(0xFF, 0x0F)     // 240
bind d <- Int.bit_not(0x00)           // -1（i64 全ビット反転 = 0xFFFFFFFFFFFFFFFF）
bind e <- Int.shift_left(1, 4)        // 16
bind f <- Int.shift_right(256, 4)     // 16

// opcode デコード（vm.fav で使う典型例）
bind opcode  <- Int.bit_and(Int.shift_right(word, 24), 0xFF)
bind operand <- Int.bit_and(word, 0x00FFFFFF)
```

---

## Int.bit_not の挙動（i64）

Favnir の `Int` は i64（64 ビット符号付き整数）。
`bit_not(0x00)` → `!0i64` → `-1`（`0xFFFFFFFFFFFFFFFF`）。

> ロードマップ例（roadmap-v23.1-v24.0.md）では `bit_not(0x00)` → `0xFFFFFFFF` と記載されているが、
> 実態は i64 なので `-1` を返す。ドキュメント（bit-ops.mdx）にもこの差異を明示すること。

---

## シフト量の範囲

`shift_left` / `shift_right` のシフト量（第 2 引数）が `0..=63` の範囲外の場合、
ランタイムエラー（`Err(...)` を返す）とする。パニックしない。

既存の `Int.shl` / `Int.shr` はこのガードを持たないが、
`Int.shift_left` / `Int.shift_right` では安全のため追加する。

---

## 実装方針

### T1: `fav/src/frontend/lexer.rs` — 16 進数リテラル

`lex_number()` を修正する。`next_token()` は advance しないため、`lex_number()` 入場時に `'0'` は pos にある。

- `self.peek() == Some('0') && matches!(self.peek2(), Some('x') | Some('X'))` → 16 進モード
- `'0'..='9'`, `'a'..='f'`, `'A'..='F'` を収集（`is_ascii_hexdigit()`）
- `i64::from_str_radix(&hex_digits, 16)` でパース → `TokenKind::Int`
- 16 進桁が 0 文字（`0x` のみ）→ `LexError`

### T2: `fav/src/backend/vm.rs` — vm_call_builtin に 6 アーム追加

既存の `"Int.bnot"` ブロック直後に追加。
`args.into_iter().next()` パターンを使う（`pop()` ではない — 既存 `Int.band` / `Int.shl` と同じ）。

### T3: `fav/src/middle/checker.rs` — builtin_ret_ty 更新

既存の `("Int", "bnot") | ("Int", "to_byte")` アームの直後に 6 エントリを追加。
compiler.rs の変更は不要（`"Int"` namespace は既に登録済み、compiler.rs 168 行目）。

### T4: `fav/src/driver.rs` — テスト追加

- `v231000_tests::version_is_23_1_0` に `#[ignore]` を追加（T5 の Cargo.toml 変更前に実施）
- `v232000_tests` モジュールを `v231000_tests` の直後に追加（テスト 5 件）
- テストは `Lexer → Parser → build_artifact → exec_artifact_main(&artifact, None)` パターン

### T5: Cargo.toml / CHANGELOG / benchmarks / MDX

- `fav/Cargo.toml`: `23.1.0` → `23.2.0`
- `CHANGELOG.md`: v23.2.0 エントリ追加
- `benchmarks/v23.2.0.json`: 新規作成
- `site/content/docs/primitives/` ディレクトリを新規作成し `bit-ops.mdx` を配置

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

## スコープ外（今後のバージョン）

- `Mut<T>` 可変コレクション → v23.3.0
- vm.fav の実装 → v23.4〜v23.8
- 旧名（`Int.band` 等）の削除 → 未定（後方互換のため保持）
