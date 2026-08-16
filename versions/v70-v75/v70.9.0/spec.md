# v70.9.0 Spec — 安定化・コードフリーズ（Language Complete 前調整）

Date: 2026-08-09
Status: 計画中

---

## Background

v70.1〜v70.8 で以下の機能を追加した:
- v70.1: compiler.fav 2 段メソッドチェーン対応
- v70.2: `fav migrate`（IO.* → ctx.io.* 変換）
- v70.3: `fav bench --all`
- v70.4: ErrorReport / `suggest_similar_name` 診断 UI
- v70.5: パターンマッチ強化（nested record / or-pattern / if-guard）
- v70.6: `bind` 分割束縛拡張（Record / List スプレッド）
- v70.7: Self-Hosting Coverage Report（`fav self-coverage`）
- v70.8: `fav doctor` 強化（Paper Rune 検出 / CHANGELOG 整合性）

本バージョンは v71.0.0 "Language Complete 1.0" 宣言に向けた**コードフリーズ**版。
全機能の動作確認と bench.yml の `|| true`（continue-on-error 相当）除去を行う。

---

## Goals

1. **`language_complete_all_stable`** テスト — v70.1〜v70.8 の代表テスト名が driver.rs に存在することを確認
2. **`bench_ci_no_continue_on_error`** テスト — bench.yml の Compare ステップが strict mode（`|| true` なし）であることを確認
3. **bench.yml 修正** — Compare ステップの `|| true` を除去（CI がリグレッション時に失敗するようにする）
4. テスト 2 件追加（3578 → 3580）

---

## 実装詳細

### `language_complete_all_stable`

driver.rs 内のテストモジュール名を列挙し、v70.1〜v70.8 の代表テストが存在することを確認する。

```rust
fn language_complete_all_stable() {
    // v70.1〜v70.8 の代表テスト関数名が driver.rs ソースに含まれることを確認
    let src = include_str!("driver.rs");
    let required = [
        "backlog_compiler_fav_ctx_multiparams",   // v70.1
        "migrate_effect_annotation_to_ctx",        // v70.2
        "bench_subcommand_all_outputs_json",       // v70.3
        "diagnostic_e0374_shows_migration_hint",   // v70.4
        "pattern_match_if_guard",                  // v70.5
        "bind_destructure_record",                 // v70.6
        "self_coverage_compiler_fav_above_95pct",  // v70.7
        "doctor_detects_paper_rune",               // v70.8
    ];
    for name in &required {
        assert!(src.contains(name), "missing test: {name}");
    }
}
```

### `bench_ci_no_continue_on_error`

bench.yml の Compare ステップが `|| true` を含まないことを確認する。

```rust
fn bench_ci_no_continue_on_error() {
    let yml = include_str!("../../.github/workflows/bench.yml");
    // Compare ステップの run コマンドが || true を含まないことを確認
    let compare_block = yml
        .split("Compare with baseline")
        .nth(1)
        .expect("Compare ステップが bench.yml に存在しない");
    // 次のステップ境界（"- name:"）まで抽出
    let compare_step = compare_block
        .split("- name:")
        .next()
        .unwrap_or(compare_block);
    assert!(
        !compare_step.contains("|| true"),
        "Compare ステップに || true が残存している（strict mode に変更してください）"
    );
}
```

### bench.yml 修正

`Compare with baseline` ステップの行末 `|| true`（シェルオペレータ）を除去する。
`continue-on-error: true`（YAML キー）は bench.yml に存在しないため、除去対象ではない。

```yaml
# 変更前
$FAV run benchmarks/compare.fav \
  -- --baseline benchmarks/v24.2.0.json \
     --current  benchmarks/latest.json \
     --threshold 5 \
     --emit-md || true

# 変更後
$FAV run benchmarks/compare.fav \
  -- --baseline benchmarks/v24.2.0.json \
     --current  benchmarks/latest.json \
     --threshold 5 \
     --emit-md
```

---

## Success Criteria

- [ ] `language_complete_all_stable`: v70.1〜v70.8 の代表テスト名 8 件が driver.rs に存在することを assert
- [ ] `bench_ci_no_continue_on_error`: bench.yml の Compare ステップに `|| true` がないことを assert
- [ ] `cargo test v709000` で 2 件 pass
- [ ] `cargo test` 全体で 3580 tests pass（0 failures）
- [ ] `fav/Cargo.toml` が `70.9.0` に更新されていること

---

## Error Codes

新規エラーコードなし

---

## Notes

- 安定化版のため `site/` MDX 更新は不要（新規ドキュメントは v71.0.0 で実施）
- bench.yml の `|| true` 除去は Compare ステップのみ（Run benchmarks / Regression check は触れない）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v709000_tests` モジュール追加（`language_complete_all_stable` / `bench_ci_no_continue_on_error`） |
| `.github/workflows/bench.yml` | Compare ステップの `|| true` を除去 |
| `fav/Cargo.toml` | `version` を `"70.8.0"` → `"70.9.0"` |
| `CHANGELOG.md` | v70.9.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.9.0 に更新 |
