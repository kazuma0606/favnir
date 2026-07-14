# v43.6.0 実装計画 — パイプライン型伝播（Pipeline stage typing）

## 前提

- v43.5.0 完了（2917 tests）
- `fav/Cargo.toml` version: `43.5.0`
- `checker.fav` に変更不要: `infer_hm_let` + `infer_list_lambda_call`（v43.5.0）の組み合わせで多段パイプラインが既に機能する
- `bind _ <-` は短絡しない → `Result.and_then` / `match { Ok(...) => ... Err(e) => ... }` を使うこと（v43.4.0/v43.5.0 で判明）

---

## タスク順序

```
T0 事前確認
T1 driver.rs — v43600_tests 追加（v43500_tests の直前）
T2 Cargo.toml — version 43.5.0 → 43.6.0 + v43500_tests スタブ化
T3 CHANGELOG.md — v43.6.0 エントリ追加
T4 cargo test 実行・確認（2920 pass, 0 fail）
T5 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` 2917 / 0 確認
2. `Cargo.toml` version = `43.5.0` 確認
3. `v43600_tests` が driver.rs に存在しないことを確認

---

## T1 — driver.rs — v43600_tests

`v43500_tests` モジュールの直前に挿入:

```rust
// -- v43600_tests (v43.6.0) -- パイプライン型伝播（Pipeline stage typing）--
#[cfg(test)]
mod v43600_tests {
    #[test]
    fn cargo_toml_version_is_43_6_0() {
        // NOTE: この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.6.0"), "Cargo.toml must contain version 43.6.0");
    }
    #[test]
    fn pipeline_two_step_bind_infers_types() {
        // List.map → bind → List.filter の2段パイプラインで型が連鎖伝播する
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn process(xs: List<Int>) -> List<Int> {
    bind doubled <- List.map(xs, |x| x * 2)
    List.filter(doubled, |x| x > 0)
}
"#;
        let prog = Parser::parse_str(src, "v43600_two_step.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "two-step pipeline should type-check: {:?}", result.err());
    }
    #[test]
    fn pipeline_three_step_bind_infers_types() {
        // 3段 bind チェーンで型が連鎖して伝播する
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn three_step(xs: List<Int>) -> List<Int> {
    bind step1 <- List.map(xs, |x| x + 1)
    bind step2 <- List.filter(step1, |x| x > 0)
    List.map(step2, |x| x * 2)
}
"#;
        let prog = Parser::parse_str(src, "v43600_three_step.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "three-step pipeline should type-check: {:?}", result.err());
    }
}
```

---

## T2 — Cargo.toml + v43500_tests スタブ化

```toml
version = "43.6.0"
```

`v43500_tests::cargo_toml_version_is_43_5_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_5_0() {
    // Stubbed: version bumped to 43.6.0 -- assertion intentionally removed
}
```

---

## T3 — CHANGELOG.md

```markdown
## [v43.6.0] — 2026-07-12

### Added
- `v43600_tests`: `cargo_toml_version_is_43_6_0` / `pipeline_two_step_bind_infers_types` / `pipeline_three_step_bind_infers_types`

### Changed
- `v43500_tests::cargo_toml_version_is_43_5_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: `infer_hm_let`（EBind 型伝播）+ v43.5.0 `infer_list_lambda_call`（ラムダ引数型推論）の組み合わせで多段パイプラインが機能
```

---

## T4 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2920 passed; 0 failed`

---

## T5 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.6.0 最新安定版（2920 tests）、次版 v43.7.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.6.0 を `✅ COMPLETE（2026-07-12）`、推定 2920 → 実績 2920 に修正
- `versions/v40-v45/v43.6.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
