# v63.6.0 Spec — バックプレッシャー制御（W041 lint + `[backpressure]` 設定）

Version: 63.6.0
Status: 未着手
Base tests: 3416
Target tests: 3418

---

## 概要

`lint.rs` に W041 `perf_hint_large_collect` を追加（`--strict` / `--perf` モード下でのみ有効）。
`toml.rs` に `BackpressureConfig { strategy: String, max_queue_depth: usize, warn_threshold: usize }` を追加。

```toml
[backpressure]
strategy = "drop"      # drop | block | sample
max_queue_depth = 500
warn_threshold = 400
```

**ロードマップとのコード番号差異（重要）**:
ロードマップは W040 / W041 と記載しているが、
- W040 は v61.7.0 で「type hole `_` inferred」として実装済み（取得済みコード）
- 本バージョンでは次の空きコードを使用する:
  - `perf_hint_large_collect` → **W041**（ロードマップの W040 に対応）
  - バックプレッシャー実行時警告（VM 統合）→ 本バージョンでは非スコープ（後送り）

**既存実装の確認**:
- `LintConfig { strict: bool, perf: bool }` は v61.8.0 で実装済み（`lint.rs` 行 52〜59）
  - `perf` フィールドは「将来用」として `#[allow(dead_code)]` 付きで既存
- `lint_program_with_config` は v61.8.0 実装済み（W041 呼び出しを追加する対象）
- W039・W040 は実装済み（W041 はその直後に追加）
- `ParallelConfig` パターン（v63.4.0）と同じ構造で `BackpressureConfig` を追加

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3416 tests passed, 0 failed を確認
- `lint.rs` で W040 が v61.7.0「type hole `_` inferred」として実装済み（W041 が次の空きコードであること）を確認
- `lint.rs` の `LintConfig` に `perf: bool` フィールドが存在することを確認（W041 のガード条件）
- `lint.rs` の `lint_program` 末尾が `check_w040_type_holes` で終わることを確認（W041 呼び出し追加位置）
- `toml.rs` に `BackpressureConfig` が存在しないことを確認（新規追加）
- `driver.rs` に `v63500_tests` が存在することを確認（`v63600_tests` の挿入位置）

**非スコープ注意**:
- バックプレッシャー実行時 VM 統合（`vm.rs` stage 間キューへの `warn_threshold` 超過警告 W042）
- `[backpressure]` 設定の VM への注入
- `site/` MDX 追加

---

## 実装スコープ

### 1. `lint.rs` — W041 lint ルール追加（3箇所）

#### ① `lint_program_with_config` — W041 呼び出しを追加

`check_w039_as_name_shadows_inner` / `check_w040_type_holes` が呼ばれている
`lint_program_with_config` 関数に W041 呼び出しを追加:

```rust
pub fn lint_program_with_config(program: &Program, config: &LintConfig) -> Vec<LintError> {
    let mut errors = lint_program(program);
    if config.strict {
        for e in &mut errors {
            if e.code == "W040" {
                e.message = format!("{} [strict]", e.message);
            }
        }
    }
    // v63.6.0: W041 は perf/strict モード下でのみ有効
    if config.perf || config.strict {
        check_w041_perf_hint_large_collect(program, &mut errors);
    }
    errors
}
```

#### ② `lint_program` — W041 呼び出しを追加

`check_w040_type_holes` の直後に以下を追加:

```rust
// v63.6.0: W041（perf/strict モードのみ発火 — lint_program_with_config 側でガード）
// check_w041_perf_hint_large_collect は lint_program_with_config 内で条件付き呼び出し
```

（注: `lint_program` はモード情報を持たないため W041 は `lint_program_with_config` 側でのみ呼び出す）

#### ③ W041 実装関数（ファイル末尾に追加）

