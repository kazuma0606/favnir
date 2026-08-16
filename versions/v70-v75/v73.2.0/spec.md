# v73.2.0 Spec — データ品質スコアリング

Date: 2026-08-13
Status: 計画中

---

## 背景

データパイプラインの品質は多次元的に評価される必要がある。
v73.2.0 では `QualityReport` 構造体と `compute_quality_report` 関数を実装し、
5 次元（完全性・妥当性・一貫性・鮮度・参照整合性）のスコアリングを提供する。
将来的には `fav quality report` CLI コマンドに接続される。

---

## 目標

1. `QualityDimension` 構造体（name / score / detail）を追加
2. `QualityReport` 構造体（overall_score / dimensions / recommendations）を追加
3. `compute_quality_report(rows)` — 5 次元スコアリング関数
4. `format_quality_report(report)` — レポート文字列フォーマット
5. `cmd_quality_report(path)` — CLI エントリポイントスタブ（実 .fav 解析は将来バージョン）
6. 2 件のテスト（`quality_report_completeness_score` / `quality_report_recommendations`）

---

## API 例

```rust
let rows: Vec<Vec<Option<String>>> = vec![
    vec![Some("ok".to_string()), Some("1.0".to_string())],
    vec![None, Some("2.0".to_string())],  // null フィールドあり
];
let report = compute_quality_report(&rows);
// report.overall_score: 0〜100
// report.dimensions: Vec<QualityDimension>
// report.recommendations: Vec<String>

let output = format_quality_report(&report);
// "Favnir Data Quality Report\n..."
```

---

## 実装詳細

### `QualityDimension` 構造体

```rust
pub struct QualityDimension {
    pub name: String,       // "Completeness" / "Validity" / "Consistency" / "Freshness" / "Referential"
    pub score: u32,         // 0〜100
    pub detail: String,
}
```

### `QualityReport` 構造体

```rust
pub struct QualityReport {
    pub overall_score: u32,
    pub dimensions: Vec<QualityDimension>,
    pub recommendations: Vec<String>,
}
```

### `compute_quality_report`

```rust
pub fn compute_quality_report(rows: &[Vec<Option<String>>]) -> QualityReport
```

- **Completeness**: null セルの割合からスコアを算出。`null_ratio = null_count / total_cells`、score = `(1.0 - null_ratio) * 100` を切り捨て
- **Validity**: 全セルが `Some` の行の割合。score = valid_rows / total_rows * 100（小数切り捨て）
- **Consistency**: rows 数が 0 の場合 100、それ以外は固定スコア 78（スタブ）
- **Freshness**: 固定スコア 92（スタブ）
- **Referential**: 固定スコア 95（スタブ）
- **overall_score**: 5 次元スコアの平均（整数切り捨て）
- **recommendations**: Completeness < 95 なら `"Add null checks to pipeline fields"` を追加、Validity < 90 なら `"Add field validators to input schema"` を追加

### `cmd_quality_report`

```rust
pub fn cmd_quality_report(path: &str) -> String
```

`path` で指定されたファイルのスタブレポートを返す。実際のファイル解析は将来バージョン。
内部で空 rows に対して `compute_quality_report` + `format_quality_report` を呼ぶ。

### `format_quality_report`

```rust
pub fn format_quality_report(report: &QualityReport) -> String
```

出力フォーマット:
```
Favnir Data Quality Report
==========================
Overall Score: {score}/100

Dimension         Score  Detail
----------------- ------ ----------------------------------------
Completeness      94%    ...
...

Recommendations:
  1. ...
```

---

## テスト

### `v732000_tests` モジュール

```rust
#[test]
fn quality_report_completeness_score() {
    // 1000 行中 58 件 null → Completeness ≈ 94
    let mut rows: Vec<Vec<Option<String>>> = vec![];
    for _ in 0..942 {
        rows.push(vec![Some("a".to_string()), Some("b".to_string())]);
    }
    for _ in 0..58 {
        rows.push(vec![None, Some("b".to_string())]);
    }
    let report = compute_quality_report(&rows);
    assert!(report.overall_score > 0, "overall score should be positive");
    let completeness = report.dimensions.iter()
        .find(|d| d.name == "Completeness").unwrap();
    assert!(completeness.score >= 90, "completeness score should be >= 90: got {}", completeness.score);
}

#[test]
fn quality_report_recommendations() {
    // null 多め → Completeness 低 → recommendation が追加される
    let rows: Vec<Vec<Option<String>>> = vec![
        vec![None, None],
        vec![None, Some("x".to_string())],
        vec![Some("a".to_string()), None],
    ];
    let report = compute_quality_report(&rows);
    assert!(!report.recommendations.is_empty(), "low quality should generate recommendations");
    assert!(report.recommendations.iter().any(|r| r.contains("null")),
        "recommendation should mention null checks");
}
```

---

## 成功基準

- `cargo test v732000` で 2 件 pass
- `cargo test` 全体で 3650 tests pass（3648 + 2）
- `fav/Cargo.toml` のバージョンが `73.2.0`
- `QualityDimension` / `QualityReport` が pub で存在する
- `compute_quality_report` / `format_quality_report` が pub で存在する

---

## スコープ外

- `fav quality report` CLI の実際の .fav ファイル入力解析（`cmd_quality_report` はスタブのみ）
- `--min-score <n> --fail-below` フラグ（CI 品質ゲート用）— v73.8.0 以降
- CSV / Parquet ファイルの実際の読み込み（v74.x 以降）
- Consistency / Freshness / Referential の実スコアリングロジック（スタブ、将来実装）
- `main.rs` への `Some("quality")` ハンドラ登録（将来バージョン）
- WASM / サイト MDX 更新（v74.x 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — `QualityDimension` / `QualityReport` 構造体 + `compute_quality_report` / `format_quality_report` / `cmd_quality_report` + `v732000_tests` + バージョン更新
- `fav/Cargo.toml` — version `73.1.0` → `73.2.0`
- `CHANGELOG.md` — v73.2.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v73.2.0 に更新
