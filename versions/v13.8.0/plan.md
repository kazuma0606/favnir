# v13.8.0 Plan — ambient effect 禁止（W008 → E0023）実装計画

Date: 2026-06-11

---

## 実装アプローチの概要

`lint.rs` の W008 実装を基盤として、`check_ambient_errors()` (E0023) を追加。
`driver.rs` の `cmd_check` に非 legacy モード専用ブロックとして統合。
`compiler.fav` の ambient IO 呼び出しを削除 or `!IO` アノテーションで保護。
`compiler_fav_runner.rs` でファイル読み込みを Rust 側に移動。

**変更ファイル一覧**:

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0023 エントリ追加（E0022 と E0213 の間） |
| `fav/src/lint.rs` | `check_ambient_errors()` 追加、`collect_ambient(code)` リファクタリング |
| `fav/src/driver.rs` | E0023 ヘルプテキスト追加、`cmd_check` に E0023 ブロック統合、`v138000_tests` 追加 |
| `fav/self/compiler.fav` | デバッグ IO 関数削除、`compile_bytes(path)` 削除、`!IO` アノテーション追加 |
| `fav/src/compiler_fav_runner.rs` | `compile_file_to_bytes` をファイル読み込み Rust 化 |
| `fav/Cargo.toml` | `version = "13.8.0"` |

---

## Phase A — E0023 エラーカタログ追加

### A-1: `fav/src/error_catalog.rs`

E0022 エントリの直後に追加:

```rust
ErrorEntry {
    code: "E0023",
    title: "ambient effect call is not allowed",
    category: "effects",
    description: "A function calls a side-effecting namespace (IO, Postgres, AWS, etc.) \
                  without threading a capability context (ctx) argument. \
                  In non-legacy mode, all ambient calls are rejected unless the function \
                  is annotated with !IO.",
    example: "fn run() -> Unit {\n    bind _ <- IO.println(\"hi\")  // E0023: ambient IO call\n    ()\n}",
    fix: "Pass an io capability through the function signature, \
          or use `ctx.io.println(...)` instead of `IO.println(...)`. \
          Use `--legacy` to allow ambient calls during migration.",
},
```

### A-2: `fav/src/driver.rs` の `get_help_text`

```rust
"E0023" => &[
    "pass an io/db/storage capability through the function signature",
    "use `ctx.io.println(...)` instead of `IO.println(...)`",
    "use `--legacy` flag to allow ambient calls during migration",
],
```

---

## Phase B — lint.rs: W008 → E0023 共通化

### B-1: 既存の W008 実装を読んで構造を把握

`check_ambient_effects(program)` → `collect_ambient_in_block` / `collect_ambient_in_expr` の
再帰構造を確認する。

### B-2: `collect_ambient(program, code: &'static str)` 共通ヘルパーを実装

既存の `check_ambient_effects` の内部ロジックを `code` 引数付きの汎用関数に切り出す。

```rust
fn collect_ambient(program: &Program, code: &'static str) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) if code == "E0023" && has_io_effect(&fd.effects) => {
                // !IO アノテーション付き関数は E0023 免除
            }
            Item::FnDef(fd) => {
                collect_ambient_in_block(&fd.body, &mut errors, code);
            }
            _ => {}
        }
    }
    errors
}

fn has_io_effect(effects: &[Effect]) -> bool {
    effects.iter().any(|e| matches!(e, Effect::IO))
}
```

`collect_ambient_in_block` と `collect_ambient_in_expr` も `code: &'static str` 引数を受け取るよう更新。
（`LintError.code` は `&'static str` のため、string literal のみ格納可能。）

### B-3: W008 ラッパーと E0023 ラッパーを実装

```rust
// 既存（変更なし、ただし内部で collect_ambient を使う）
pub fn check_ambient_effects(program: &Program) -> Vec<LintError> {
    collect_ambient(program, "W008")
}

// 新規
pub fn check_ambient_errors(program: &Program) -> Vec<LintError> {
    collect_ambient(program, "E0023")
}
```

**注意**: `collect_ambient_in_expr` の全分岐に `code` を引き回すこと。
`Expr::Block(b)` 分岐など見落としやすい箇所あり。

---

## Phase C — driver.rs: cmd_check への統合

### C-1: E0023 チェックブロックの追加箇所

`cmd_check` の `if strict { ... }` ブロックの直後に追加:

```rust
// E0023: ambient effect check（非 legacy モード専用）
if !legacy_check && !json {
    let program = Parser::parse_str(&source, path).ok();
    if let Some(prog) = program {
        let e0023s = crate::lint::check_ambient_errors(&prog);
        if !e0023s.is_empty() {
            for e in &e0023s {
                // print with source context (同様の形式)
                eprintln!("error[E0023]: {}", e.message);
                eprintln!("  --> {}:{}:{}", path, e.line, e.col);
                // ... source line + underline ...
            }
            eprintln!(
                "error: {} ambient effect call(s) rejected (E0023)",
                e0023s.len()
            );
            eprintln!("  = note: use `--legacy` to allow ambient calls during migration");
            process::exit(1);
        }
    }
}
```

### C-2: 条件確認

| 条件 | E0023 実行 |
|---|---|
| 通常 `fav check` | YES |
| `fav check --legacy` | NO |
| `fav check --json` | NO（フォーマット統一後回し） |
| `fav check --ambient` | NO（W008 は別パスで表示） |

---

## Phase D — compiler.fav IO 移行

### D-1: 削除対象関数

