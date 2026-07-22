# Spec: v53.2.0 — bench × par 統合（par stage 個別計測）

Status: 計画中
Date: 2026-07-22

---

## 概要

`fav bench` の出力・JSON スキーマに `par` ブロック内 stage 名を追加する。
`FlwDef.steps` から `FlwStep::Par` / `FlwStep::ParDistributed` を静的解析し、
`BenchStats.par_stages: Vec<String>` に stage 名一覧を格納する。
`bench_stats_to_json` を更新して `benchmarks/<version>.json` に `par_stages` フィールドを追加する。

> ロードマップには個別タイミング（`par.Enrich: 12.3ms` 等）の表示例があるが、
> 実際に個別 stage を並列実行スレッドから個別計測するには VM スレッド計装の大規模改修が必要。
> v53.2.0 では「静的解析による stage 名検出 + JSON スキーマへの `par_stages` 追加」に絞る。
> 個別タイミング・ボトルネック表示は将来バージョン（v54.x）の課題とする。

---

## 実装スコープ

### 1. `collect_par_stage_names(program: &ast::Program) -> Vec<String>`

`driver.rs` の bench セクションに追加する公開関数。
`program.items` を走査し `Item::FlwDef` を見つけ、
`FlwDef.steps` 内の `FlwStep::Par(names)` / `FlwStep::ParDistributed(names)` から
stage 名を収集して返す（重複なし・出現順）。

```rust
pub fn collect_par_stage_names(program: &ast::Program) -> Vec<String> {
    let mut names = Vec::new();
    for item in &program.items {
        if let ast::Item::FlwDef(flw) = item {
            for step in &flw.steps {
                match step {
                    ast::FlwStep::Par(ns) | ast::FlwStep::ParDistributed(ns) => {
                        for n in ns {
                            if !names.contains(n) {
                                names.push(n.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    names
}
```

### 2. `BenchStats` に `par_stages: Vec<String>` フィールド追加

```rust
pub struct BenchStats {
    // 既存フィールド...
    pub par_stages: Vec<String>,  // v53.2.0: par ブロック内 stage 名一覧
}
```

`compute_bench_stats` は `par_stages: vec![]` で初期化する。
`cmd_bench` の bench case 処理ループで、`compute_bench_stats` 呼び出し後に
`stats.par_stages = collect_par_stage_names(prog);` を付加する。

### 3. `bench_stats_to_json` 更新

`par_stages` フィールドを JSON に追加:

```rust
serde_json::json!({
    "name":       s.name,
    "runs":       s.runs,
    "avg_us":     s.avg_us,
    "p50_us":     s.p50_us,
    "p95_us":     s.p95_us,
    "min_us":     s.min_us,
    "max_us":     s.max_us,
    "par_stages": s.par_stages,  // v53.2.0: 追加
})
```

### 4. テスト仕様

`v53200_tests` モジュールを `driver.rs` に追加（`v53100_tests` の直前）:

```rust
#[cfg(test)]
mod v53200_tests {
    #[test]
    fn bench_par_stage_individual() {
        use crate::ast;
        use crate::frontend::parser::Parser;
        use crate::driver::collect_par_stage_names;
        let source = r#"
stage Enrich: Int -> Int = |n| { n }
stage Validate: Int -> Int = |n| { n }
seq pipeline = par [Enrich, Validate] |> Merge.ordered
"#;
        let program = Parser::parse_str(source, "t.fav").expect("parse");
        let names = collect_par_stage_names(&program);
        assert!(names.contains(&"Enrich".to_string()), "must detect Enrich");
        assert!(names.contains(&"Validate".to_string()), "must detect Validate");
    }

    #[test]
    fn bench_par_stage_total() {
        use crate::driver::{BenchStats, bench_stats_to_json};
        let stats = BenchStats {
            name: "parallel pipeline".to_string(),
            runs: 10,
            avg_us: 18900.0,
            p50_us: 18800.0,
            p95_us: 19500.0,
            min_us: 18000.0,
            max_us: 20000.0,
            par_stages: vec!["Enrich".to_string(), "Validate".to_string()],
        };
        let json = bench_stats_to_json(&[stats]);
        assert!(json.contains("par_stages"), "JSON must include par_stages field");
        assert!(json.contains("Enrich"), "JSON must include Enrich in par_stages");
        assert!(json.contains("Validate"), "JSON must include Validate in par_stages");
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `"53.1.0"` → `"53.2.0"`

---

## 完了条件

- `cargo test` 3167 passed, 0 failed（3165 + 2 件追加）
  （ベース 3165 = v53.1.0 完了時実績。ロードマップ推定値 3161 は v53.1.0 コードレビュー修正前の値）
- `v53200_tests` 2 件 pass:
  - `bench_par_stage_individual`
  - `bench_par_stage_total`
- `cargo clippy -- -D warnings` クリーン

---

## 影響範囲

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `BenchStats` に `par_stages` 追加、`compute_bench_stats` 初期化、`bench_stats_to_json` 更新、`collect_par_stage_names` 追加、`v53200_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v53.2.0 エントリ追加 |
| `versions/current.md` | v53.2.0 / 3167 tests に更新 |
| `versions/roadmap/roadmap-v53.1-v54.0.md` | v53.2.0 実績欄を COMPLETE に更新 |

---

## 設計上の注意

- `par_stages` の重複除去: `!names.contains(n)` でグローバルに重複除去する（FlwDef 境界・同一 FlwDef 内の重複を問わず、プログラム全体で最初の出現のみ収集する）
- `bench_stats_to_compare_json`（regression compare 用）は `par_stages` 非対象 — `metrics` マップは stage 名をキーに avg_ms を値とする形式のため構造が異なる。変更不要
- `cmd_bench` の `--json` 出力パス（`bench_stats_to_json`）のみ更新対象
- wasm32 影響なし（`cmd_bench` は `#[cfg(not(wasm32))]` 相当のパス）
- `FlwStep::Par` と `FlwStep::ParDistributed` の両方を対象とする
- `compute_bench_stats` の既存シグネチャは変えない（`par_stages` は `cmd_bench` 側で後付け）
- **`BenchStats` 直接構築箇所**: `v176000_tests::bench_json_output`（driver.rs 行 32061）が `BenchStats { ... }` リテラルを使用している。`par_stages` 追加後はここに `par_stages: vec![]` を追加しないとコンパイルエラーになる。T2 で明示的に対処すること
- `bench_stats_to_compare_json` は `BenchStats` フィールドを直接参照せず `metrics` マップを構築するため変更不要
