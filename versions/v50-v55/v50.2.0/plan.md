# Plan: v50.2.0 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

Date: 2026-07-18

---

## 実装方針

### Step 1: 現状確認

```bash
# LSP Diagnostic struct に data フィールドがないことを確認
grep -n "struct Diagnostic" fav/src/lsp/protocol.rs
grep -n "data" fav/src/lsp/protocol.rs

# errors_to_diagnostics が suggestion を設定していないことを確認
grep -n "suggestion" fav/src/lsp/diagnostics.rs

# default_suggestion が E0380 等の新規カタログコードを返すことを確認
grep -n "default_suggestion\|E0380\|E0213" fav/src/driver.rs | head -10
```

### Step 2: `lsp/protocol.rs` — `DiagnosticData` + `Diagnostic.data` 追加

`Diagnostic` struct の直前に `DiagnosticData` struct を追加し、
`Diagnostic` に `data: Option<DiagnosticData>` フィールドを追加する。

```rust
// 追加: DiagnosticData struct（Diagnostic の直前）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticData {
    pub suggestion: String,
}

// 変更: Diagnostic に data フィールドを追加
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub range:    Range,
    pub severity: u32,
    pub code:     String,
    pub message:  String,
    /// v50.2.0: suggestion for LSP clients (LSP spec §3.16 Diagnostic.data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DiagnosticData>,
}
```

既存の `Diagnostic { range, severity, code, message }` 構築箇所は `data` フィールド追加後に
**確実にコンパイルエラーになる**。`lsp/diagnostics.rs` の `errors_to_diagnostics` 内 1 箇所を
Step 3 で同時に修正する（`data: None` を追加してビルドを通す → その後 suggestion を設定）。

### Step 3: `lsp/diagnostics.rs` — `errors_to_diagnostics` を更新

`use crate::error_catalog;` と `use crate::lsp::protocol::DiagnosticData;` を追加し、
各 error に対して catalog から suggestion を取得して `data` に設定する。

```rust
use crate::error_catalog;
use crate::lsp::protocol::DiagnosticData;

pub fn errors_to_diagnostics(errors: &[TypeError], source: &str) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|err| {
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
                range:    span_to_range(&err.span, source, err.code),
                severity: 1,
                code:     err.code.to_string(),
                message:  err.message.clone(),
                data,
            }
        })
        .collect()
}
```

### Step 4: `v502000_tests` モジュール追加

`driver.rs` の `v501000_tests` 直前に挿入する。

```rust
// -- v502000_tests (v50.2.0) -- 診断統一 Phase 2 --
#[cfg(test)]
mod v502000_tests {
    #[test]
    fn cargo_toml_version_is_50_2_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("version = \"50.2.0\""),
            "Cargo.toml version field should be 50.2.0"
        );
    }

    #[test]
    fn check_json_includes_suggestion() {
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

> 注: テストは 3 件（`cargo_toml_version_is_50_2_0` + 2 件）。
> spec の「2 件」は機能テストの数。バージョン確認テストを加えると合計 3 件、
> テスト総数 3095 → 3096 になる場合は tasks.md に記録して調整する。

実際のテスト数は `cargo test` 実行後に確認し、tasks.md の完了条件を更新すること。

### Step 5: バージョン更新・完了

順序を守ること:
1. `fav/Cargo.toml` version → `"50.2.0"`
2. `cargo test` 通過確認（3095 または 3096）
3. `cargo clippy -- -D warnings` クリーン確認
4. `CHANGELOG.md` に v50.2.0 エントリ追加
5. `versions/current.md` 更新
6. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.2.0 実績を記入

---

## 注意事項

- `Diagnostic` struct に `data` フィールドを追加すると、`PartialEq` 比較を使っている
  既存テスト（`lsp/diagnostics.rs` の `converts_checker_error_to_zero_origin_diagnostic` 等）が
  コンパイルエラーになる可能性がある。
  → `data: None` を期待値に追加するか、`data` フィールドを無視する形に修正する。
- `errors_to_diagnostics` の既存テストも `Diagnostic` 構造体変更の影響を受ける。
  `lsp/diagnostics.rs` の `#[cfg(test)]` ブロックを確認し、必要に応じて `data: None` を追加。
- `error_catalog::lookup` は `pub fn` であり `lsp/diagnostics.rs` から直接呼び出せる。
- 本バージョンでは self-hosted ファイル（`compiler.fav` / `checker.fav`）への変更はないため、
  `fav lint` の self-lint 確認は不要。
