# v32.9.0 — Plan: エフェクト推論 確認・テスト補強

## 実装方針

エフェクト推論（`infer_effects_fn` / `infer_effects_for_program` / `EffectSet`）は
v18.1.0 で完成済み。v32.9.0 は v32.1.0〜v32.8.0 と同じ「確認・記録」パターン。

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `"32.8.0"` → `"32.9.0"` |
| `fav/src/driver.rs` | `cargo_toml_version_is_32_8_0` スタブ化 + `v329000_tests` 追加 |
| `CHANGELOG.md` | `[v32.9.0]` セクションを先頭に追記 |
| `benchmarks/v32.9.0.json` | 新規作成（実測値で埋める） |
| `versions/current.md` | 最新安定版を v32.9.0 に更新 |
| `versions/v30-v35/v32.9.0/tasks.md` | COMPLETE に更新（全 [x]） |

---

## driver.rs 変更詳細

### ① `cargo_toml_version_is_32_8_0` をスタブ化

```rust
// v328000_tests 内（既存の #[test] fn を空スタブに置き換える）
fn cargo_toml_version_is_32_8_0() {
    // Stubbed: version bumped to 32.9.0 in v32.9.0.
}
```

### ② `v329000_tests` を挿入

挿入位置: `v328000_tests` の閉じ `}` 直後、`// ── v31.7.0 tests` コメントの前。
（`#[cfg(test)]` も含む v31.7.0 ブロック開始行より前）

```rust
// ── v32.9.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v329000_tests {
    use crate::ast::{Effect, Item};
    use crate::middle::checker::infer_effects_fn;
    use crate::frontend::parser::Parser;

    #[test]
    fn cargo_toml_version_is_32_9_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.9.0"), "Cargo.toml must contain '32.9.0'");
    }

    #[test]
    fn benchmark_v32_9_0_exists() {
        let src = include_str!("../../benchmarks/v32.9.0.json");
        assert!(src.contains("32.9.0"), "benchmarks/v32.9.0.json must contain '32.9.0'");
    }

    #[test]
    fn effect_infer_io_println() {
        // IO.println → !Io が推論される
        // (テスト名は v181000_tests::effect_inference_db と異なる / Postgres でなく Io を使用)
        let src = r#"
fn log_msg() -> String {
    bind _ <- IO.println("hello")
    "done"
}
"#;
        let prog = Parser::parse_str(src, "v329000_test.fav").expect("parse");
        let fn_def = prog.items.iter()
            .filter_map(|item| if let Item::FnDef(f) = item { Some(f) } else { None })
            .find(|f| f.name == "log_msg")
            .expect("fn log_msg not found");
        let (effects, _) = infer_effects_fn(fn_def);
        assert!(
            effects.contains(&Effect::Io),
            "IO.println should produce !Io effect, got: {:?}",
            effects
        );
    }

    #[test]
    fn effect_infer_pure_mul_no_effects() {
        // 算術のみの純粋関数 → エフェクト空集合
        // (テスト名は v181000_tests::effect_inference_pure と異なる / fn mul を使用)
        let src = r#"fn mul(a: Int, b: Int) -> Int { a * b }"#;
        let prog = Parser::parse_str(src, "v329000_test.fav").expect("parse");
        let fn_def = prog.items.iter()
            .filter_map(|item| if let Item::FnDef(f) = item { Some(f) } else { None })
            .find(|f| f.name == "mul")
            .expect("fn mul not found");
        let (effects, _) = infer_effects_fn(fn_def);
        assert!(
            effects.is_empty(),
            "pure fn mul should have no effects, got: {:?}",
            effects
        );
    }
}
```

---

### ③ `versions/v30-v35/v32.9.0/tasks.md` を COMPLETE に更新

全チェックボックスを `[x]` にし、ステータスを `COMPLETE` に変更する。

---

## テスト数の見通し

| ステップ | 増減 | 累計 |
|---|---|---|
| v32.8.0 完了時点 | — | 2488 |
| `cargo_toml_version_is_32_8_0` スタブ化 | 0（テストは残る） | 2488 |
| `v329000_tests` 追加（4 件） | +4 | **2492** |

---

## CHANGELOG 追記内容

```markdown
## [v32.9.0] — 2026-07-03

### Added
- `v329000_tests`: エフェクト推論（Effect Inference）動作確認テスト 4 件
  - `cargo_toml_version_is_32_9_0` — バージョン確認
  - `benchmark_v32_9_0_exists` — ベンチマークファイル存在確認
  - `effect_infer_io_println` — `IO.println` → `!Io` エフェクト推論確認
  - `effect_infer_pure_mul_no_effects` — 純粋関数 `mul` → エフェクトなし確認

### Notes
- `infer_effects_fn` / `infer_effects_for_program` / `EffectSet` は v18.1.0 実装済み
- v32.9.0 はその動作を Language Power フェーズの記録として明示的に確認する
```
