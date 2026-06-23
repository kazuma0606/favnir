# v24.2.0 実装計画 — 4-Stage Bootstrap 検証

## 前提確認

v24.2.0 は fixture 5 件 + driver.rs テストモジュール追加 + ドキュメント変更。
Rust ソース（driver.rs）はテストモジュールのみ追加。本体ロジックの変更なし。

### 実装前チェック

```bash
grep -n "version = " fav/Cargo.toml
# → "24.1.0" であること

grep -n "mod v241000_tests\|mod v242000_tests" fav/src/driver.rs | head -5
# → v242000_tests が未存在であること

ls fav/tests/bootstrap/ 2>/dev/null || echo "not found"
# → ディレクトリが未存在であること（または空）

grep -n "\[v24.2.0\]" CHANGELOG.md | head -3
# → 0 件であること
```

---

## T0: fixture ディレクトリ・ファイル作成

`fav/tests/bootstrap/` を作成し、5 fixture を追加する。

### T0-1: `fav/tests/bootstrap/hello.fav`

```favnir
public fn main() -> String {
    "Hello, Favnir!"
}
```

### T0-2: `fav/tests/bootstrap/arithmetic.fav`

```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn mul(a: Int, b: Int) -> Int { a * b }

public fn main() -> String {
    bind sum <- add(3, 7)
    bind product <- mul(4, 5)
    f"sum={sum} product={product}"
}
```

### T0-3: `fav/tests/bootstrap/pattern_match.fav`

```favnir
type Shape = Circle | Square | Triangle

fn shape_name(s: Shape) -> String {
    match s {
        Circle   => "circle"
        Square   => "square"
        Triangle => "triangle"
    }
}

public fn main() -> String {
    shape_name(Circle)
}
```

### T0-4: `fav/tests/bootstrap/list_ops.fav`

```favnir
fn list_sum(xs: List<Int>) -> Int {
    match xs {
        []      => 0
        [h | t] => h + list_sum(t)
    }
}

public fn main() -> String {
    bind r <- list_sum([1, 2, 3, 4, 5])
    f"total={r}"
}
```

### T0-5: `fav/tests/bootstrap/closures.fav`

```favnir
fn apply(f: Int -> Int, x: Int) -> Int { f(x) }

public fn main() -> String {
    bind r <- apply(|x| x * x, 7)
    f"result={r}"
}
```

- [ ] **事後確認**: 5 ファイルが存在すること

---

## T1: `fav/src/driver.rs` — v242000_tests 追加

### T1-1: `v241000_tests::version_is_24_1_0` を削除（T3-1 より前に必須）

```rust
    #[test]
    fn version_is_24_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"24.1.0\""), "Cargo.toml should have version 24.1.0");
    }
```

この関数ごと削除する。

### T1-2: `v242000_tests` モジュールを `v241000_tests` の直後に追加

挿入位置: `v241000_tests` モジュールの閉じ括弧 `}` の直後。

