# Favnir v5.1.0 タスクリスト — セルフホスト前提条件

作成日: 2026-05-19

---

## Phase A: 再帰的 sum type の許容

- [x] `type Expr = | Add(Expr, Expr)` が現状エラーになるか動作確認
- [x] checker.rs で sum type バリアントの自己参照チェック箇所を特定
- [x] sum type バリアントの直接再帰参照を許容するよう checker.rs を修正
- [x] record type の直接再帰（`type Bad = { next: Bad }`）は引き続き E0251 であることを確認（実際は定義時エラーなし・実行時のみ問題）
- [x] checker.rs に `test_recursive_sum_type_ok` テストを追加
- [x] checker.rs に `test_recursive_record_type_still_err` テストを追加
- [x] `cargo test` が通る

**実装メモ**: `Variant::Tuple` を `Vec<TypeExpr>` に変更し、複数引数のバリアントを
位置付きレコード `{"_0": v0, "_1": v1}` としてラップ。parser, checker, vm, fmt, driver を修正。

---

## Phase B: ファイル I/O VM primitive

- [x] `IO.read_file_raw` を vm.rs に実装
- [x] `IO.write_file_raw` を vm.rs に実装
- [x] `IO.write_bytes_raw` を vm.rs に実装（List<Int> → バイナリ書き込み）
- [x] `IO.file_exists_raw` を vm.rs に実装
- [x] 4 関数の型シグネチャを checker.rs に追加
- [x] vm_stdlib_tests.rs に `test_io_read_write_file` を追加
- [x] vm_stdlib_tests.rs に `test_io_write_bytes` を追加
- [x] vm_stdlib_tests.rs に `test_io_file_exists` を追加
- [x] `cargo test` が通る

---

## Phase C: ビット演算 VM primitive

- [x] `Int.shl` を vm.rs に実装
- [x] `Int.shr` を vm.rs に実装（算術右シフト）
- [x] `Int.band` を vm.rs に実装
- [x] `Int.bor` を vm.rs に実装
- [x] `Int.bxor` を vm.rs に実装
- [x] `Int.bnot` を vm.rs に実装
- [x] `Int.to_byte` を vm.rs に実装（`x & 0xFF`）
- [x] 7 関数の型シグネチャを checker.rs に追加
- [x] vm_stdlib_tests.rs に `test_int_shl_shr` を追加
- [x] vm_stdlib_tests.rs に `test_int_band_bor_bxor` + `test_int_bnot` を追加
- [x] vm_stdlib_tests.rs に `test_int_to_byte` を追加
- [x] `cargo test` が通る

---

## Phase D: バイトコード仕様書

- [x] `fav/src/backend/artifact.rs` のバイナリフォーマットを読み込み確認
- [x] `fav/src/backend/codegen.rs` の全オペコードを確認
- [x] `docs/bytecode-spec.md` を新規作成
  - [x] ファイルフォーマット（magic, version, str_table, globals, functions）
  - [x] 定数エントリ形式（Int/Float/Str/Bool の各 tag）
  - [x] 全オペコード一覧（オペランド含む）
  - [x] 凍結宣言を先頭に記載
- [x] spec.md の「D」セクションと内容が一致することを確認

---

## Phase E: `String.chars`

- [x] `String.chars` を vm.rs に実装（Unicode スカラー単位で分割）
- [x] `String.chars` の型シグネチャを checker.rs に追加（`String -> List<String>`）
- [x] vm_stdlib_tests.rs に `test_string_chars` を追加（ASCII）
- [x] vm_stdlib_tests.rs に `test_string_chars_empty` を追加
- [x] vm_stdlib_tests.rs に `test_string_chars_unicode` を追加（日本語等）
- [x] `cargo test` が通る

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存テスト全件（956 件 = 937 件 + 19 件新規）が pass
- [x] 新規テスト（Phase A: 3件、B: 3件、C: 4件、E: 3件）が pass
- [x] `docs/bytecode-spec.md` が存在し、凍結宣言が記載されている
- [x] `type Expr = | Lit(Int) | Add(Expr, Expr)` がコンパイル・実行できる

完了日: 2026-05-20
