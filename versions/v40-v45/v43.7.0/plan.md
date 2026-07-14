# v43.7.0 実装計画 — 構造体リテラル推論（Structural inference）

## 前提

- v43.6.0 完了（2920 tests）
- `fav/Cargo.toml` version: `43.6.0`
- `checker.fav` に変更不要: 既存の `ERecordLit` → `tname` 返却で名前付きレコードリテラルが機能する
- `bind _ <-` は短絡しない → `Result.and_then` / `match { Ok(...) => ... Err(e) => ... }` を使うこと（v43.4.0/v43.5.0 で判明）

---

## タスク順序

```
T0 事前確認
T1 driver.rs — v43700_tests 追加（v43600_tests の直前）
T2 Cargo.toml — version 43.6.0 → 43.7.0 + v43600_tests スタブ化
T3 CHANGELOG.md — v43.7.0 エントリ追加
T4 cargo test 実行・確認（2922 pass, 0 fail）
T5 バージョン管理ドキュメント更新
```

---

## T0 — 事前確認

1. `cargo test` 2920 / 0 確認
2. `Cargo.toml` version = `43.6.0` 確認
3. `v43700_tests` が driver.rs に存在しないことを確認
4. `checker.fav` line 1957 に `ERecordLit({ _0: tname, _1: fields }) => Result.ok(tname)` が存在することを確認

---

## T1/T2 アトミシティ注記

T1（driver.rs 追加）と T2（Cargo.toml bump + v43600_tests スタブ化）は**同一コミット**で適用する。
`cargo_toml_version_is_43_7_0` テストは Cargo.toml が 43.7.0 であることを前提とするため、
T1 のみ適用して `cargo test` を実行すると当該テストが失敗する。

---

## T1 — driver.rs — v43700_tests

`v43600_tests` モジュールの直前に挿入:

```rust
// -- v43700_tests (v43.7.0) -- 構造体リテラル推論（Structural inference）--
#[cfg(test)]
mod v43700_tests {
    #[test]
    fn cargo_toml_version_is_43_7_0() {
        // この assert は次バージョン bump 時にスタブ化すること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("43.7.0"), "Cargo.toml must contain version 43.7.0");
    }
    #[test]
    fn structural_record_literal_type_checks() {
        // 名前付きレコードリテラルが関数の戻り値型・引数型と一致する
        use crate::frontend::parser::Parser;
        use crate::middle::ast_lower_checker::lower_program;
        use crate::checker_fav_runner::run_checker_fav;
        let src = r#"
type Point = { x: Int  y: Int }
fn make_point() -> Point { Point { x: 1  y: 2 } }
fn shift(p: Point) -> Point { Point { x: p.x  y: p.y } }
"#;
        let prog = Parser::parse_str(src, "v43700_record.fav").expect("parse");
        let result = run_checker_fav(lower_program(&prog));
        assert!(result.is_ok(), "record literal type check should pass: {:?}", result.err());
    }
}
```

---

## T2 — Cargo.toml + v43600_tests スタブ化

```toml
version = "43.7.0"
```

`v43600_tests::cargo_toml_version_is_43_6_0` をスタブ化:

```rust
fn cargo_toml_version_is_43_6_0() {
    // Stubbed: version bumped to 43.7.0 -- assertion intentionally removed
}
```

---

## T3 — CHANGELOG.md

```markdown
## [v43.7.0] — 2026-07-12

### Added
- `v43700_tests`: `cargo_toml_version_is_43_7_0` / `structural_record_literal_type_checks`

### Changed
- `v43600_tests::cargo_toml_version_is_43_6_0` をスタブ化

### Notes
- `fav/self/checker.fav` は変更なし: 名前付きレコードリテラル（`TypeName { ... }`）は既存の `ERecordLit → tname` 機構で型チェックを通過する
```

---

## T4 — テスト実行

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待: `2922 passed; 0 failed`

---

## T5 — バージョン管理ドキュメント更新

- `versions/current.md` → v43.7.0 最新安定版（2922 tests）、次版 v43.8.0
- `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.7.0 を `✅ COMPLETE（2026-07-12）`、推定 2922 → 実績 2922 に修正
- `versions/v40-v45/v43.7.0/tasks.md` → COMPLETE、全チェックボックス `[x]`
