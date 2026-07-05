# v33.8.0 — Spec

## 概要

**テーマ**: プロファイリング強化 確認（`fav profile --flamegraph` / フォールデッドスタック）

**方針**: 確認・記録パターン。v19.8.0 で実装済みのプロファイリング強化機能を `v338000_tests` 4 件で確認する。新規コードは追加しない。

---

## 背景

v19.8.0 で実装済み:
- `fav profile --flamegraph` — SVG flamegraph 出力
- `profiler::collector::{StageRecord, parse_profile_json, to_folded_stacks}` — プロファイル JSON 解析・フォールデッドスタック変換
- `profiler::flamegraph::generate_svg` — SVG 生成
- `profiler::report::{format_json_report, format_text_report}` — レポート整形

v198000_tests（既存）でカバー済み:
- `profile_flamegraph_generates_svg`
- `profile_text_output`
- `profile_json_output`
- `profile_hot_path_detected`

v338000_tests では **JSON パーサーとフォールデッドスタック変換** をより直接的にカバーする。

---

## 実装スコープ

### 変更ファイル
1. `fav/Cargo.toml` — version `33.7.0` → `33.8.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_33_7_0` をスタブ化、`v338000_tests` 4 件追加
3. `benchmarks/v33.8.0.json` — 新規作成
4. `CHANGELOG.md` — `[v33.8.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v33.8.0 に更新

### 新規ファイル
- `versions/v30-v35/v33.8.0/` — spec.md / plan.md / tasks.md

---

## テスト仕様（v338000_tests）

```rust
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod v338000_tests {
    use crate::profiler::collector::{StageRecord, parse_profile_json, to_folded_stacks};

    #[test]
    fn cargo_toml_version_is_33_8_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("33.8.0"), "Cargo.toml must contain '33.8.0'");
    }

    #[test]
    fn benchmark_v33_8_0_exists() {
        let src = include_str!("../../benchmarks/v33.8.0.json");
        assert!(src.contains("33.8.0"), "benchmarks/v33.8.0.json must contain '33.8.0'");
    }

    #[test]
    fn profile_parse_json_valid_records() {
        // parse_profile_json: JSON 文字列 → Vec<StageRecord> を確認
        // JSON キーは "name" / "ms"（StageRecord.name / StageRecord.elapsed_ms に対応）
        let json = r#"[{"name":"Load","ms":10},{"name":"Transform","ms":25}]"#;
        let records = parse_profile_json(json);
        assert_eq!(records.len(), 2, "expected 2 StageRecords");
        assert_eq!(records[0].name, "Load");
        assert_eq!(records[0].elapsed_ms, 10);
        assert_eq!(records[1].name, "Transform");
        assert_eq!(records[1].elapsed_ms, 25);
    }

    #[test]
    fn profile_folded_stacks_has_pipeline_prefix() {
        // to_folded_stacks: Vec<StageRecord> → Vec<String> を確認
        // 各エントリが "pipeline;<name>" で始まること
        let records = vec![
            StageRecord { name: "Load".to_string(), elapsed_ms: 10 },
            StageRecord { name: "Transform".to_string(), elapsed_ms: 25 },
        ];
        let folded = to_folded_stacks(&records);
        assert!(
            folded.iter().all(|line| line.starts_with("pipeline;")),
            "all folded stack entries must start with 'pipeline;'"
        );
    }
}
```

### 設計注記
- `#[cfg(test)]` → `#[cfg(not(target_arch = "wasm32"))]` の順（既存 v198000_tests スタイルに合わせる）
- `profiler` モジュールは `lib.rs` で `#[cfg(not(target_arch = "wasm32"))]` ゲート済み。v337000_tests は profiler を import しないため cfg gate なし; v338000_tests は profiler を使うため必須
- `parse_profile_json` は `Vec<StageRecord>` を直接返す（パース失敗時は空ベクタ）。`Result` でラップされていないため `.expect()` 不要
- `StageRecord` フィールド: `name: String`, `elapsed_ms: i64`（`stage`/`duration_ms` ではない）
- `to_folded_stacks` は `&[StageRecord]` を受け取り `Vec<String>` を返す（`String` ではない）
- JSON キーは `"name"` / `"ms"`（serde の Raw struct フィールドに対応）
- v198000_tests との重複なし（v198000 は struct から始まる; v338000 は JSON 解析から開始）

---

## 完了条件

- [ ] `Cargo.toml` version = `"33.8.0"`
- [ ] `cargo_toml_version_is_33_7_0` が空スタブ
- [ ] `cargo test --bin fav v338000` — 4/4 PASS
- [ ] `cargo test` — 全件 PASS（2528 件、0 failures）
- [ ] `CHANGELOG.md` に `[v33.8.0]` セクション
- [ ] `benchmarks/v33.8.0.json` 存在かつ `tests_passed` が実測値
- [ ] `benchmarks/v33.8.0.json` の `milestone` フィールドが `"Performance & Tooling"`
- [ ] `versions/current.md` を v33.8.0 に更新
