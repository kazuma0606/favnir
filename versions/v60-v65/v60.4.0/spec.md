# v60.4.0 Spec — LSP Diagnostic 完全統合（全エラーコードの位置情報付与）

Date: 2026-07-30
Status: 計画中

---

## 概要

LSP `textDocument/publishDiagnostics` はすでに span 付き diagnostic を送出済み（実装済み）。
`fav check --json` の出力に `"span"` サブオブジェクトフィールドを追加し、
ロードマップが要求する JSON 形式に準拠させる。

---

## 既存実装の状況

| コンポーネント | 状態 | 備考 |
|---|---|---|
| `lsp/diagnostics.rs` の `errors_to_diagnostics` | **実装済み** | `TypeError.span` → `Diagnostic.range`（LSP 座標系）に変換。`converts_checker_error_to_zero_origin_diagnostic` テストで動作保証済み |
| `lsp/mod.rs` の `publish_diagnostics` | **実装済み** | `textDocument/publishDiagnostics` で位置情報付き diagnostic を送出。`lsp_capabilities_include_code_action` 等の LSP 統合テストで動作保証済み |
| `driver.rs` の `CheckDiagnostic` | **部分実装** | `file`/`line`/`col` フラットフィールドはあるが `"span"` サブオブジェクトがない |
| `driver.rs` の `type_error_to_diag` | **更新必要** | `span` フィールドを追加する |

---

## ロードマップとの差分

| ロードマップ記述 | 実際のスコープ | 理由 |
|---|---|---|
| E0001〜E0426 に span 付与 | `TypeError` はすでに `span` フィールドを持つ | span 付与自体は既存；JSON 出力フォーマットの追加が実際の作業 |
| LSP publishDiagnostics 更新 | 実装済みにつき確認テストのみ | v50.2.0 で完成済み |
| `fav check --json` に `"span"` 追加 | **実装する** | `SpanOutput` 構造体 + `CheckDiagnostic.span` フィールド追加 |

`type_warning_to_diag`（W コード用）への `span` 追加はスコープ外。
ロードマップ記述は error コード（E0001〜E0426）のみ言及している。

---

## 実装詳細

### 追加: `SpanOutput` 構造体（`driver.rs`）

`CheckDiagnostic` の直前に追加する。

```rust
/// v60.4.0: `fav check --json` の span サブオブジェクト
#[derive(serde::Serialize)]
struct SpanOutput {
    file: String,
    line: u32,
    col:  u32,
    len:  u32,
}
```

`len` は `span.end.saturating_sub(span.start)`（バイト長）。

### 変更: `CheckDiagnostic` に `span` フィールド追加

```rust
#[derive(serde::Serialize)]
struct CheckDiagnostic {
    code:       String,
    message:    String,
    file:       String,   // 後方互換のため残す
    line:       u32,      // 後方互換のため残す
    col:        u32,      // 後方互換のため残す
    span:       SpanOutput,   // v60.4.0: 追加
    suggestion: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hints:      Vec<String>,
}
```

### 変更: `type_error_to_diag` に `span` フィールド追加

```rust
fn type_error_to_diag(e: &crate::middle::checker::TypeError, suggestion: &str) -> CheckDiagnostic {
    CheckDiagnostic {
        code:       e.code.to_string(),
        message:    e.message.clone(),
        file:       e.span.file.clone(),
        line:       e.span.line,
        col:        e.span.col,
        span:       SpanOutput {
            file: e.span.file.clone(),
            line: e.span.line,
            col:  e.span.col,
            len:  e.span.end.saturating_sub(e.span.start),
        },
        suggestion: suggestion.to_string(),
        hints:      e.hints.clone(),
    }
}
```

---

## テスト仕様

### `check_json_includes_span`

- `TypeError` を生成し `type_error_to_diag` でシリアライズ（`Span::new("test.fav", 0, 7, 1, 4)` → `len=7`）
- 出力 JSON を `serde_json::Value` にパースして `json["span"]["line"].is_number()` を assert
  ※ `json.contains("\"line\"")` では既存フラットフィールドでも通過するため、`span` サブオブジェクト内の値を検証する

### `lsp_diagnostic_has_span`

- `TypeError` にスパン情報（line=3, col=7）を設定し `errors_to_diagnostics` を呼ぶ
- `src` の 3 行目は 10 文字以上（`"       foo"` など）にし、`span_to_range` のクランプが発生しないようにする
  ※ `span_to_range` は `start_char = span.col.saturating_sub(1).min(line_len)` でクランプするため、
     3 行目が `"  foo"`（5 文字）だと col=7 → `min(6,5)=5` となり `character==6` アサートが FAIL する
- 返却 `Diagnostic` の `range.start.line == 2`（0-indexed）を assert
- `range.start.character == 6`（0-indexed）を assert

`len` はバイト長（`end - start`）。マルチバイト文字含む識別子では文字数と異なる場合がある。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `SpanOutput` 構造体追加、`CheckDiagnostic.span` 追加、`type_error_to_diag` 更新、`v60400_tests` 追加 |

`lsp/diagnostics.rs` / `lsp/mod.rs` への変更なし。

---

## 完了条件

- `cargo test` 全通過（3336 → **3338** tests passed, 0 failed）
- 以下の 2 テストが pass:
  - `v60400_tests::check_json_includes_span`
  - `v60400_tests::lsp_diagnostic_has_span`

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v60.1-v61.0.md`（v60.4.0 セクション）
- 既存実装: `fav/src/lsp/diagnostics.rs`（LSP diagnostic 変換）
- 既存実装: `fav/src/driver.rs` L3927–3980（`CheckDiagnostic` / `type_error_to_diag`）
- 次バージョン: v60.5.0 — `fav repl` 強化