```rust
// ── v242000_tests (v24.2.0) — 4-Stage Bootstrap 検証 ────────────────────
#[cfg(test)]
mod v242000_tests {
    use super::*;

    #[test]
    fn version_is_24_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.2.0\""),
            "Cargo.toml should have version 24.2.0"
        );
    }

    #[test]
    fn bootstrap_hello_compiles() {
        let src = include_str!("../tests/bootstrap/hello.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "hello.fav")
            .tokenize()
            .expect("hello.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("hello.fav parse failed");
        let _art = build_artifact(&prog);
    }

    #[test]
    fn bootstrap_arithmetic_compiles() {
        let src = include_str!("../tests/bootstrap/arithmetic.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "arithmetic.fav")
            .tokenize()
            .expect("arithmetic.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("arithmetic.fav parse failed");
        let _art = build_artifact(&prog);
    }

    #[test]
    fn bootstrap_pattern_match_compiles() {
        let src = include_str!("../tests/bootstrap/pattern_match.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "pattern_match.fav")
            .tokenize()
            .expect("pattern_match.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("pattern_match.fav parse failed");
        let _art = build_artifact(&prog);
    }

    #[test]
    fn bootstrap_list_ops_compiles() {
        let src = include_str!("../tests/bootstrap/list_ops.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "list_ops.fav")
            .tokenize()
            .expect("list_ops.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("list_ops.fav parse failed");
        let _art = build_artifact(&prog);
    }

    #[test]
    fn bootstrap_closures_compiles() {
        let src = include_str!("../tests/bootstrap/closures.fav");
        let tokens = crate::frontend::lexer::Lexer::new(src, "closures.fav")
            .tokenize()
            .expect("closures.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("closures.fav parse failed");
        let _art = build_artifact(&prog);
    }

    #[test]
    fn changelog_has_v24_2_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("[v24.2.0]"),
            "CHANGELOG.md should have [v24.2.0] entry"
        );
    }

    // ── Stage 1–3 bootstrap 検証（低速 — CI の full pass でのみ実行） ──────
    // Stage 2 = compiler.fav のセルフコンパイル（~5s）のため #[ignore]
    // 注: カウント済みテスト（include_str!）と異なり、#[ignore] テストは
    //     run_compiler_artifact_on にファイルパスを渡すため concat!(env!(...)) 方式を使う

    #[test]
    #[ignore]
    fn bootstrap_stage1_stage3_hello_match() {
        // Stage 1: Rust build_artifact(compiler.fav) → artifact → run on hello.fav → bytecode_A
        let compiler_src = include_str!("../../self/compiler.fav");
        let tokens = crate::frontend::lexer::Lexer::new(compiler_src, "compiler.fav")
            .tokenize()
            .expect("compiler.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("compiler.fav parse failed");
        let artifact_s1 = std::sync::Arc::new(build_artifact(&prog));
        let hello_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bootstrap/hello.fav");
        let (ok1, bytecode_a, _, _) =
            run_compiler_artifact_on(artifact_s1, hello_path.to_string());
        assert!(ok1, "Stage 1 failed for hello.fav");
        // Stage 2: Rust VM + compiler.fav(元) → compiler.fav → compiler_artifact（self-compiled）
        let artifact_s2 = build_stage2_compiler_artifact();
        // Stage 3: Rust VM + compiler_artifact → hello.fav → bytecode_B
        let (ok3, bytecode_b, _, _) =
            run_compiler_artifact_on(artifact_s2, hello_path.to_string());
        assert!(ok3, "Stage 3 failed for hello.fav");
        assert_eq!(
            bytecode_a, bytecode_b,
            "Stage 1 and Stage 3 bytecode must match for hello.fav"
        );
    }

    #[test]
    #[ignore]
    fn bootstrap_stage1_stage3_arithmetic_match() {
        let compiler_src = include_str!("../../self/compiler.fav");
        let tokens = crate::frontend::lexer::Lexer::new(compiler_src, "compiler.fav")
            .tokenize()
            .expect("compiler.fav tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("compiler.fav parse failed");
        let artifact_s1 = std::sync::Arc::new(build_artifact(&prog));
        let arith_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bootstrap/arithmetic.fav");
        let (ok1, bytecode_a, _, _) =
            run_compiler_artifact_on(artifact_s1, arith_path.to_string());
        assert!(ok1, "Stage 1 failed for arithmetic.fav");
        let artifact_s2 = build_stage2_compiler_artifact();
        let (ok3, bytecode_b, _, _) =
            run_compiler_artifact_on(artifact_s2, arith_path.to_string());
        assert!(ok3, "Stage 3 failed for arithmetic.fav");
        assert_eq!(
            bytecode_a, bytecode_b,
            "Stage 1 and Stage 3 bytecode must match for arithmetic.fav"
        );
    }
}
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0
- [ ] `cargo test v242000 --bin fav` — 7/7 PASS

---

## T2: Cargo.toml + CHANGELOG + benchmarks + bootstrap.mdx

> **注意**: T1-1 の `version_is_24_1_0` 削除完了後に Cargo.toml を更新すること（T3-1）。

### T3-1: `fav/Cargo.toml` バージョン更新

```
version = "24.1.0" → "24.2.0"
```

### T3-2: `CHANGELOG.md` 先頭に v24.2.0 エントリ追加

```markdown
## [v24.2.0] — 2026-06-23 — 4-Stage Bootstrap 検証

