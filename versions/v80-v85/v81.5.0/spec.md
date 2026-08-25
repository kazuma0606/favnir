# Spec: v81.5.0 — 来歴付き品質レポート（Provenance + Quality 統合）

## Background

v81.4.0 で `QualityScore` / `quality_grade` を導入した。
本バージョンでは「どのソースから来たデータがどの品質スコアを持つか」を追跡する
来歴付き品質レポート型を追加する。

`provenance_hash` は Favnir 3.0 の `ProvenanceTag` との将来的な統合を見越したフィールドだが、
本バージョンでは `String` スタブとして扱う（実際のハッシュ計算は行わない）。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.5.0 セクション）

> **テスト数**: 実際のベースは **3851**（v81.4.0 完了後）。
> 本バージョンの完了条件は **3851 + 2 = 3853**。

## Goals

- `ProvenanceQualityEntry` 構造体を `test_framework.rs` に追加する
- `ProvenanceQualityReport` 構造体を追加する
- `build_provenance_quality_report(entries: Vec<ProvenanceQualityEntry>, pipeline: &str) -> ProvenanceQualityReport` を実装する
- `format_provenance_quality_report(report: &ProvenanceQualityReport) -> String` を実装する
- `worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry>` を実装する
- テスト 2 件を追加して **3853 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// ソース単位の来歴付き品質エントリ。
///
/// `provenance_hash` は本バージョンでは `String` スタブ（Favnir 3.0 連携は将来対応）。
/// `quality_score` は 0.0〜1.0 の範囲。
#[derive(Debug, Clone)]
pub struct ProvenanceQualityEntry {
    pub source_name: String,
    pub provenance_hash: String,
    pub quality_score: f64,
}

/// パイプライン単位の来歴付き品質レポート。
#[derive(Debug)]
pub struct ProvenanceQualityReport {
    pub entries: Vec<ProvenanceQualityEntry>,
    pub pipeline_name: String,
}

/// `entries` と `pipeline` からレポートを構築する。
pub fn build_provenance_quality_report(
    entries: Vec<ProvenanceQualityEntry>,
    pipeline: &str,
) -> ProvenanceQualityReport;

/// レポートを人間向けの文字列に変換する。
///
/// 出力形式:
/// ```
/// pipeline={pipeline_name} sources={count}
/// - {source_name}: score={quality_score:.3} hash={provenance_hash}
/// - ...
/// ```
pub fn format_provenance_quality_report(report: &ProvenanceQualityReport) -> String;

/// `quality_score` が最も低いエントリを返す。
/// `entries` が空のとき `None` を返す。
pub fn worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry>;
```

### 出力例

```text
// 概念説明（Favnir 風疑似コード）
bind entries <- vec![
    ProvenanceQualityEntry { source_name: "db_A".to_string(), provenance_hash: "abc123".to_string(), quality_score: 0.95 },
    ProvenanceQualityEntry { source_name: "api_B".to_string(), provenance_hash: "def456".to_string(), quality_score: 0.72 },
];
bind report <- build_provenance_quality_report(entries, "my_pipeline");
// report.pipeline_name == "my_pipeline"
// report.entries.len() == 2

bind worst <- worst_quality_source(&report);
// worst == Some(&ProvenanceQualityEntry { source_name: "api_B", quality_score: 0.72, ... })

bind formatted <- format_provenance_quality_report(&report);
// "pipeline=my_pipeline sources=2\n- db_A: score=0.950 hash=abc123\n- api_B: score=0.720 hash=def456"
```

## Success Criteria

- `cargo test` が **3853 tests**, 0 failures
- `provenance_quality_report_built`:
  - 2 件のエントリで `build_provenance_quality_report` を呼んで `pipeline_name` と `entries.len()` を確認する
  - `format_provenance_quality_report` の出力に `"pipeline=my_pipeline"` と `"sources=2"` と両 `source_name` が含まれることを確認する
- `worst_source_identified`:
  - スコアの低い方のエントリが `worst_quality_source` で返ることを確認する（`source_name` で判定）
  - 空 entries のとき `None` が返ることを確認する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `ProvenanceQualityEntry` / `ProvenanceQualityReport` / `build_provenance_quality_report` / `format_provenance_quality_report` / `worst_quality_source` |
| `fav/src/driver.rs` | 追記 | `mod v81500_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `worst_quality_source` は `entries` を `quality_score` で比較して最小値を返す。スコアが同値の場合は先に現れたエントリを返す（`f64::min_by` / partial_ord の fold）。`quality_score` に `NaN` を渡した場合の動作は未定義。
- `format_provenance_quality_report` は各エントリを `\n` 区切りで列挙する。
- `provenance_hash` は現バージョンでは呼び出し元が任意の文字列を渡す（SHA256 等の計算は行わない）。
