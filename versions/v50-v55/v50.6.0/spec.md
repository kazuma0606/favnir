# Spec: v50.6.0 — LSP ホバー情報強化（Rune メソッドシグネチャ）

Date: 2026-07-19
Status: Draft

---

## 概要

`textDocument/hover` の応答に Rune メソッドのシグネチャ・エフェクト・ドキュメントコメントを含める。
`lsp/hover.rs` に `builtin_hover_at` と `rune_hover_at` の 2 関数を追加し、
テキストスキャンで `Namespace.method` パターンを検出してホバー情報を生成する。

> **ロードマップ記載「rune.toml `[[exports]]` セクションからメタデータを読み込む」につ��て**:
> 既存の rune ディレクトリに `rune.toml` の `[[exports]]` セクションは未実装のため、
> v50.6.0 では静的テーブ��� `RUNE_FNS` を `hover.rs` 内に定義して代替する。
> 動的な rune.toml 読み込みは将来バージョン（v51.x）のスコープとする。

---

## 背景

### 現状（v50.5.0 時点）

| 機能 | 状態 | 備考 |
|---|---|---|
| `handle_hover` | **実装済み** | `type_at` スパン → 型表示 |
| ドキュメントコメントホバー | **実装済み** | `doc_comments` HashMap 経由 |
| ビルトイン関数シグネチャ | **未実装** | `completion.rs` の `BUILTIN_FNS` は補完のみ |
| Rune メソッドシグネチャ | **未実装** | 本バージョンで追加 |

### `BUILTIN_FNS` について

`completion.rs` の `BUILTIN_FNS` は `namespace`, `name`, `signature`, `params` フィールドを持つ。
`builtin_hover_at` はこのテーブルを再利用し��シグネチャを表示する。

---

## 仕様

### 変更 1: `lsp/hover.rs` — `RuneFn` 構造体 + `RUNE_FNS` 定数追加

```rust
/// Static metadata for Rune exported functions.
/// v50.6.0: populated from known rune signatures.
/// Future: populate dynamically from rune.toml `[[exports]]` sections.
pub(crate) struct RuneFn {
    pub rune: &'static str,
    pub name: &'static str,
    pub signature: &'static str, // e.g. "(topic: String) -> Stream<RawMessage>"
    pub effect: &'static str,    // e.g. "!Kafka"
    pub doc: &'static str,
}

pub(crate) const RUNE_FNS: &[RuneFn] = &[
    RuneFn {
        rune: "kafka", name: "consume",
        signature: "(topic: String) -> Stream<RawMessage>",
        effect: "!Kafka",
        doc: "Consumes messages from the given Kafka topic.",
    },
    RuneFn {
        rune: "kafka", name: "produce",
        signature: "(topic: String, msg: RawMessage) -> Unit",
        effect: "!Kafka",
        doc: "Produces a message to the given Kafka topic.",
    },
    RuneFn {
        rune: "csv", name: "read",
        signature: "(path: String) -> List<Row>",
        effect: "!IO",
        doc: "Reads a CSV file and returns rows as a list.",
    },
    RuneFn {
        rune: "csv", name: "write",
        signature: "(path: String, rows: List<Row>) -> Unit",
        effect: "!IO",
        doc: "Writes rows to a CSV file.",
    },
];
```

### 変更 2: `word_and_ns_at` ヘルパー追加（モジュールプライベート）

> **オフセット空間について**: `offset` は `position_to_char_offset` が返す値を想定する。
> `position_to_char_offset` は `char_indices()` の `idx`（先頭バイトインデックス）を返すため、
> `word_and_ns_at` が `as_bytes()[offset]` でインデックスするバイト空間と一致する。
> ASCII ソースでは byte index == char index。マルチバイト文字の内部バイトが `offset` に
> 渡された場合は `start==end` となり `None` を返す（安全に縮退）。

> **境界動作**: `offset` がドット（`.`）自体を指す場合は `start==end` となり `None` を返す。
> `offset` が method 文字列の任意のバイト位置（先頭・中間・末尾）を指す場合は正常動作する。

```rust
/// Extract (namespace, method) from source at byte offset.
/// `offset` must be a byte index (as returned by `position_to_char_offset`).
/// Returns None if the cursor is not on `namespace.method` pattern,
/// or if `offset` is on the dot separator itself.
fn word_and_ns_at(source: &str, offset: usize) -> Option<(&str, &str)> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    // find method end
    let mut end = offset;
    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    // find method start
    let mut start = offset;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    if start == end { return None; }
    // require preceding dot
    if start == 0 || bytes[start - 1] != b'.' { return None; }
    let dot_pos = start - 1;
    // find namespace start
    let mut ns_start = dot_pos;
    while ns_start > 0 && (bytes[ns_start - 1].is_ascii_alphanumeric() || bytes[ns_start - 1] == b'_') {
        ns_start -= 1;
    }
    if ns_start == dot_pos { return None; }
    Some((&source[ns_start..dot_pos], &source[start..end]))
}
```

### 変更 3: `builtin_hover_at` 追加

