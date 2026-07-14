# v43.8.0 実装計画 — 双方向型推論（Bidirectional / top-down）

## 前提

- v43.7.0 完了（2922 tests）
- `fav/Cargo.toml` version: `43.7.0`
- `checker.fav` に変更不要: `infer_list_lambda_call`（v43.5.0）がラムダ引数型の下向き伝播を実現済み
- `bind _ <-` は短絡しない → `Result.and_then` / `match { Ok(...) => ... Err(e) => ... }` を使うこと（v43.4.0/v43.5.0 で判明）

---

## タスク順序

```
T0 事前確認
T1 driver.rs — v43800_tests 追加（v43700_tests の直前）
T2 Cargo.toml — version 43.7.0 → 43.8.0 + v43700_tests スタブ化
T3 CHANGELOG.md — v43.8.0 エントリ追加
T4 cargo test 実行・確認（2925 pass, 0 fail）
T5 バージョン管理ドキュメント更新
```

---

## T1/T2 アトミシティ注記

T1（driver.rs 追加）と T2（Cargo.toml bump + v43700_tests スタブ化）は**同一コミット**で適用する。
`cargo_toml_version_is_43_8_0` テストは Cargo.toml が `43.8.0` であることを前提とするため、
T1 のみ適用して `cargo test` を実行すると当該テストが失敗する。

---

## T0 — 事前確認

1. `cargo test` 2922 / 0 確認
2. `Cargo.toml` version = `43.7.0` 確認
3. `v43800_tests` が driver.rs に存在しないことを確認
4. `checker.fav` に `fn infer_list_lambda_call` が存在することを確認（現在 line 1849 付近）

---

## T1 — driver.rs — v43800_tests

`v43700_tests` モジュールの直前に挿入:

```rust
// -- v43800_tests (v43.8.0) -- 双方向型推論（Bidirectional / top-down）--
#[cfg(test)]
mod v43800_tests {
    #[test]
    fn cargo_toml_version_is_43_8_0() {
        // この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.8.0"), "Cargo.toml must contain version 43.8.0");
    }
    #[test]
    fn bidirectional_filter_infers_elem_type() {
        // ロードマップ記載例: xs の要素型 Int がラムダ引数 x に下向き伝播する
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn filter_positive(xs: List<Int>) -> List<Int> {
    List.filter(xs, |x| x > 0)
}
"#;
        let prog = Parser::parse_str(src, "v43800_filter.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "bidirectional filter should type-check: {:?}", result.err());
    }
    #[test]
    fn bidirectional_nested_map_filter_expression() {
        // ネスト呼び出し式: List.map の結果型 List<Int> が List.filter のラムダ引数 y に伝播する
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
fn transform(xs: List<Int>) -> List<Int> {
    List.filter(List.map(xs, |x| x + 1), |y| y > 0)
}
"#;
        let prog = Parser::parse_str(src, "v43800_nested.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "nested map+filter should type-check: {:?}", result.err());
    }
}
```

---

## T2 — Cargo.toml + v43700_tests スタブ化

```toml
version = "43.8.0"
```

`v43700_tests::cargo_toml_version_is_43_7_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_7_0() {
    // Stubbed: version bumped to 43.8.0 in v43.8.0.
}
```

---

## T3 — CHANGELOG.md

日付は実装当日のものに変更すること。

```markdown
## [v43.8.0] — 2026-07-13

### Added
- `v43800_tests`: `cargo_toml_version_is_43_8_0` / `bidirectional_filter_infers_elem_type` / `bidirectional_nested_map_filter_expression`

### Changed
- `v43700_tests::cargo_toml_version_is_43_7_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: v43.5.0 の `infer_list_lambda_call` がリスト要素型の下向き伝播（双方向型推論）を実現済み
```

---

## T4 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2925 passed; 0 failed`

---

## T5 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.8.0 最新安定版（2925 tests）、次版 v43.9.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.8.0 を `✅ COMPLETE（2026-07-13）`、推定 2925 → 実績 2925 に修正
- `versions/v40-v45/v43.8.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
