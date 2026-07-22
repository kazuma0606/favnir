# Spec: v53.3.0 — DX × DQ 統合（`assert_schema` 失敗時の詳細 suggestion）

Status: 計画中
Date: 2026-07-22

---

## 概要

`assert_schema` 失敗時のエラー体験を改善する。
`error_catalog.rs` の E0419 エントリを更新し、フィールド差分・型変換ヒント・具体的な修正例を含む
リッチな説明・example・suggestion に書き換える。
`fav explain --error E0419` 出力が「どのフィールドで何が違うか・どう直すか」を示すようになる。

> ロードマップには「`EXPLAIN_CATALOG` 更新」と「span + suggestion 統一診断フォーマット適用」と書かれているが、
> 実装上の対応は以下の通り:
>
> - **`EXPLAIN_CATALOG`**: ロードマップ上の名称。実体は `fav/src/error_catalog.rs` の `ErrorEntry` 配列。
>   `fav explain --error E0419` → `cmd_explain_error_collect("E0419")` → `error_catalog::lookup("E0419")` の経路で参照される。
>
> - **span 適用**: ランタイムエラー（VM 出力）への span 埋め込みは本バージョンのスコープ外。
>   VM 側（`backend/vm.rs`）は既に `"field \`id\`: expected Int but got String"` 形式のメッセージを出力しており、
>   これ以上の変更は不要と判断した。`fav explain --error E0419` の出力品質向上のみ実施する。
>
> v53.3.0 の実作業は **E0419 カタログエントリの内容強化**のみ（ランタイム VM の変更なし）。

---

## 実装スコープ

### 1. `error_catalog.rs` — E0419 エントリ更新

現状:
```rust
ErrorEntry {
    code: "E0419",
    title: "assert_schema type mismatch",
    category: "runtime",
    description: "assert_schema<T> found a field whose runtime type does not match the schema T.",
    example: "// expected { id: Int } but got { id: \"hello\" }",
    fix: "Ensure the input map contains fields matching the schema T.",
    suggestion: Some("Check the upstream data source for type mismatches."),
}
```

更新後:
```rust
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
}
```

**注意**: `title` フィールド（`"assert_schema type mismatch"`）は変更しない。
`v52100_tests::assert_schema_type_fail` が `src.contains("assert_schema type mismatch")` を assert しているため。

### 2. テスト仕様

`v53300_tests` モジュールを `driver.rs` に追加（`v53200_tests` の直前）:

```rust
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

---

## バージョン更新

- `fav/Cargo.toml`: `"53.2.0"` → `"53.3.0"`

---

## 完了条件

- `cargo test` 3169 passed, 0 failed（3167 + 2 件追加）
  （ベース 3167 = v53.2.0 完了時実績。ロードマップ推定値 3163 との差 +6 は v53.1.0 コードレビュー起因）
- `v53300_tests` 2 件 pass:
  - `assert_schema_error_has_suggestion`
  - `assert_schema_diff_shown`
- `cargo clippy -- -D warnings` クリーン
- `v52100_tests::assert_schema_type_fail` が引き続き pass すること（title 変更なし確認）

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/error_catalog.rs` | E0419 エントリの `description` / `example` / `fix` / `suggestion` 更新（`title` は変更しない） |
| `fav/src/driver.rs` | `v53300_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.3.0 エントリ追加 |
| `versions/current.md` | v53.3.0 / 3169 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.3.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `title` フィールドを変更すると `v52100_tests::assert_schema_type_fail` が壊れる — 変更禁止
- `category` / `code` フィールドも変更しない
- VM 側（`backend/vm.rs`）の E0419 ランタイムメッセージは変更しない（既に詳細なメッセージを出力）
- `cmd_explain_error_collect` の実装は変更不要（カタログエントリの内容を整形して返すだけ）
- wasm32 影響なし（`error_catalog.rs` は wasm でも使用されるが、文字列変更のみのため影響なし）