```rust
/// v50.6.0: Text-scan hover for builtin Namespace.method.
/// Reuses `BUILTIN_FNS` from completion.rs.
pub(crate) fn builtin_hover_at(source: &str, offset: usize) -> Option<String> {
    let (ns, method) = word_and_ns_at(source, offset)?;
    // Builtin namespaces are PascalCase (uppercase first letter)
    if ns.chars().next().map(|c| c.is_lowercase()).unwrap_or(true) {
        return None;
    }
    use crate::lsp::completion::BUILTIN_FNS;
    let entry = BUILTIN_FNS.iter().find(|f| f.namespace == ns && f.name == method)?;
    // Note: entry.params is intentionally unused here.
    // Signature help (existing completion.rs feature) handles parameter-level detail.
    Some(format!("```favnir\nfn {}.{}{}\n```", ns, method, entry.signature))
}
```

### 変更 4: `rune_hover_at` 追加

```rust
/// v50.6.0: Text-scan hover for rune.method.
/// Uses static RUNE_FNS table.
pub(crate) fn rune_hover_at(source: &str, offset: usize) -> Option<String> {
    let (rune, method) = word_and_ns_at(source, offset)?;
    // Rune names are lowercase (vs PascalCase builtins)
    if rune.chars().next().map(|c| c.is_uppercase()).unwrap_or(true) {
        return None;
    }
    let entry = RUNE_FNS.iter().find(|f| f.rune == rune && f.name == method)?;
    // Two spaces before effect to match roadmap display convention
    let sig_line = format!("fn {}{}  {}", method, entry.signature, entry.effect);
    let value = if entry.doc.is_empty() {
        format!("```favnir\n{}\n```", sig_line)
    } else {
        format!("```favnir\n{}\n```\n\n{}", sig_line, entry.doc)
    };
    Some(value)
}
```

### 変更 5: `handle_hover` 更新

```rust
pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    // v50.6.0: try builtin / rune method hover first (text-scan, richer info)
    if let Some(content) = builtin_hover_at(&doc.source, offset)
        .or_else(|| rune_hover_at(&doc.source, offset))
    {
        return Some(Hover {
            contents: MarkupContent {
                kind: "markdown".to_string(),
                value: content,
            },
        });
    }

    // existing type_at lookup
    let (span, ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, offset))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))?;

    let type_block = format!("```favnir\n{}\n```", display_type(ty));
    let value = match doc_comment_for_span(doc, span, offset) {
        Some(text) if !text.is_empty() => format!("{}\n\n{}", text, type_block),
        _ => type_block,
    };

    Some(Hover {
        contents: MarkupContent {
            kind: "markdown".to_string(),
            value,
        },
    })
}
```

### テスト仕様

`v506000_tests` モジュールを `driver.rs` の `v505000_tests` 直前に追加（3 件）。

テスト総数: 3101（ベース）− 1（`cargo_toml_version_is_50_5_0` 削除���+ 3（v506000_tests 追加）= **3103**。

```rust
// -- v506000_tests (v50.6.0) -- LSP ホバー情報強化 --
#[cfg(test)]
mod v506000_tests {
    #[test]
    fn cargo_toml_version_is_50_6_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.6.0\""),
            "Cargo.toml version should be 50.6.0");
    }

    #[test]
    fn lsp_hover_builtin_fn() {
        use crate::lsp::hover::builtin_hover_at;
        // "List.map(items, f)" — offset=5 is on "m" of "map"
        // 'L'=0 'i'=1 's'=2 't'=3 '.'=4 'm'=5
        let source = "List.map(items, f)";
        let result = builtin_hover_at(source, 5);
        assert!(result.is_some(), "expected hover for List.map");
        let text = result.unwrap();
        assert!(text.contains("map"),    "hover should mention function name, got: {}", text);
        assert!(text.contains("List"),   "hover should show List signature, got: {}", text);
    }

    #[test]
    fn lsp_hover_rune_method() {
        use crate::lsp::hover::rune_hover_at;
        // "kafka.consume(topic)" — offset=6 is on "c" of "consume"
        // 'k'=0 'a'=1 'f'=2 'k'=3 'a'=4 '.'=5 'c'=6
        let source = "kafka.consume(topic)";
        let result = rune_hover_at(source, 6);
        assert!(result.is_some(), "expected hover for kafka.consume");
        let text = result.unwrap();
        assert!(text.contains("consume"), "hover should mention method name, got: {}", text);
        assert!(text.contains("Kafka"),   "hover should show !Kafka effect, got: {}", text);
    }
}
```

#### バイトオフセット検証

`"List.map(items, f)"`:
- 'L'=0, 'i'=1, 's'=2, 't'=3, '.'=4, 'm'=5
- offset=5 は `map` の先頭 ✓

`"kafka.consume(topic)"`:
- 'k'=0, 'a'=1, 'f'=2, 'k'=3, 'a'=4, '.'=5, 'c'=6
- offset=6 は `consume` の先頭 ✓

---

## 完了条件

- `cargo test` 3103 passed, 0 failed
- `lsp/hover.rs`: `RuneFn` 構造体・`RUNE_FNS` 定数・`word_and_ns_at`・`builtin_hover_at`・`rune_hover_at` 追加
- `handle_hover` が builtin/rune lookup を優先し、マッチしなければ既存 `type_at` にフォールバック
- `builtin_hover_at` は PascalCase 名前空間（ビルトイン）のみ対象
- `rune_hover_at` は lowercase 名前空間（rune）のみ対象（PascalCase は即 `None`）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.6.0 エントリ追加
- `versions/current.md` を v50.6.0 に更新
- ロードマップ実績欄の記録は tasks.md T3 が権威（spec.md での重複管理は行わない）