```rust
// ── W041: perf hint — large collect without filter (v63.6.0) ──────────────────
fn check_w041_perf_hint_large_collect(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        let body = match item {
            Item::FnDef(fd) => &fd.body,
            Item::TrfDef(td) => &td.body,
            _ => continue,
        };
        check_w041_in_block(body, errors);
    }
}

fn check_w041_in_block(block: &Block, errors: &mut Vec<LintError>) {
    check_w041_in_expr(&block.expr, errors);
    for stmt in &block.stmts {
        match stmt {
            Stmt::Expr(e) => check_w041_in_expr(e, errors),
            Stmt::Bind(b) => check_w041_in_expr(&b.expr, errors),
            _ => {}
        }
    }
}

fn check_w041_in_expr(expr: &Expr, errors: &mut Vec<LintError>) {
    if let Expr::Collect(block, span) = expr {
        if !block_mentions_filter(block) {
            errors.push(LintError::new(
                "W041",
                "collect block without a filter may accumulate all rows — \
                 consider adding a filter condition to reduce memory pressure [perf]",
                span.clone(),
            ));
        }
        check_w041_in_block(block, errors);
    }
}

fn block_mentions_filter(block: &Block) -> bool {
    // ブロック内に "filter" への参照（Expr::Ident / Expr::FieldAccess）がある場合に true。
    block.stmts.iter().any(|s| stmt_mentions_name_w041(s, "filter"))
        || expr_mentions_name_w041(&block.expr, "filter")
}

fn stmt_mentions_name_w041(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::Expr(e) => expr_mentions_name_w041(e, name),
        Stmt::Bind(b) => expr_mentions_name_w041(&b.expr, name),
        _ => false,
    }
}

fn expr_mentions_name_w041(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Ident(n, _) => n == name,
        Expr::FieldAccess(_, field, _) => field == name,
        Expr::Apply(callee, args, _) => {
            expr_mentions_name_w041(callee, name)
                || args.iter().any(|a| expr_mentions_name_w041(a, name))
        }
        _ => false,
    }
}
```

### 2. `toml.rs` — `BackpressureConfig` 追加（6箇所）

`ParallelConfig`（v63.4.0）と同じパターン。`ParallelConfig` の直後（`BuildConfig` の直前）に追加する。

**① 構造体**（`// ── Build config` コメントの直前）:

```rust
// ── Backpressure config (v63.6.0) ───────────────────────────────────────────

/// `[backpressure]` section of fav.toml (v63.6.0).
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// バックプレッシャー戦略: "drop" | "block" | "sample"。デフォルト: "block"。
    pub strategy: String,
    /// stage 間キューの最大深度。デフォルト: 500。
    pub max_queue_depth: usize,
    /// キュー深度がこの値を超えると W042 警告を出す。デフォルト: 400。
    pub warn_threshold: usize,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        BackpressureConfig {
            strategy: "block".to_string(),
            max_queue_depth: 500,
            warn_threshold: 400,
        }
    }
}
```

**② `FavToml` フィールド**（`parallel: Option<ParallelConfig>` の直後）:

```rust
/// Optional backpressure configuration (v63.6.0).
pub backpressure: Option<BackpressureConfig>,
```

**③ `parse_fav_toml` ローカル変数**（`let mut parallel_cfg` の直後）:

```rust
let mut backpressure_cfg: Option<BackpressureConfig> = None;
```

**④ セクション検出**（`if trimmed == "[parallel]" { ... }` の直後）:

```rust
if trimmed == "[backpressure]" {
    section = "backpressure";
    continue;
}
```

**⑤ セクション処理**（`"parallel" => { ... }` ブロックの直後）:

```rust
"backpressure" => {
    let mut current = backpressure_cfg.take().unwrap_or_default();
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "strategy"         => current.strategy         = val.to_string(),
            "max_queue_depth"  => current.max_queue_depth  = val.parse::<usize>().unwrap_or(500),
            "warn_threshold"   => current.warn_threshold   = val.parse::<usize>().unwrap_or(400),
            _ => {}
        }
    }
    backpressure_cfg = Some(current);
}
```

**⑥ `FavToml { ... }` リテラル**（`parallel: parallel_cfg,` の直後）:

```rust
backpressure: backpressure_cfg,
```

### 3. `driver.rs` — `v63600_tests` 追加

