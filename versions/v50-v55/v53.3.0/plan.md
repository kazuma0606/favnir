# Plan: v53.3.0 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3167 passed, 0 failed を確認

# E0419 の現状内容を確認
rg -n "E0419" fav/src/error_catalog.rs
# → title が "assert_schema type mismatch" であることを確認

# v52100_tests::assert_schema_type_fail の assert 内容を確認
rg -n "assert_schema_type_fail\|assert_schema type mismatch" fav/src/driver.rs

# v53300_tests が未存在を確認
rg -n "v53300_tests" fav/src/driver.rs  # → 0 件

# v53200_tests の行番号を確認（挿入位置）
rg -n "v53200_tests" fav/src/driver.rs  # → 行番号を特定

# Cargo.toml が 53.2.0 であることを確認
```

---

## ステップ 2: `error_catalog.rs` — E0419 エントリ更新

E0419 エントリの `description` / `example` / `fix` / `suggestion` を以下に置き換える。
**`title` / `code` / `category` は変更しない。**

```rust
// ── E0419: assert_schema 型不一致 (v52.1.0) — suggestion 強化 (v53.3.0) ──
ErrorEntry {
    code: "E0419",
    title: "assert_schema type mismatch",
    category: "runtime",
    description: "assert_schema<T> validates at runtime that a map's fields match schema T. \
                  Validation fails when a required field is missing, a field's type does not match, \
                  or (with --strict-schema) unexpected fields are present. \
                  Each mismatched field is reported individually.",
    example: "type OrderRow = { id: Int  amount: Float  status: String }\n\
              // runtime: { id: \"abc\", amount: 99.0, status: \"ok\" }\n\
              // E0419: assert_schema type mismatch\n\
              //   expected: { id: Int, amount: Float, status: String }\n\
              //   got:      { id: \"abc\", amount: 99.0, status: \"ok\" }\n\
              //   field `id`: expected Int but got String\n\
              //   help: use Int.parse(row[\"id\"]) to convert",
    fix: "Ensure the input map contains fields matching the schema T. \
          Check each mismatched field and convert to the expected type before calling assert_schema.",
    suggestion: Some("For field type mismatches, use type conversion functions: \
                      Int.parse() for String to Int, Float.from_int() for Int to Float. \
                      Run `fav explain --error E0419` for field-level diff examples."),
},
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `driver.rs` — `v53300_tests` 追加

`v53200_tests` モジュールの直前に `v53300_tests` を追加:

```rust
// -- v53300_tests (v53.3.0) -- DX × DQ 統合: assert_schema suggestion --
#[cfg(test)]
mod v53300_tests {
    #[test]
    fn assert_schema_error_has_suggestion() {
        use crate::driver::cmd_explain_error_collect;
        let out = cmd_explain_error_collect("E0419")
            .expect("E0419 must be in error catalog");
        assert!(
            out.contains("Suggestion"),
            "E0419 explain output must include Suggestion section"
        );
        // "Float.from_int" は suggestion テキスト固有（example には含まれない）
        assert!(
            out.contains("Float.from_int"),
            "E0419 suggestion must mention Float.from_int() as type conversion hint"
        );
    }

    #[test]
    fn assert_schema_diff_shown() {
        use crate::driver::cmd_explain_error_collect;
        let out = cmd_explain_error_collect("E0419")
            .expect("E0419 must be in error catalog");
        assert!(
            out.contains("expected Int"),
            "E0419 example must show 'expected Int' in field diff"
        );
        assert!(
            out.contains("got String"),
            "E0419 example must show 'got String' in field diff"
        );
    }
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "53.2.0"` → `version = "53.3.0"`

v53200_tests にはバージョンピンテストが存在しないため、空化対象なし。

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3169 passed, 0 failed

既存テスト確認:
```bash
cargo test v52100_tests -- --nocapture 2>&1 | grep "assert_schema_type_fail"
# → PASS（title 変更なしのため）
```

---

## ステップ 6: 後処理

- `CHANGELOG.md` に v53.3.0 エントリ追加
- `versions/current.md` を v53.3.0（3169 tests）に更新
- `roadmap-v53.1-v54.0.md` の v53.3.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T4 全 `[x]`）
