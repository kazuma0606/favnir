# v32.9.0 — Spec: エフェクト推論 確認・テスト補強

## 概要

v32.9.0 は **エフェクト推論（Effect Inference）** の確認・テスト補強バージョン。

ロードマップ v32.5〜v32.9 候補のうち「エフェクト推論の強化」は v18.1.0 で既に実装済みである。
v32.9.0 では新規実装は行わず、`infer_effects_fn` / `infer_effects_for_program` の動作を
`v329000_tests` で確認・記録するにとどまる（v32.1〜v32.8 と同じ「確認・記録」パターン）:

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `pub type EffectSet = HashSet<Effect>` | ✓ | v18.1.0 |
| `pub fn infer_effects_fn(fn_def: &FnDef) -> (EffectSet, Vec<String>)` | ✓ | v18.1.0 |
| `pub fn infer_effects_for_program(program: &Program) -> HashMap<String, EffectSet>` | ✓ | v18.1.0 |
| `v181000_tests` — 4 件のテスト（`#[ignore]` なし） | ✓ | v18.1.0 |

v32.9.0 では、エフェクト推論の動作を `v329000_tests` で明示的に確認し、
バージョンと CHANGELOG を更新する。

---

## エフェクト推論 仕様確認

### 動作仕様

| ケース | 関数呼び出し | 推論結果 |
|---|---|---|
| IO エフェクト | `IO.println(...)` | `!Io` |
| 純粋関数 | 算術演算のみ | エフェクトなし（空集合） |

### 利用 API

```rust
// エフェクト直接推論（1 関数）
let (effects, _) = infer_effects_fn(fn_def);
effects.contains(&Effect::Io)  // !Io が推論されているか

// 純粋性確認
effects.is_empty()  // エフェクトなし = 純粋関数
```

---

## 追加するテスト（v329000_tests — 4 件）

`v329000_tests` は v32.1.0〜v32.8.0 と同じパターン:
- `use super::*` **なし**
- モジュール内に独自のヘルパー `get_effects` を定義
- `use crate::ast::{Effect, Item}; use crate::middle::checker::infer_effects_fn; use crate::frontend::parser::Parser;`

テスト名は v181000_tests（`effect_inference_db` / `effect_inference_multi` / `effect_inference_pure` / `effect_inference_transitive`）と被らないよう `effect_infer_` プレフィックスを使用する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_32_9_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("32.9.0"), "Cargo.toml must contain '32.9.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v32_9_0_exists() {
    let src = include_str!("../../benchmarks/v32.9.0.json");
    assert!(src.contains("32.9.0"), "benchmarks/v32.9.0.json must contain '32.9.0'");
}
```

### テスト 3: IO.println → !Io エフェクト推論（ポジティブ）

> **注**: `IO.println` は W009（非推奨呼び出し）の対象だが、`infer_effects_fn` は lint・ambient check を
> 経由しない純粋な AST 走査のため W009/E0023 は発火しない。テストは PASS する。

```rust
fn effect_infer_io_println() {
    // IO.println → !Io が推論される（infer_effects_fn は lint を経由しないため W009 は発火しない）
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
```

### テスト 4: 純粋関数 → エフェクトなし（ポジティブ）

```rust
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
```

---

## テストモジュールの配置

`v329000_tests` は `v328000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"32.9.0"`
- `cargo_toml_version_is_32_8_0` が空スタブになっていること
- `effect_infer_io_println` テストが PASS（`!Io` 推論）
- `effect_infer_pure_mul_no_effects` テストが PASS（エフェクトなし）
- `cargo test --bin fav v329000` — 4/4 PASS
- `cargo test` — 全件 PASS（2492 件、0 failures）
- `CHANGELOG.md` に `[v32.9.0]` セクション
- `benchmarks/v32.9.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v32.9.0.json` の `milestone` フィールドが `"Language Power"` であること
- `versions/current.md` を v32.9.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
- site/ MDX 更新: 対象外（エフェクト推論は v18.1.0 で完成済み）
