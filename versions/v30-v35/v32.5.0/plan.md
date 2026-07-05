# v32.5.0 — Plan: 線形型 確認・テスト補強

## 実装方針

線形型（`TokenKind::LinearArrow`・E0332・E0333）は v18.5.0 で完成済み。
v32.5.0 は v32.1.0〜v32.4.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.4.0"` → `"32.5.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_4_0` スタブ化 + `v325000_tests` 追加 |
| `CHANGELOG.md` | `[v32.5.0]` セクションを先頭に追記 |
| `benchmarks/v32.5.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.5.0 に更新 |
| `versions/v30-v35/v32.5.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_4_0` をスタブ化

```rust
// v324000_tests 内
fn cargo_toml_version_is_32_4_0() {
    // Stubbed: version bumped to 32.5.0 in v32.5.0.
}
```

### ② `v325000_tests` を挿入

挿入位置: `v324000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` の前。

```rust
// ── v32.5.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v325000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v325000_test.fav").expect("parse");
        // Error::code は String 型 — .to_string() で Vec<String> に変換
        // v321000/v322000/v323000 と同パターン
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.to_string())
            .collect()
    }

    #[test]
    fn cargo_toml_version_is_32_5_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.5.0"), "Cargo.toml must contain '32.5.0'");
    }

    #[test]
    fn benchmark_v32_5_0_exists() {
        let src = include_str!("../../benchmarks/v32.5.0.json");
        assert!(src.contains("32.5.0"), "benchmarks/v32.5.0.json must contain '32.5.0'");
    }

    #[test]
    fn linear_type_double_use_e0332() {
        // Connection を 2 回 consume → E0332
        // テスト名は v185000_tests::linear_double_use_is_e0332 と重複しないよう
        // `linear_type_` プレフィックスを使用
        let errors = check_errors(r#"
fn open_conn() -> Connection {
    Connection
}
fn consume(c: Connection) -> String { "ok" }
fn use_twice() -> String {
    bind c <- open_conn()
    bind _a <- consume(c)
    bind _b <- consume(c)
    "done"
}
"#);
        assert!(
            errors.iter().any(|e| e == "E0332"),
            "Expected E0332 for double use of linear variable, got: {:?}",
            errors
        );
    }

    #[test]
    fn linear_type_unused_var_e0333() {
        // Connection を bind して使わない → E0333
        // テスト名は v185000_tests::linear_unused_is_e0333 と重複しないよう
        // `linear_type_` プレフィックスを使用
        let errors = check_errors(r#"
fn open_conn() -> Connection {
    Connection
}
fn forget_conn() -> String {
    bind _c <- open_conn()
    "done"
}
"#);
        assert!(
            errors.iter().any(|e| e == "E0333"),
            "Expected E0333 for unused linear variable, got: {:?}",
            errors
        );
    }
}
```

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.4.0 完了時点 | — | 2472 |
| `cargo_toml_version_is_32_4_0` スタブ化 | 0（テストは残る） | 2472 |
| `v325000_tests` 追加（4 件） | +4 | **2476** |

---

## CHANGELOG 追記内容

```markdown
## [v32.5.0] — 2026-07-03

### Added
- `v325000_tests`: 線形型（Linear Types）動作確認テスト 4 件
  - `cargo_toml_version_is_32_5_0` — バージョン確認
  - `benchmark_v32_5_0_exists` — ベンチマークファイル存在確認
  - `linear_type_double_use_e0332` — Connection 二重使用で E0332
  - `linear_type_unused_var_e0333` — Connection 未使用で E0333

### Notes
- `TokenKind::LinearArrow`・E0332・E0333 は v18.5.0 実装済み
- v32.5.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
