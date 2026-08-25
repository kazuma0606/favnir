# Spec: v81.7.0 — `fav quality report` コマンド

## Background

v81.1.0〜v81.6.0 で `QualityCheck` / `QualityScore` / `QualityGate` 等の品質基盤型が揃った。
v81.7.0 では、それらの実行結果を人間・機械双方が読める形式に整形する **レポート生成層** を追加する。

## Goals

- `ReportFormat` enum（`Text` / `Json` / `Markdown`）を追加する
- `QualityReportOptions` 構造体（`format`, `include_violations`, `include_stats`）を追加する
- `build_quality_report(check, rows, opts) -> String` でフォーマット別レポートを生成する
- `cmd_quality_report` で `build_quality_report` を薄くラップする

## API

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Text,
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct QualityReportOptions {
    pub format: ReportFormat,
    pub include_violations: bool,
    pub include_stats: bool,
}

/// `check` のルールを `rows` に適用し、`opts` の形式でレポートを返す。
pub fn build_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String

/// `build_quality_report` の薄いラッパー（CLI コマンドハンドラ相当）。
pub fn cmd_quality_report(
    check: &QualityCheck,
    rows: &[Vec<String>],
    opts: &QualityReportOptions,
) -> String
```

## 出力フォーマット仕様

### Text（`ReportFormat::Text`）

```
quality_report format=text violations={n}
```

`include_violations=true` の場合、各違反を改行で追記する:

```
- row={row} col={col} rule={rule}
```

### Json（`ReportFormat::Json`）

```
{"format":"json","violations":{n}}
```

`include_violations=true` の場合、`"items":[{"row":R,"col":"C","rule":"R"}]` を追記する（単純な文字列結合、serde 不使用）:

```
{"format":"json","violations":{n},"items":[{"row":0,"col":"1","rule":"NotNull"}]}
```

### Markdown（`ReportFormat::Markdown`）

```
## Quality Report
format: markdown
violations: {n}
```

`include_violations=true` の場合、Markdown リスト形式で追記する:

```
- row=0 col=1 rule=NotNull
```

## `include_stats` について

`include_stats: bool` フィールドは **将来拡張用スタブ**。
現バージョン（v81.7.0）の `build_quality_report` では参照せず、何も追記しない。
将来バージョンで分布統計（`DistributionStats`）の要約を追記する予定。

## 出力例（テスト向け擬似コード）

```rust
// Rust pseudocode (not Favnir)
// Text, include_violations=false
let opts = QualityReportOptions { format: ReportFormat::Text, include_violations: false, include_stats: false };
let out = build_quality_report(&check, &rows, &opts);
// out contains "text" and "violations=2"

// Json, include_violations=false
let opts_j = QualityReportOptions { format: ReportFormat::Json, include_violations: false, include_stats: false };
let out_j = build_quality_report(&check, &rows, &opts_j);
// out_j contains "json" and "violations"
```

## Success Criteria

- `cargo test` 3857 tests, 0 failures（3855 + 2）
- `quality_report_text_format`: Text 出力に `"text"` と `"violations"` が含まれることを確認
- `quality_report_json_format`: Json 出力に `"json"` と `"violations"` が含まれることを確認

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `ReportFormat` / `QualityReportOptions` / `build_quality_report` / `cmd_quality_report` 追加 |
| `fav/src/driver.rs` | `mod v81700_tests` 追加（2 テスト） |
| `CHANGELOG.md` | v81.7.0 エントリ追加 |
