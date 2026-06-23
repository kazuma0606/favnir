# v24.2.0 — 4-Stage Bootstrap 検証

Date: 2026-06-23

## 目標

v24.0.0 で vm.fav が完成した。v24.2.0 では、Rust VM と vm.fav（Favnir）の両実行系が同一バイトコードを生成することを、複数の fixture プログラムで段階的に自動検証する infrastructure を整える。

```
Stage 1: Rust VM        + compiler.fav（元）→ fixture.fav    → bytecode_A
Stage 2: Rust VM        + compiler.fav（元）→ compiler.fav   → compiler_artifact
Stage 3: Rust VM        + compiler_artifact → fixture.fav    → bytecode_B
Stage 4: vm.fav（Favnir）+ compiler_artifact → fixture.fav   → bytecode_C（将来）

検証:
  bytecode_A == bytecode_B  ← Stage 1/3 の一致（3-stage bootstrap 拡張）
  bytecode_B == bytecode_C  ← Stage 3/4 の一致（vm.fav の等価性証明）← 将来
```

**v24.2.0 スコープ**: Stage 1–3 検証 infrastructure + 5 fixture 作成。Stage 4 は vm.fav のユーザー定義関数ディスパッチ（Phase 6）完了後に追加予定。

---

## ロードマップとの対応

| ロードマップ | v24.2.0 での対応 |
|---|---|
| `tests/bootstrap/hello.fav` | 作成 ✓ |
| `tests/bootstrap/arithmetic.fav` | 作成 ✓ |
| `tests/bootstrap/pattern_match.fav` | 作成 ✓ |
| `tests/bootstrap/list_ops.fav` | 作成 ✓ |
| `tests/bootstrap/closures.fav` | 作成 ✓ |
| `self/compiler.fav → hello.fav`（Stage 4 経由） | Stage 4 待ち（v25.x 以降） |
| `bytecode_A == bytecode_B` 自動検証 | `#[ignore]` Stage 1–3 テストで実装 ✓ |
| `bytecode_B == bytecode_C` 自動検証 | Stage 4 完了後（v25.x 以降） |

---

## スコープ

### fixture ファイル（新規作成）

| ファイル | 検証内容 |
|---|---|
| `fav/tests/bootstrap/hello.fav` | 基本文字列出力（`"Hello, Favnir!"`） |
| `fav/tests/bootstrap/arithmetic.fav` | 整数演算・関数呼び出し |
| `fav/tests/bootstrap/pattern_match.fav` | カスタム型 + バリアントマッチ |
| `fav/tests/bootstrap/list_ops.fav` | 再帰 + リストパターンマッチ |
| `fav/tests/bootstrap/closures.fav` | 高階関数・クロージャ |

### Rust（driver.rs）

| 変更種別 | 対象 | 内容 |
|---|---|---|
| テストモジュール追加 | `driver.rs` | `v242000_tests`（7 件カウント済 + 2 件 `#[ignore]`） |
| バージョンテスト削除 | `driver.rs` | `v241000_tests::version_is_24_1_0` を削除 |

### ドキュメント

| 変更種別 | 対象 | 内容 |
|---|---|---|
| エントリ追加 | `CHANGELOG.md` | v24.2.0 エントリ |
| 新規作成 | `benchmarks/v24.2.0.json` | test_count: 1940 |
| 新規作成 | `site/content/docs/tools/bootstrap.mdx` | 4-stage bootstrap 説明ページ |

---

## fixture ファイル定義

### `fav/tests/bootstrap/hello.fav`

```favnir
public fn main() -> String {
    "Hello, Favnir!"
}
```

### `fav/tests/bootstrap/arithmetic.fav`

```favnir
fn add(a: Int, b: Int) -> Int { a + b }
fn mul(a: Int, b: Int) -> Int { a * b }

public fn main() -> String {
    bind sum <- add(3, 7)
    bind product <- mul(4, 5)
    f"sum={sum} product={product}"
}
```

### `fav/tests/bootstrap/pattern_match.fav`

```favnir
fn describe(x: Option<Int>) -> String {
    match x {
        none    => "nothing"
        some(n) => f"got {n}"
    }
}

public fn main() -> String {
    describe(some(42))
}
```

> **注**: `type Shape = Circle | Square | Triangle`（フィールドなしバリアント）と
> `[h | t]`（リストパターン）はパーサー非対応。Option マッチで代替。

### `fav/tests/bootstrap/list_ops.fav`

```favnir
fn sum(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
    a + b + c + d + e
}

public fn main() -> String {
    bind r <- sum(1, 2, 3, 4, 5)
    f"total={r}"
}
```

