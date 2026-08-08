# v61.6.0 — 型エラーメッセージ品質（期待型 vs 実際型の差分表示）

## 概要

型不一致エラー（E0009）のメッセージに構造的差分を追加し、
データエンジニアが「どこが違うのか」を一目で把握できるようにする。

`diff_types` ヘルパー関数を `checker.rs` に追加し、
E0009 発行時に差分 hint を `type_error_h` 経由で出力する。

---

## 動機

現状の E0009 メッセージは期待型と実際型の名前を並べるだけで、
Record フィールドの過不足やスカラー vs 複合型の違いが分からない。

```
error[E0009]: type mismatch
  expected: List<Row>
  found:    List<String>
```

変更後:

```
error[E0009]: type mismatch
  expected: List<Row>
  found:    List<String>
            ^^^^^^^^^^^^
  difference: Row has fields { id: Int, name: String }, but String is a scalar type.
  help: Did you forget to wrap the string in a Row record?
```

---

## スコープ

### 変更ファイル（3 ファイル）

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `diff_types` 関数追加 + E0009 call site を `type_error_h` + hints に更新 |
| `fav/src/error_catalog.rs` | E0009 の `long_description` を更新 |
| `fav/src/driver.rs` | `v61600_tests` モジュール追加（テスト 2 件） |

---

## 実装詳細

### 1. `diff_types` ヘルパー（checker.rs）

```rust
/// 型の構造的差分を人間向けテキストで返す。
/// 差分がない（同型）か、表示するほどの差分でない場合は None を返す。
fn diff_types(
    expected: &Type,
    found: &Type,
    type_defs: &HashMap<String, TypeBody>,
) -> Option<String> {
    match (expected, found) {
        // Record vs スカラー
        (Type::Record(fields), _) if !matches!(found, Type::Record(_)) => {
            let field_list = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, ty_to_str(v)))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!(
                "{} has fields {{ {} }}, but {} is a scalar type.",
                ty_to_str(expected),
                field_list,
                ty_to_str(found)
            ))
        }
        // Record vs Record: フィールド差分
        (Type::Record(exp_fields), Type::Record(found_fields)) => {
            let missing: Vec<_> = exp_fields
                .iter()
                .filter(|(k, _)| !found_fields.contains_key(k.as_str()))
                .map(|(k, v)| format!("{}: {}", k, ty_to_str(v)))
                .collect();
            let mismatched: Vec<_> = exp_fields
                .iter()
                .filter_map(|(k, exp_ty)| {
                    found_fields.get(k.as_str()).and_then(|found_ty| {
                        if exp_ty != found_ty {
                            Some(format!(
                                "{}: expected {}, found {}",
                                k,
                                ty_to_str(exp_ty),
                                ty_to_str(found_ty)
                            ))
                        } else {
                            None
                        }
                    })
                })
                .collect();
            if missing.is_empty() && mismatched.is_empty() {
                None
            } else {
                let mut parts = Vec::new();
                if !missing.is_empty() {
                    parts.push(format!("missing fields: {}", missing.join(", ")));
                }
                if !mismatched.is_empty() {
                    parts.push(format!("type mismatches: {}", mismatched.join("; ")));
                }
                Some(parts.join("; "))
            }
        }
        // List<A> vs List<B>: 要素型の差分
        (Type::List(exp_inner), Type::List(found_inner)) => {
            diff_types(exp_inner, found_inner, type_defs)
                .map(|msg| format!("element type mismatch — {}", msg))
        }
        // 名前付き型: TypeDefs を展開して再比較
        (Type::Named(name, _), _) => {
            type_defs.get(name).and_then(|body| match body {
                TypeBody::Record(fields) => {
                    let rec = Type::Record(
                        fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    );
                    diff_types(&rec, found, type_defs)
                }
                _ => None,
            })
        }
        _ => None,
    }
}
```

`diff_types` は既存の `unify` 失敗パスに組み込む（非侵襲的）。
`type_defs` は checker の `Checker` 構造体が保持する `HashMap<String, TypeBody>` から渡す。

### 2. E0009 call site 更新（checker.rs）

`unify` が失敗したとき `type_error` を呼んでいる箇所を `type_error_h` に変更し、
`diff_types` の結果を hints に追加する。

```rust
// 変更前
Err(e) => return Err(type_error("E0009", &msg, span)),

// 変更後
let hints = diff_types(&expected_ty, &found_ty, &self.type_defs)
    .map(|d| vec![d])
    .unwrap_or_default();
return Err(type_error_h("E0009", &msg, span, hints));
```

対象 call site: `check_stage_output` 内の seq chain 型一致チェック部分（L5050 付近）。
全 call site の特定は tasks.md T0 で `grep "E0009"` により実測する。

### 3. error_catalog.rs E0009 更新

`suggestion` フィールドは静的テキストを維持する（`ErrorEntry` は静的データのため動的生成不可）。
動的な差分テキストは `diff_types` の出力を `type_error_h` の `hints` に渡すことで実行時に提供する。

```rust
ErrorEntry {
    code: "E0009",
    message: "type mismatch in stage output",
    suggestion: Some("Check that the stage output type matches the next stage input."),
    long_description: Some(
        "The output type of a stage does not match what the next stage expects.\n\
         Structural differences (missing fields, type mismatches) are shown as hints at runtime.\n\
         Common causes: missing record fields, List element type mismatch, scalar vs record."
    ),
},
```

---

## 完了条件

- **Rust テスト 2 件**（ベース 3369 + 2 = 3371 tests passed, 0 failed）
  - `type_error_diff_display_record` — Record vs スカラー差分が hint に含まれる
  - `type_error_suggestion_e0009` — E0009 の suggestion テキストが正しい

---

## 注意事項

- `diff_types` は `unify` 関数内ではなく `check_stage_output` 等の call site に置く
  （`unify` は汎用で hints 生成コストをかけたくないため）
- `ty_to_str` は既存のユーティリティ関数を使う（新規追加不要）
- `type_defs` フィールド名は checker の実際の構造体フィールドを確認して使う
- `TypeBody::Record` の型は checker.rs 内の定義に合わせる
- exhaustive match の追加ファイルは発生しない（新規 AST バリアント追加なし）