`v63500_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63600_tests (v63.6.0) -- バックプレッシャー制御 W041 lint + [backpressure] 設定 --
#[cfg(test)]
mod v63600_tests {
    #[test]
    fn lint_w041_large_collect() {
        use crate::frontend::parser::Parser;
        // collect ブロック内に filter がない場合に W041 が発火することを確認
        let src = "public fn heavy() -> Int { collect { yield 1; yield 2; 0 } }";
        let prog = Parser::parse_str(src, "<test>").expect("parse ok");
        let config = crate::lint::LintConfig { strict: false, perf: true };
        let errors = crate::lint::lint_program_with_config(&prog, &config);
        assert!(
            errors.iter().any(|e| e.code == "W041"),
            "W041 should fire for collect without filter in perf mode: {:?}", errors
        );
    }

    #[test]
    fn backpressure_toml_parsed() {
        let toml = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[backpressure]\nstrategy = \"drop\"\nmax_queue_depth = 500\nwarn_threshold = 400\n";
        let config = crate::toml::parse_fav_toml_pub(toml);
        let bp = config.backpressure.expect("backpressure config should be parsed");
        assert_eq!(bp.strategy, "drop", "strategy should be 'drop'");
        assert_eq!(bp.max_queue_depth, 500, "max_queue_depth should be 500");
        assert_eq!(bp.warn_threshold, 400, "warn_threshold should be 400");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63600_tests` で 2 件 PASS
  - `lint_w041_large_collect` PASS（W041 が perf モードで発火）
  - `backpressure_toml_parsed` PASS（strategy / max_queue_depth / warn_threshold が正しくパース）
- `cargo test -j 8 -- --test-threads=8` で 3418 tests passed, 0 failed
- `CHANGELOG.md` 先頭に v63.6.0 エントリを追加
- `versions/roadmap/roadmap-v63.1-v64.0.md` v63.6.0 セクションに実績追記
- `versions/current.md` の「進行中」を v63.6.0（3418 tests）に更新

---

## 非スコープ

- バックプレッシャー実行時 VM 統合（`vm.rs` stage 間キューへの `warn_threshold` 超過 W042 警告）
- `[backpressure]` 設定の VM 実行エンジンへの注入
- W040 の既存 strict タグ付けロジックの変更（`lint_program_with_config` の W040 処理はそのまま）
- `site/` MDX ドキュメント追加

---

## 技術ノート

### W040 コード番号の衝突

ロードマップでは W040 を `perf_hint_large_collect` として指定しているが、
v61.7.0 で W040 は「type hole `_` inferred」として実装済み。
本バージョンでは次の空きコード W041 を使用する。
テスト名は `lint_w041_large_collect`（ロードマップの `lint_w040_large_collect` に対応）。

### W041 の発火条件

`lint_program_with_config` 内で `config.perf || config.strict` が true の場合のみ呼び出す。
`lint_program`（設定なし版）では W041 は発火しない。
これにより通常の `fav check` では W041 は表示されず、`--perf` / `--strict` フラグ時のみ表示される。

### `block_mentions_filter` の設計

`Expr::Ident("filter", _)` または `Expr::FieldAccess(_, "filter", _)` を
collect ブロックの直接の子式・子文から検索する。
深い再帰は避け（関数名を `_w041` サフィックスで限定）、既存の `lint.rs` ヘルパーとの名前衝突を防ぐ。

### `BackpressureConfig` のデフォルト値

`strategy = "block"` — block が最も安全なデフォルト（データロスなし）。
`max_queue_depth = 500` — ロードマップ例と同値。
`warn_threshold = 400` — ロードマップ例と同値（max の 80%）。

### `BindStmt.expr` フィールド

`Stmt::Bind(BindStmt)` の値フィールドは `b.value` ではなく `b.expr`（ast.rs:536 確認）。
`check_w041_in_block` / `stmt_mentions_name_w041` では `b.expr` を参照する。

### `FavToml` リテラルへの追加漏れ防止

v63.4.0 と同様、`FavToml` 構造体のリテラルが `driver.rs`・`checker.rs`・`resolver.rs` に存在する。
`backpressure: None` をすべてのリテラルに追加しないとコンパイルエラーになる。
`cargo build` でエラーが出た場合は `grep -rn "FavToml {" fav/src/` で漏れを確認すること。
