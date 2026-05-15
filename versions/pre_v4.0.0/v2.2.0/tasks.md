# Favnir v2.2.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.2.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.2.0` に更新

---

## Phase 1 — variant 大文字小文字の正規化

### `src/middle/compiler.rs`

- [x] `compile_pattern` 内の `Pattern::Variant` アームを修正
  - [x] `"Ok"` → `"ok"` の正規化を追加
  - [x] `"Err"` → `"err"` の正規化を追加
  - [x] `"Some"` → `"some"` の正規化を追加
  - [x] `"None"` → `"none"` の正規化を追加
  - [x] ユーザー定義 ADT（その他の名前）は変換しないことを確認

### 動作確認

- [x] `Result.ok(5) |> match { Ok(v) => v  Err(_) => 0 }` が `Int(5)` を返す
- [x] `Result.err("x") |> match { Ok(v) => v  Err(_) => -1 }` が `Int(-1)` を返す
- [x] `Option.some(42) |> match { Some(v) => v  None => 0 }` が `Int(42)` を返す
- [x] `Option.none() |> match { Some(v) => v  None => -1 }` が `Int(-1)` を返す
- [x] 既存テスト `test_match_variant_with_payload` が引き続き通る（ユーザー ADT 影響なし）

---

## Phase 2 — pipe match エンドツーエンドテスト

### `src/backend/vm_stdlib_tests.rs`

- [x] `test_pipe_match_ok`: `Result.ok(5) |> match { Ok(v) => v  Err(_) => 0 }` → `Int(5)`
- [x] `test_pipe_match_err`: `Result.err("oops") |> match { Ok(v) => v  Err(_) => -1 }` → `Int(-1)`
- [x] `test_pipe_match_option_some`: `Option.some(42) |> match { Some(v) => v  None => 0 }` → `Int(42)`
- [x] `test_pipe_match_option_none`: `Option.none() |> match { Some(v) => v  None => -1 }` → `Int(-1)`
- [x] `test_pipe_match_chained`: `fn double(n) |> match { Ok(v) => v  Err(_) => 0 }` → `Int(14)`

---

## Phase 3 — pattern guard テスト補完

### `src/backend/vm_stdlib_tests.rs`

- [x] `test_pattern_guard_fallthrough`: `match 15 { n where n > 20 => "big"  n where n > 10 => "medium"  _ => "small" }` → `Str("medium")`
- [x] `test_pattern_guard_all_fail`: 全ガード不成立 → `Str("small")`
- [x] `test_pattern_guard_record`: `match u { { age } where age >= 18 => "adult"  _ => "minor" }` で adult
- [x] `test_pattern_guard_record_minor`: age < 18 のケースで minor
- [x] `test_pattern_guard_compound_and`: `where n >= 18 && n < 65` → `Str("working-age")`

### `src/middle/checker.rs`

- [x] `test_guard_non_bool_compound`: `where n + 1` で E027

---

## Phase 4 — テスト・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.1.0 の 556 → 567）
- [x] `fav run` で `result |> match { Ok(v) => v  Err(_) => 0 }` が動くことを確認
- [x] `fav check` で E027 が適切に出ることを確認

### ドキュメント作成

- [x] `versions/v2.2.0/langspec.md` を作成
  - [x] `|> match {}` 構文の説明（パーサー脱糖の説明を含む）
  - [x] `where` ガードの構文・優先順位
  - [x] `Ok` / `Err` / `Some` / `None` の大文字パターン名サポートを記載
  - [x] E027 エラーコードの説明
  - [x] 使用例（パイプラインでの `|> match`）

---

## 完了条件チェック

- [x] `result |> match { Ok(v) => v  Err(_) => 0 }` が動く
- [x] `result |> match { Ok(v) => v  Err(_) => -1 }` で Err のケースが動く
- [x] `opt |> match { Some(v) => v  None => 0 }` が動く
- [x] `match x { n where n > 10 => "big"  _ => "small" }` が動く
- [x] `match u { { age } where age >= 18 => "adult"  _ => "minor" }` が動く
- [x] ガード不成立時に次アームへフォールスルーする
- [x] ガード式が Bool でない場合に E027 が出る
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.2.0"`
- [x] `versions/v2.2.0/langspec.md` 作成済み