### Added
- `fav/tests/bootstrap/` — Bootstrap 検証用 fixture 5 件（hello / arithmetic / pattern_match / list_ops / closures）
- `v242000_tests` — Bootstrap fixture コンパイルテスト 7 件（カウント済）
- `bootstrap_stage1_stage3_hello_match` / `bootstrap_stage1_stage3_arithmetic_match` — Stage 1/3 bytecode 比較（`#[ignore]`、低速）

### Notes
- Stage 4（vm.fav + compiler_artifact → bytecode_C）は vm.fav Phase 6（ユーザー定義関数ディスパッチ）完了後に追加予定
- `bytecode_A == bytecode_B` 検証は `cargo test bootstrap_stage1 -- --ignored` で実行
```

### T3-3: `benchmarks/v24.2.0.json` 作成

```json
{
  "version": "24.2.0",
  "date": "2026-06-23",
  "test_count": 1940,
  "feature": "4-Stage Bootstrap 検証",
  "metrics": {
    "fixture_count": 5,
    "stage4_deferred": true,
    "new_ignored_tests": 2
  }
}
```

### T3-4: `site/content/docs/tools/bootstrap.mdx` 作成

~~~mdx
---
title: 4-Stage Bootstrap 検証
description: Rust VM と vm.fav（Favnir）の等価性を自動検証する仕組み
---

# 4-Stage Bootstrap 検証

Favnir コンパイラは以下の 4 段階で自己検証します。

## ステージ定義

| ステージ | 実行系 | 入力 | 出力 |
|---|---|---|---|
| Stage 1 | Rust VM + compiler.fav（元） | fixture.fav | bytecode_A |
| Stage 2 | Rust VM + compiler.fav（元） | compiler.fav | compiler_artifact |
| Stage 3 | Rust VM + compiler_artifact | fixture.fav | bytecode_B |
| Stage 4 | vm.fav（Favnir）+ compiler_artifact | fixture.fav | bytecode_C |

## 検証式

```
bytecode_A == bytecode_B  （Stage 1/3 一致 — 3-stage bootstrap）
bytecode_B == bytecode_C  （Stage 3/4 一致 — vm.fav 等価性証明）
```

## fixture 一覧

- `tests/bootstrap/hello.fav` — 基本文字列出力
- `tests/bootstrap/arithmetic.fav` — 整数演算
- `tests/bootstrap/pattern_match.fav` — カスタム型 + バリアントマッチ
- `tests/bootstrap/list_ops.fav` — 再帰 + リストパターンマッチ
- `tests/bootstrap/closures.fav` — 高階関数・クロージャ

## テスト実行

```bash
# 高速テスト（CI 常時実行）
cargo test v242000 --bin fav

# Stage 1–3 bootstrap 検証（低速・任意実行）
cargo test bootstrap_stage1 --bin fav -- --ignored
```

## Stage 4 について

Stage 4（vm.fav + compiler_artifact → bytecode_C）は vm.fav Phase 6
（ユーザー定義関数ディスパッチ）完了後に有効化される予定です（v25.x 以降）。
~~~

---

## 実装順序

```
T0（fixture 5 件作成）
T1-1（version_is_24_1_0 削除）← T3-1 より前に必須
T1-2（v242000_tests 追加）
cargo check → エラー 0 確認
cargo test v242000 → 7/7 PASS 確認
T3-1（version 更新）← T1-1 完了後
T3-2〜4（CHANGELOG / benchmarks / bootstrap.mdx）
cargo test --bin fav → リグレッションなし確認（1940 件）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `[h \| t]` リストパターンが parser 非対応 | `bootstrap_list_ops_compiles` が失敗 | list_ops.fav を `List.fold` 等の builtin で書き直す |
| `fn apply(f: Int -> Int, x: Int)` 高階関数型シグネチャが非対応 | `bootstrap_closures_compiles` が失敗 | closures.fav をシンプルなクロージャのみに変更 |
| `run_compiler_artifact_on` シグネチャ変更 | `#[ignore]` テストの cargo check エラー | driver.rs の実際のシグネチャを `grep -n "fn run_compiler_artifact_on"` で確認してから実装 |
| `include_str!` パスが fixture 移動後に壊れる | cargo check エラー | パスは `../tests/bootstrap/X.fav`（driver.rs からの相対パス）に固定する |
| Stage 1–3 テストが低速すぎて CI タイムアウト | `#[ignore]` なのでカウント済みテストには影響しない | `--ignored` 実行は任意とする |