> **注**: `[h | t]` リストパターンがパーサー非対応のため多引数算術関数で代替。

### `fav/tests/bootstrap/closures.fav`

```favnir
fn apply(f: Int -> Int, x: Int) -> Int { f(x) }

public fn main() -> String {
    bind r <- apply(|x| x * x, 7)
    f"result={r}"
}
```

---

## テスト設計

### カウント済みテスト（7 件、高速）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_2_0` | Cargo.toml に `version = "24.2.0"` | — |
| `bootstrap_hello_compiles` | hello.fav を `build_artifact` でコンパイル | パニックなし |
| `bootstrap_arithmetic_compiles` | arithmetic.fav を `build_artifact` でコンパイル | パニックなし |
| `bootstrap_pattern_match_compiles` | pattern_match.fav を `build_artifact` でコンパイル | パニックなし |
| `bootstrap_list_ops_compiles` | list_ops.fav を `build_artifact` でコンパイル | パニックなし |
| `bootstrap_closures_compiles` | closures.fav を `build_artifact` でコンパイル | パニックなし |
| `changelog_has_v24_2_0` | CHANGELOG.md に `[v24.2.0]` | — |

> **注**: `list_ops.fav`（`[h | t]` パターン）と `closures.fav`（高階関数型シグネチャ）は
> parser 非対応のリスクがある。コンパイルテストが失敗した場合は fixture を简化する（plan.md リスク表参照）。

### `#[ignore]` テスト（2 件、低速）

低速テスト（Stage 2 = compiler.fav のセルフコンパイル、~5s）は `#[ignore]` でマーク。

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `bootstrap_stage1_stage3_hello_match` | Stage 1（bytecode_A）と Stage 3（bytecode_B）を hello.fav で比較 | bytecode_A == bytecode_B |
| `bootstrap_stage1_stage3_arithmetic_match` | Stage 1/3 を arithmetic.fav で比較 | bytecode_A == bytecode_B |

> 注: Stage 4 関連の `#[ignore]` テストは Phase 6（vm.fav ユーザー定義関数ディスパッチ）完了後に追加。

### `v241000_tests::version_is_24_1_0` 削除

バージョンテストの慣例に従い、Cargo.toml バージョン更新（T3-1）より前に削除する。

---

## Stage 1–3 テスト実装方針

既存ヘルパー（`run_compiler_artifact_on` / `build_stage2_compiler_artifact`）を活用する。
`run_compiler_artifact_on` のシグネチャ: `(artifact: Arc<FvcArtifact>, input_path: String) -> (bool, Vec<u8>, String, String)`

```rust
// Stage 1: Rust build_artifact(compiler.fav) → artifact_s1 → run on fixture → bytecode_A
let compiler_src = include_str!("../../self/compiler.fav");
let tokens = crate::frontend::lexer::Lexer::new(compiler_src, "compiler.fav")
    .tokenize().expect("compiler.fav tokenize");
let prog = crate::frontend::parser::Parser::new(tokens)
    .parse_program().expect("compiler.fav parse");
let artifact_s1 = std::sync::Arc::new(build_artifact(&prog));
// #[ignore] テストはファイルパスを実行時に渡すため concat!(env!(...)) 方式を使う
let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bootstrap/hello.fav");
let (ok1, bytecode_a, _, _) = run_compiler_artifact_on(artifact_s1, fixture_path.to_string());
assert!(ok1, "Stage 1 failed");

// Stage 2: Rust VM + compiler.fav(元) → compiler.fav → compiler_artifact（self-compiled）
let artifact_s2 = build_stage2_compiler_artifact();

// Stage 3: Rust VM + compiler_artifact → fixture → bytecode_B
let (ok3, bytecode_b, _, _) = run_compiler_artifact_on(artifact_s2, fixture_path.to_string());
assert!(ok3, "Stage 3 failed");

assert_eq!(bytecode_a, bytecode_b, "bytecode_A must equal bytecode_B");
```

---

## 完了条件

- [ ] `fav/tests/bootstrap/` に 5 fixture 作成済み
- [ ] `cargo test v242000 --bin fav` — 7/7 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1940 件合格）
- [ ] `CHANGELOG.md` に v24.2.0 エントリ
- [ ] `benchmarks/v24.2.0.json` 作成済み（test_count: 1940）
- [ ] `site/content/docs/tools/bootstrap.mdx` 作成済み
- [ ] Stage 4 保留が CHANGELOG / bootstrap.mdx / spec.md / plan.md / tasks.md に明記済み
