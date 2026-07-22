# Spec: v50.2.0 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

Date: 2026-07-18
Status: Draft

---

## 概要

`fav check --json`・LSP `textDocument/publishDiagnostics`・CLI stderr の 3 経路すべてで
`suggestion` と `span` が一貫して出力されることを保証する。

現状の経路別ステータス:

| 経路 | `suggestion` | `span` | 備考 |
|---|---|---|---|
| `fav check --json` | **実装済み** | **実装済み** | `CheckDiagnostic.suggestion/.line/.col`（v12.5.0） |
| CLI stderr | **実装済み** | **実装済み** | `fav check` stderr フォーマット済み |
| LSP `publishDiagnostics` | **未対応** | **実装済み** | `Diagnostic.range` は実装済み、`data.suggestion` が欠如 |

v50.2.0 では **LSP `suggestion` 経路を補完** し、3 経路すべての一貫化を完成させる。
`span`（LSP `range`）はすでに全経路で実装済みのため本バージョンの対象外。

---

## 背景

v50.1.0 で全エラーコードに `suggestion: Some(...)` が設定された。
しかし LSP クライアント（VS Code 等）が受け取る `textDocument/publishDiagnostics` の
`Diagnostic` オブジェクトには `suggestion` 情報が含まれていないため、
エディタ上では修正提案が届かない。

LSP spec（3.16+）では `Diagnostic.data` フィールドに任意の付加情報を格納できる。
`data: { "suggestion": "..." }` を追加することで、LSP クライアント側で
code action や hover に suggestion を利用できるようになる。

また、`fav check --json` の suggestion は v12.5.0 時点の実装では E0018 等の
特殊ケースハードコードと catalog フォールバックが混在している。
v50.2.0 では catalog 経由の coverage（v50.1.0 完成）を利用し、
JSON 出力の suggestion が catalog 由来であることをテストで固定する。

---

## 仕様

### 変更 1: `lsp/protocol.rs` — `Diagnostic` struct 拡張

`DiagnosticData` struct を新規追加し、`Diagnostic` に `data` フィールドを追加。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticData {
    pub suggestion: String,
}

pub struct Diagnostic {
    pub range:    Range,
    pub severity: u32,
    pub code:     String,
    pub message:  String,
    /// v50.2.0: suggestion for LSP clients (data.suggestion in LSP spec §3.16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DiagnosticData>,
}
```

### 変更 2: `lsp/diagnostics.rs` — `data.suggestion` を設定

`errors_to_diagnostics` で `error_catalog::lookup(err.code)` を呼び出し、
`suggestion` が存在する場合に `data` フィールドを設定する。

```rust
use crate::error_catalog;
use crate::lsp::protocol::DiagnosticData;

pub fn errors_to_diagnostics(errors: &[TypeError], source: &str) -> Vec<Diagnostic> {
    errors.iter().map(|err| {
        let suggestion = error_catalog::lookup(err.code)
            .and_then(|e| e.suggestion)
            .unwrap_or("")
            .to_string();
        let data = if suggestion.is_empty() {
            None
        } else {
            Some(DiagnosticData { suggestion })
        };
        Diagnostic {
            range: span_to_range(&err.span, source, err.code),
            severity: 1,
            code: err.code.to_string(),
            message: err.message.clone(),
            data,
        }
    }).collect()
}
```

### テスト仕様

`v502000_tests` モジュールを `driver.rs` の `v501000_tests` 直前に追加（機能テスト 2 件 + バージョン確認 1 件 = 合計 3 件）。

テスト総数: 3093（ベース）− 1（`cargo_toml_version_is_50_1_0` 削除）+ 3（v502000_tests 追加）= **3095**。

```rust
// -- v502000_tests (v50.2.0) -- 診断統一 Phase 2 --
#[cfg(test)]
mod v502000_tests {
    #[test]
    fn cargo_toml_version_is_50_2_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.2.0\""), "Cargo.toml version should be 50.2.0");
    }

    #[test]
    fn check_json_includes_suggestion() {
        // v50.1.0 以降は E0213 等の catalog コードすべてに suggestion あり。
        // default_suggestion が catalog フォールバック経由で値を返すことを確認。
        let s = super::default_suggestion("E0213");
        assert!(!s.is_empty(), "E0213 should have non-empty suggestion in JSON output");
        let s2 = super::default_suggestion("E0380");
        assert!(
            !s2.is_empty(),
            "E0380 (added in v50.1.0) should have suggestion in JSON output"
        );
    }

    #[test]
    fn lsp_diagnostic_includes_suggestion() {
        use crate::frontend::lexer::Span;
        use crate::lsp::diagnostics::errors_to_diagnostics;
        use crate::middle::checker::TypeError;

        let errors = vec![TypeError::new(
            "E0213",
            "type mismatch",
            Span::new("test.fav", 0, 0, 1, 1),
        )];
        let diags = errors_to_diagnostics(&errors, "fn f() -> Int { true }");
        assert!(!diags.is_empty());
        let json = serde_json::to_string(&diags[0]).expect("serialize");
        assert!(
            json.contains("\"suggestion\""),
            "LSP diagnostic should include suggestion in data field, got: {json}"
        );
    }
}
```

### テスト配置

`v502000_tests` を `driver.rs` の `v501000_tests` 直前に挿入する。

---

## 完了条件

- `cargo test` 3095 passed, 0 failed（3093 − 1 + 3 = net +2 件）
- `lsp/protocol.rs`: `DiagnosticData` struct + `Diagnostic.data: Option<DiagnosticData>` 追加
- `lsp/diagnostics.rs`: `errors_to_diagnostics` が `data.suggestion` を設定
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.2.0 エントリ追加
- `versions/current.md` を v50.2.0 に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.2.0 実績を記入
