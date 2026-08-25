# Tasks: v81.5.0 — 来歴付き品質レポート（Provenance + Quality 統合）

> COMPLETE — 2026-08-19
> 3853 tests, 0 failures（+2 from 3851）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3851 tests, 0 failures を確認する
- [x] `Cargo.toml` の `version` フィールドが `81.0.0` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。このバージョン完了後も `81.0.0` のまま変更しない。v82.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v81400_tests` が存在することを確認する（v81.4.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `quality_grade` が定義済みであることを確認する

## T1: `fav/src/test_framework.rs` に追記

- [x] `ProvenanceQualityEntry` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `source_name: String`, `provenance_hash: String`, `quality_score: f64`
- [x] `ProvenanceQualityReport` 構造体（`#[derive(Debug)]`）を追加する
  - フィールド: `entries: Vec<ProvenanceQualityEntry>`, `pipeline_name: String`
- [x] `build_provenance_quality_report(entries: Vec<ProvenanceQualityEntry>, pipeline: &str) -> ProvenanceQualityReport` を実装する
  - `pipeline.to_string()` で `pipeline_name` を設定する
- [x] `format_provenance_quality_report(report: &ProvenanceQualityReport) -> String` を実装する
  - ヘッダ: `"pipeline={name} sources={count}"`
  - 各エントリ: `"\n- {source_name}: score={:.3} hash={provenance_hash}"`
- [x] `worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry>` を実装する
  - `entries.iter().reduce(|worst, e| if e.quality_score < worst.quality_score { e } else { worst })` で最小スコアを返す
  - 空のとき `None`

## T2: `fav/src/driver.rs` に `mod v81500_tests` を追加

- [x] `mod v81400_tests { ... }` の直後に `#[cfg(test)] mod v81500_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `provenance_quality_report_built` テストを実装する
  - 2 件のエントリ（`db_A` / `api_B`）で `build_provenance_quality_report` を呼ぶ
  - `pipeline_name == "my_pipeline"` と `entries.len() == 2` を確認する
  - `format_provenance_quality_report` の出力に `"pipeline=my_pipeline"` / `"sources=2"` / `"db_A"` / `"api_B"` が含まれることを確認する
- [x] `worst_source_identified` テストを実装する
  - スコア 3 件（0.95 / 0.40 / 0.75）で `worst_quality_source` が `"low"` を返すことを確認する
  - 同値スコア 2 件で先頭エントリが返ることを確認する（spec-reviewer [LOW] 指摘対応）
  - 空 entries で `None` が返ることを確認する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3853 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.5.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
