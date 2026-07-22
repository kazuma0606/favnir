# Spec: v50.5.0 — LSP インレイヒント Phase 2（パイプライン stage 型）

Date: 2026-07-19
Status: Draft

---

## 概要

パイプラインの各 `stage` の型を `: In -> Out` 形式でインレイヒント表示する。
`lsp/inlay_hints.rs` に `collect_pipeline_type_hints` を追加し、`type_at` に格納された
`Type::Arrow` / `Type::Trf` 型を持つ stage 名に対して `: In -> Out` ヒントを生成する。

> **テスト件数の注記**: ロードマップ v50.5.0 は機能テスト 2 件（`lsp_inlay_hint_stage_type`・
> `lsp_inlay_hint_pipeline_type`）を完了条件として記載している。
> 本バージョンではこれに加えてバージョン確認テスト 1 件（`cargo_toml_version_is_50_5_0`）を
> 追加するため、`v505000_tests` モジュールは合計 3 件となる。テスト総数は 3101。

---

## 背景

### 現状（v50.4.0 時点で実装済み）

| 機能 | 実装状況 | 備考 |
|---|---|---|
| `collect_stage_hints` | **実装済み** | stage 名の型を `: Type` 形式で表示（v46.4.0） |
| `collect_bind_hints` | **実装済み** | bind 束縛の型を `: Type` 形式で表示 |
| `collect_fn_return_hints` | **実装済み** | fn 戻り型を ` -> Type` 形式で表示（v50.4.0） |
| pipeline stage 特化ヒント | **未実装** | 本バージョンで追加 |

### `Type::Arrow.display()` について

`checker.rs` の `Type::Arrow(a, b)` は `display()` で `"a -> b"` を返す。
つまり、stage の型が `Type::Arrow(RawOrder, Order)` であれば、
`collect_stage_hints` は既に `: RawOrder -> Order` を表示できる。

ただし `collect_stage_hints` はすべての stage 型（`Int`・`String` など非関数型も含む）を
無差別に表示する。`collect_pipeline_type_hints` は **`Type::Arrow` / `Type::Trf` のみ**を
対象として `: In -> Out` ヒントを生成し、pipeline コンテキストに特化した表示を提供する。

> **ロードマップ記載「`lsp/references.rs` 拡張」について**:
> `collect_stage_hints` が既に `type_at` 経由で stage 型を提供しているため、
> `references.rs` の変更は不要と判断した。`inlay_hints.rs` 内の新関数で対応する。

---

## 仕様

### 変更 1: `lsp/inlay_hints.rs` — `collect_pipeline_type_hints` 追加

`stage <Name>` 行をスキャンし、`type_at` に `Type::Arrow` または `Type::Trf` 型が記録されている
stage に対して `: In -> Out` 形式のヒントを生成する。

```rust
/// v50.5.0: Collect inlay hints for pipeline stage IO types.
/// Only emits hints for stages whose type is `Type::Arrow` or `Type::Trf`
/// (i.e. has a meaningful In -> Out structure).
/// Shares the same text-scan approach and known limitations as `collect_stage_hints`.
pub(crate) fn collect_pipeline_type_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_stage_prefix(line) {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 {
                byte_offset += line.len() + 1;
                continue;
            }
            let name = &rest[..name_end];
            if name == "_" {
                byte_offset += line.len() + 1;
                continue;
            }
            let prefix_len = line.len() - rest.len();
            let name_start = byte_offset + prefix_len;
            let name_end_offset = name_start + name_end;
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                // Only emit for Arrow / Trf types (stage IO types)
                let label = match ty {
                    Type::Arrow(_, _) | Type::Trf(_, _) => {
                        Some(format!(": {}", ty.display()))
                    }
                    _ => None,
                };
                if let Some(label) = label {
                    let col = (prefix_len + name_end) as u32;
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: col,
                        },
                        label,
                        kind: 1,
                    });
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}
```

### 変更 2: `handle_inlay_hints` に `collect_pipeline_type_hints` を追加

```rust
pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    hints.extend(collect_fn_return_hints(&doc.source, &doc.type_at));
    // v50.5.0: pipeline stage IO type hints (Arrow / Trf types only)
    hints.extend(collect_pipeline_type_hints(&doc.source, &doc.type_at));
    hints
}
```

### テスト仕様

`v505000_tests` モジュールを `driver.rs` の `v504000_tests` 直前に追加（3 件）。

テスト総数: 3099（ベース）− 1（`cargo_toml_version_is_50_4_0` 削除）+ 3（v505000_tests 追加）= **3101**。

> **`lsp_inlay_hint_stage_type` について**: この関数は既存の `collect_stage_hints`（v46.4.0 実装済み）を呼び出す。
> `Type::Arrow` を持つ stage に対して既存関数が正しく `: In -> Out` を表示することを確認する**回帰テスト**である。
> 本バージョンの新関数 `collect_pipeline_type_hints` は `lsp_inlay_hint_pipeline_type` で直接検証する。

> **`Type::Trf` のカバレッジについて**: `collect_pipeline_type_hints` の `Type::Trf` ブランチは
> `Type::Arrow` と同一コードパス（同一 `match` アーム）を通るため、`Type::Arrow` のテストカバレッジで代替する。