以下の関数を削除（デバッグ用 IO のみ含む）:
- `compile_file_after_prog` — IO.println デバッグ出力
- `compile_file_after_parse` — IO.println デバッグ出力
- `compile_file_after_lex` — IO.println デバッグ出力
- `compile_file(path)` — `compile_bytes(path)` に委譲するだけ（削除可）
- `compile_bytes(path)` — Rust 側でファイル読み込みに移行

### D-2: 保持する関数（`!IO` アノテーション付き）

```fav
fn compile_file_quiet(path: String) -> Result<Artifact, String> !IO {
    Result.and_then(IO.read_file_raw(path), |src| compile_bytes_from_src(src))
}

fn print_bytes(bytes: List<Int>) -> Bool !IO {
    // 既存実装を維持
}

public fn main() -> Bool !IO {
    // bootstrap entry point を維持
}
```

`!IO` アノテーションにより、E0023 チェックの対象外になる（明示的オプトイン）。

### D-3: パブリック API

`compile_bytes_from_src(src: String) -> Result<List<Int>, String>` のみメインパブリック API。
IO 呼び出しなし → E0023 ゼロ。

---

## Phase E — compiler_fav_runner.rs 更新

### E-1: `compile_file_to_bytes` の変更

**変更前**（VM 経由で compiler.fav の `compile_bytes(path)` を呼ぶ）:
```rust
pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String> {
    let fn_idx = artifact.fn_idx_by_name("compile_bytes")?;
    VM::run(&artifact, fn_idx, vec![Value::Str(path.to_string())])
    // ...
}
```

**変更後**（Rust でファイル読み込み → `compile_bytes_from_src` に委譲）:
```rust
pub fn compile_file_to_bytes(path: &str) -> Result<Vec<u8>, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read `{}`: {}", path, e))?;
    compile_src_str_to_bytes(&src)
}
```

`compile_src_str_to_bytes` が `compile_bytes_from_src` を呼ぶ既存のパスを利用。

---

## Phase F — テスト追加

### F-1: `v138000_tests` モジュール（driver.rs 末尾）

```rust
#[cfg(test)]
mod v138000_tests {
    use super::*;

    #[test]
    fn version_is_13_8_0() {
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(version, "13.8.0");
    }

    #[test]
    fn e0023_ambient_io_println() {
        // fn run() -> Unit で IO.println を呼ぶ → E0023
    }

    #[test]
    fn e0023_ambient_postgres_raw() {
        // fn fetch(sql: String) -> ... で Postgres.query_raw を呼ぶ → E0023
    }

    #[test]
    fn legacy_mode_allows_ambient() {
        // check_ambient_errors → E0023 あり
        // check_ambient_effects（W008）→ W008 あり（--ambient フラグ相当）
        // legacy モード（check_ambient_errors 呼ばない）→ E0023 なし
    }

    #[test]
    fn ctx_based_compiler_fav_compiles() {
        // compiler.fav 全体に E0023 がゼロであることを確認
        // check_ambient_errors(&prog).is_empty() == true
    }

    #[test]
    fn pure_fn_no_e0023() {
        // IO 呼び出しのない純粋関数 → E0023 なし
    }
}
```

### F-2: 実行確認

```bash
cargo test v138000  # 6/6 パス確認
cargo test          # 全件パス（リグレッション確認）
```

---

## Phase G — バージョンバンプ + コミット

```bash
# Cargo.toml: version = "13.7.0" → "13.8.0"
cargo test -- --test-threads=1  # 全件パス確認
git add -A
git commit -m "feat: v13.8.0 — ambient effect 禁止 (W008 → E0023)"
```

---

## 実装上の注意点・リスク

### R-1: `LintError.code` は `&'static str`

`LintError` 構造体の `code` フィールドは `&'static str`。
`"W008".to_string()` のような動的文字列は不可。
`collect_ambient` の `code: &'static str` パラメータで直接 `LintError { code, ... }` に格納。

### R-2: bootstrap テストの依存

`bootstrap_*` / `bootstrap_d2_*` テストは `exec_artifact_main` 経由で compiler.fav の `main()` を呼ぶ。
`main()` を削除するとこれらのテストが全件失敗する。
`main()` と `compile_file_quiet` / `print_bytes` は `!IO` アノテーション付きで必ず保持すること。

### R-3: `collect_ambient_in_expr` の分岐網羅

`Expr::Block(b)` など見落としやすい分岐が E0023 検出を漏らす可能性がある。
W008 テスト（`w008_ambient_*`）が全件パスすることで間接的に確認できるが、
E0023 テストも明示的に複数パターン追加すること。

### R-4: JSON モードとの非干渉

`fav check --json` では E0023 を実行しない（Phase C-2 参照）。
JSON 出力フォーマットに E0023 を統合するのは v13.x 以降の課題。
条件分岐の順序に注意: `if !legacy_check && !json { ... }` の `!json` を忘れない。

### R-5: `--ambient` フラグとの共存

`fav check --ambient` は W008 パス（`check_ambient_effects`）を通る既存の動作を維持。
E0023 パス（`check_ambient_errors`）は `--ambient` フラグに関係なく、`--legacy` のみで抑制。
2 つのコードパスが独立していることを実装で確認すること。

---

## 実装順序（推奨）

```
A（error_catalog + help text）
→ B（lint.rs リファクタリング）→ W008 テスト全件パス確認
→ C（driver.rs 統合）→ e0023_ambient_io_println テスト追加して確認
→ D（compiler.fav 移行）→ cargo build で !IO アノテーションのコンパイル確認
→ E（compiler_fav_runner.rs）→ bootstrap テスト全件パス確認
→ F（テスト全件追加）→ cargo test v138000
→ G（バージョンバンプ + cargo test 全件）
```
