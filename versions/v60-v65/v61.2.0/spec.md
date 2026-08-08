# v61.2.0 Spec — as-pattern 拡張（ネストパターン・LSP hover 統合）

Date: 2026-07-31
Status: COMPLETE

---

## 概要

`Pattern::As` は v56.6.0 で AST に追加済み。本バージョンでは以下 3 点を拡張する。

1. **checker.rs**: as-pattern + Record パターンのネストが正しく型チェックされることを確認テストで保証
2. **inlay_hints.rs**: as-pattern の束縛変数名に型ヒントを表示（`collect_as_pattern_hints` 追加）
3. **lint.rs**: W039 — as-name が内側パターンの束縛変数と衝突する場合に警告

---

## W038 / W039 との関係（注意）

ロードマップには「W038 lint（名前衝突）」と記載されているが、**W038 は v56.7.0 で wildcard import collision チェックとして既に実装済み**。

次の空きコード **W039** を本バージョン（v61.2.0）の as-name シャドウ警告に使用する。
これにより、ロードマップ v61.7.0 に記載されていた「W039 `type_hole_inferred`」は **W040** に繰り上がる。
ロードマップの v61.7.0 セクションは本バージョン実装後に W040 に更新すること（T5 で実施）。

---

## 実装スコープ

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/middle/checker.rs` | 変更なし | 現行実装が正しいことをテストで確認（コード変更不要） |
| `fav/src/lsp/inlay_hints.rs` | 追加 | `collect_as_pattern_hints` 関数 + `handle_inlay_hints` から呼び出し |
| `fav/src/lint.rs` | 追加 | `check_w039_as_name_shadows_inner` 関数 + `check_all` から呼び出し |
| `fav/src/driver.rs` | 追加 | `v61200_tests` モジュール（2 件） |

新規ファイルなし。`Cargo.toml` バージョン変更なし（v61.x.x はサブバージョン）。

---

## checker.rs — 現行実装の確認

`checker.rs` L4238 の現行実装:

```rust
Pattern::As(name, inner, _) => {
    self.env.define(name.clone(), ty.clone());
    self.check_pattern_bindings(inner, ty);
}
```

`name` に外側の型 `ty` を束縛し、`inner` パターンに対して同じ `ty` で再帰する。
Record パターン（`{ id, name }`）が内側にある場合、`check_pattern_bindings` の Record アームが
フィールドごとに型を分解するため、コードは変更不要。

**テストのみ追加**して動作を保証する。

---

## inlay_hints.rs — `collect_as_pattern_hints`

### 仕様

テキストスキャン方式（`collect_bind_hints` / `collect_stage_hints` と同一方針）。
ソース行から ` as ` を検索し、直後の識別子を名前として、`type_at` マップで型を探す。

`find_as_prefix` ヘルパーは**追加しない**。`collect_bind_hints` と異なり ` as ` は行頭固定ではないため、オフセット計算をインラインで実施する（他の hints 関数群と構造的に若干異なるが dead code 警告を回避するため）。

```rust
/// v61.2.0: as-pattern 束縛変数の型を inlay hint 表示。
/// テキストスキャン方式 — コメント・文字列内の ` as ` に誤検出する可能性があるが
/// collect_bind_hints 同様の方針で許容する。
pub(crate) fn collect_as_pattern_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(as_pos) = line.find(" as ") {
            let rest = &line[as_pos + 4..];
            let trimmed = rest.trim_start();
            let trim_delta = rest.len() - trimmed.len();
            let name_end = trimmed
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(trimmed.len());
            if name_end > 0 {
                let name = &trimmed[..name_end];
                if name != "_" {
                    let prefix_len = as_pos + 4 + trim_delta;
                    let name_start = byte_offset + prefix_len;
                    let name_end_offset = name_start + name_end;
                    if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                        let col = (prefix_len + name_end) as u32;
                        hints.push(InlayHint {
                            position: Position { line: line_idx as u32, character: col },
                            label: format!(": {}", ty.display()),
                            kind: 1,
                        });
                    }
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}
```

### `handle_inlay_hints` への組み込み

```rust
// v61.2.0: as-pattern 束縛変数の型ヒント
hints.extend(collect_as_pattern_hints(&doc.source, &doc.type_at));
```

---

## lint.rs — W039 as-name シャドウ警告

### 仕様

`Pattern::As(name, inner, span)` において、`inner` パターンが束縛する変数名のセットに
`name` が含まれる場合、W039 を発行する。

例:
```favnir
{ id, name } as id => ...  // W039: as-name 'id' shadows inner binding 'id'
```

### 実装

```rust
// ── W039: as-name shadows inner binding (v61.2.0) ────────────────────────────

/// as-pattern の内側パターンが束縛する変数名を収集する。
/// Pattern::Record のフィールドは PatternField enum
///（Pun(String,Span) / Alias(String,Box<Pattern>,Span) / Wildcard(Span)）で表現される。
fn collect_pattern_bound_names(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Bind(name, _) => vec![name.clone()],
        Pattern::Record(fields, _) => fields.iter().filter_map(|f| match f {
            PatternField::Pun(name, _) => Some(name.clone()),
            PatternField::Alias(name, _, _) => Some(name.clone()),
            PatternField::Wildcard(_) => None,
        }).collect(),
        Pattern::Or(pats, _) => pats.iter().flat_map(|p| collect_pattern_bound_names(p)).collect(),
        Pattern::As(name, inner, _) => {
            let mut names = collect_pattern_bound_names(inner);
            names.push(name.clone());
            names
        }
        _ => vec![],
    }
}
```

**W039 スコープ制限（intentional）**: `check_w039_as_name_shadows_inner` は FnDef 本体の直接の match 式のみを探索する。ネストされた match（`if` 条件内・クロージャ内等）は検出対象外とする。これは他の W03x lint と同じ「浅い探索」方針による。完了条件にこの制限を明記する。

`check_all` 末尾に `check_w039_as_name_shadows_inner(program, &mut errors);` を追加。

---

## テスト仕様（`v61200_tests` 2 件）

### `pattern_as_nested_record`

```rust
/// as-pattern が Record パターンとネストできることを確認（v61.2.0: 既存 checker の動作保証）
#[test]
fn pattern_as_nested_record() {
    let src = concat!(
        "type Point { x: Int, y: Int }\n",
        "fn origin(p: Point) -> Int {\n",
        "  match p {\n",
        "    { x, y } as whole => x\n",
        "    _ => 0\n",
        "  }\n",
        "}\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
    let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
    assert!(
        errors.is_empty(),
        "as-pattern nested in record should pass type check; errors: {:?}",
        errors
    );
}
```

### `pattern_as_lsp_hover_type`

```rust
/// as-pattern 束縛変数に inlay hint が生成されることを確認（v61.2.0: LSP 統合）
#[test]
fn pattern_as_lsp_hover_type() {
    use crate::lsp::inlay_hints::collect_as_pattern_hints;
    use crate::frontend::lexer::Span;
    use crate::middle::checker::Type;
    use std::collections::HashMap;

    // "  { x, y } as whole => x"
    //              ^-- " as " の後に "whole" (offset 14..19)
    let source = "  { x, y } as whole => x";
    let name_start: usize = 14; // "whole" の先頭 byte offset
    let name_end: usize = 19;   // "whole" の末尾 byte offset
    let mut type_at = HashMap::new();
    type_at.insert(
        // col は find_type_at で参照されないため 1 を渡す（start/end のみで比較）
        Span::new("test", name_start, name_end, 1, 1u32),
        Type::Named("Point".to_string(), vec![]),
    );
    let hints = collect_as_pattern_hints(source, &type_at);
    assert!(
        !hints.is_empty(),
        "should generate an inlay hint for as-pattern name 'whole'"
    );
    assert!(
        hints[0].label.contains("Point"),
        "hint label should contain the type name; got {:?}",
        hints[0].label
    );
}
```

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3355 + 2 = 3357」は実際のテスト数と一致する（v61.1.0 実績 3355）。

---

## 完了条件

- `pattern_as_nested_record` pass
- `pattern_as_lsp_hover_type` pass
- 総テスト数: **3357** tests passed, 0 failed
- W039 が lint.rs に追加されている（`check_all` から呼び出し済み）
- `collect_as_pattern_hints` が `handle_inlay_hints` から呼び出されている
- W039 の探索スコープ: FnDef 本体の直接 match 式のみ（ネストされた match は対象外、intentional）
- roadmap v61.7.0 の `type_hole_inferred` lint コードが W039 → W040 に更新されている

---

## テスト数推移（参照用）

| バージョン | テスト数 | 備考 |
|---|---|---|
| v61.0.0 | 3353 | Developer Experience 2.0 宣言 |
| v61.1.0 | 3355 | OR パターン強化 |
| v61.2.0 | **3357** | as-pattern 拡張 |