```rust
// -- v505000_tests (v50.5.0) -- LSP インレイヒント Phase 2 --
#[cfg(test)]
mod v505000_tests {
    #[test]
    fn cargo_toml_version_is_50_5_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.5.0\""),
            "Cargo.toml version should be 50.5.0");
    }

    #[test]
    fn lsp_inlay_hint_stage_type() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_stage_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        // "  stage Parse\n": "stage " = 6 bytes after 2-space indent,
        // "Parse" at bytes (indent=2 + "stage "=6) = 8..13
        // Span.line is 1-indexed; find_type_at uses start/end only
        let source = "  stage Parse\n";
        let span = Span { file: "t".to_string(), start: 8, end: 13, line: 1, col: 9 };
        let mut type_at = HashMap::new();
        type_at.insert(span, Type::Arrow(
            Box::new(Type::Named("RawOrder".to_string(), vec![])),
            Box::new(Type::Named("Order".to_string(), vec![])),
        ));

        let hints = collect_stage_hints(source, &type_at);
        assert_eq!(hints.len(), 1, "expected one hint for stage");
        assert!(hints[0].label.contains("RawOrder"),
            "hint should show input type, got: {}", hints[0].label);
        assert!(hints[0].label.contains("Order"),
            "hint should show output type, got: {}", hints[0].label);
    }

    #[test]
    fn lsp_inlay_hint_pipeline_type() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_pipeline_type_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        // Two stages: "  stage Parse\n  stage Validate\n"
        // "  stage Parse":   "Parse"   at bytes 8..13   (line 1)
        // "  stage Validate": "Validate" at bytes 22..30 (line 2, offset 14+2+6=22)
        // Span.line is 1-indexed; find_type_at uses start/end only
        let source = "  stage Parse\n  stage Validate\n";
        let mut type_at = HashMap::new();
        type_at.insert(
            Span { file: "t".to_string(), start: 8, end: 13, line: 1, col: 9 },
            Type::Arrow(
                Box::new(Type::Named("RawOrder".to_string(), vec![])),
                Box::new(Type::Named("Order".to_string(), vec![])),
            ),
        );
        type_at.insert(
            Span { file: "t".to_string(), start: 22, end: 30, line: 2, col: 9 },
            Type::Arrow(
                Box::new(Type::Named("Order".to_string(), vec![])),
                Box::new(Type::Named("ValidOrder".to_string(), vec![])),
            ),
        );

        let hints = collect_pipeline_type_hints(source, &type_at);
        assert_eq!(hints.len(), 2, "expected hints for both stages");
        let labels: Vec<_> = hints.iter().map(|h| &h.label).collect();
        assert!(labels.iter().any(|l| l.contains("RawOrder")),
            "first stage hint should show RawOrder, got: {:?}", labels);
        assert!(labels.iter().any(|l| l.contains("ValidOrder")),
            "second stage hint should show ValidOrder, got: {:?}", labels);
    }
}
```

### byte offset 計算の検証

`"  stage Parse\n"` (14 bytes):
- `line.trim_start()` → `"stage Parse"`
- `find_stage_prefix` → `rest = "Parse"` (after `"stage "` strip + trim_start)
- `prefix_len = 14 - 1 - 5 = ?` ← 要確認

`find_stage_prefix` の実装:
```rust
fn find_stage_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();           // "stage Parse"
    trimmed.strip_prefix("stage ").map(|r| r.trim_start())  // "Parse"
}
```
`rest = "Parse"`, `rest.len() = 5`, `line.len() = 13` (末尾 `\n` は `lines()` で除去)

→ `prefix_len = 13 - 5 = 8`、`name_start = 0 + 8 = 8`、`name_end = 5`、`name_end_offset = 13`

スパン `start: 8, end: 13` と一致 ✓

`"  stage Validate\n"` は 2 行目なので `byte_offset = 14` (1 行目 13 bytes + `\n` 1 byte):
- `line.len() = 16` (末尾 `\n` 除去後)
- `prefix_len = 16 - 8 = 8`、`name_start = 14 + 8 = 22`、`name_end = 8`、`name_end_offset = 30`

スパン `start: 22, end: 30` と一致 ✓

---

## 完了条件

- `cargo test` 3101 passed, 0 failed
- `lsp/inlay_hints.rs`: `collect_pipeline_type_hints` 追加、`handle_inlay_hints` に組み込み
- `collect_pipeline_type_hints` は `pub(crate)` で `driver.rs` テストからアクセス可能
- `collect_pipeline_type_hints` は `Type::Arrow` / `Type::Trf` 型の stage のみヒントを生成する
- `Type::Arrow` / `Type::Trf` を持つ stage には `collect_stage_hints` と `collect_pipeline_type_hints` の両方がヒントを生成することを**許容する**（重複表示の解消は v51.0 以降のスコープ）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.5.0 エントリ追加
- `versions/current.md` を v50.5.0 に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.5.0 実績欄に「`references.rs` 変更は不要と判断、`inlay_hints.rs` 内の新関数 `collect_pipeline_type_hints` で代替」と記録する
