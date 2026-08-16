# v70.4.0 Spec — 構造化エラー診断

Date: 2026-08-09
Status: 計画中

---

## Background

`fav check` / `fav run` が返すエラーメッセージは現状プレーンな文字列（`"E0001: undefined variable 'x'"` 等）。ユーザーが「次に何をすべきか」を即座に判断できるレベルではない。

**既存の関連実装:**
- `strsim = "0.11"` が Cargo.toml に登録済み（Levenshtein 距離計算に使用可能）
- `suggest.rs` の `builtin_hint(error_code)` — 単純な文字列 hint のみ
- `error_catalog.rs` — エラーコードの文字列定義

**未実装:**
- `ErrorReport` 構造体（code / span / message / hint / suggestion / doc_url）
  - ロードマップの `span` は `line / col / source_line / span_len` の 4 フィールドとして展開する
- `suggest_similar_name` — Levenshtein 距離によるタイポ候補検出
- `format_diagnostic` — 構造化テキスト出力（rustc スタイル）

**v70.4 スコープ外（将来バージョンに先送り）:**
- カラー付きターミナル出力（ANSI エスケープ）— v70.5 以降で `colored: bool` パラメータを追加
- LSP JSON 出力（`to_lsp_diagnostic()` 相当）— v70.5 以降で `lsp/` モジュールと統合
- 修正後コードブロック（`corrected_line: Option<String>`）— v70.5 以降
- `ErrorReport` の `src/diagnostic.rs` への分離 — v70.5 で実施（v70.4 では driver.rs に暫定配置）

---

## Goals

1. `ErrorReport` 構造体を定義し、`format_diagnostic` で rustc スタイルの診断メッセージを生成する
2. `suggest_similar_name` が Levenshtein 距離 ≤ 3 の候補を返す（`strsim::levenshtein` を使用）
3. E0374（`!Effect` 廃止）に対して `ctx: AppCtx` 追加 + `fav migrate` 案内のヒントを返す
4. E0001（未定義変数）に対して `suggest_similar_name` でタイポ候補を提示する
5. 新規 Rust テスト 2 件追加 → 3567 tests

---

## Syntax / API Examples

```
error[E0374] benchmarks/compare.fav:43:62
  |
43| fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO {
  |                                                              ^^^^
  | `!Effect` アノテーション構文は v35.4.0 で廃止されました
  |
  = ヒント: `ctx: AppCtx` を第1引数として追加し、`!IO` を削除してください
  = 自動移行: fav migrate --from v35 --in-place <file>
  = 参照: https://favnir.dev/docs/language/ctx-migration

error[E0001] pipeline.fav:12
  |
12|     bind result <- process(ordr)
  |                            ^^^^
  | 未定義変数 `ordr`
  |
  = ヒント: `order` のことですか？（3文字以内の編集距離）
```

---

## ErrorReport 構造体

```rust
pub struct ErrorReport {
    pub code: &'static str,      // "E0001"
    pub file: String,            // "pipeline.fav"
    pub line: usize,             // 1-indexed
    pub col: usize,              // 1-indexed
    pub source_line: String,     // ソース行テキスト
    pub span_len: usize,         // アンダーライン長
    pub message: String,         // エラー本文
    pub hint: Option<String>,    // = ヒント: ...
    pub suggestion: Option<String>, // = 自動移行: ...
    pub doc_url: Option<String>, // = 参照: https://...
}
```

---

## 関数仕様

### `suggest_similar_name(name: &str, candidates: &[&str]) -> Option<String>`

- `strsim::levenshtein(name, c)` ≤ 3 の候補を収集
- 距離が最小のものを 1 件返す（同距離の場合は辞書順で最初）
- 候補がなければ `None`

### `format_diagnostic(report: &ErrorReport) -> String`

rustc スタイルの診断テキストを生成:
```
error[{code}] {file}:{line}:{col}
  |
{line}| {source_line}
  |{underline}
  | {message}
  |
  = ヒント: {hint}
  = 自動移行: {suggestion}
  = 参照: {doc_url}
```

`hint` / `suggestion` / `doc_url` が `None` の場合はその行を省略。

### `build_e0374_report(file, line, col, source_line, effect_name) -> ErrorReport`

E0374 専用のレポートビルダー。`hint` に `ctx: AppCtx` 追加の案内、`suggestion` に `fav migrate --from v35` コマンドを設定。

### `build_e0001_report(file, line, col, source_line, var_name, candidates) -> ErrorReport`

E0001 専用のレポートビルダー。`suggest_similar_name(var_name, candidates)` の結果を `hint` に設定。

---

## Success Criteria

- [ ] `ErrorReport` 構造体が `fav/src/driver.rs` に定義されている
- [ ] `suggest_similar_name("ordr", &["order", "other"])` が `Some("order")` を返す
- [ ] `format_diagnostic` が `error[E0374]` で始まるテキストを返す
- [ ] `cargo test v704000` で 2 件 pass
- [ ] `cargo test` 全体で 3567 tests pass

---

## Error Codes

新規エラーコードなし（E0374 / E0001 は既存）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `ErrorReport` 構造体・`suggest_similar_name`・`format_diagnostic`・`build_e0374_report`・`build_e0001_report` 追加、`v704000_tests` モジュール追加 |
| `fav/Cargo.toml` | `version` を `"70.3.0"` → `"70.4.0"` に更新 |
| `CHANGELOG.md` | v70.4.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.4.0 に更新 |
