# v63.6.0 Plan — バックプレッシャー制御（W041 lint + `[backpressure]` 設定）

Version: 63.6.0
Status: 未着手

---

## 実装順序

### Step 1: `lint.rs` — W041 関数群追加

ファイル末尾（`check_w040_type_holes` 関数群の直後）に追加する:

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

`cargo build` でエラーなしを確認。

### Step 2: `lint.rs` — `lint_program_with_config` に W041 呼び出しを追加 + `#[allow(dead_code)]` 削除

既存の `lint_program_with_config` 関数の末尾（`errors` を return する直前）に追加する:

```rust
// v63.6.0: W041 は perf/strict モード下でのみ有効
if config.perf || config.strict {
    check_w041_perf_hint_large_collect(program, &mut errors);
}
```

`cargo build` でエラーなしを確認。

また、`LintConfig` の `perf` フィールドは W041 で実際に使用するため、以下の `#[allow(dead_code)]` アトリビュートと「将来用」コメントを削除する:

```rust
// 削除対象:
/// 将来用: パフォーマンス系警告の有効化（v61.8.0 では常に false、v62+ で追加予定）。
#[allow(dead_code)]
pub perf: bool,

// 変更後:
/// `--perf` フラグ有効時 true。W041 以降のパフォーマンス系 lint を有効化する（v63.6.0）。
pub perf: bool,
```

### Step 3: `toml.rs` — `BackpressureConfig` 構造体追加

`// ── Build config (v62.7.0)` コメントの直前（`// ── Parallel config (v63.4.0)` の直前または直後）に追加する。
`ParallelConfig` の直後に追加するのが自然:

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

`cargo build` でエラーなしを確認。

### Step 4: `toml.rs` — `FavToml` フィールド追加 + `parse_fav_toml` 6箇所更新（構造体・フィールド・変数・セクション検出・セクション処理・リテラル）

**① `FavToml` フィールド**（`parallel: Option<ParallelConfig>` の直後）:
```rust
/// Optional backpressure configuration (v63.6.0).
pub backpressure: Option<BackpressureConfig>,
```

**② ローカル変数**（`let mut parallel_cfg` の直後）:
```rust
let mut backpressure_cfg: Option<BackpressureConfig> = None;
```

**③ セクション検出**（`if trimmed == "[parallel]" { ... }` の直後）:
```rust
if trimmed == "[backpressure]" {
    section = "backpressure";
    continue;
}
```

**④ セクション処理**（`"parallel" => { ... }` ブロックの直後）:
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

**⑤ `FavToml { ... }` リテラル**（`parallel: parallel_cfg,` の直後）:
```rust
backpressure: backpressure_cfg,
```

`cargo build` でエラーなしを確認。コンパイルエラーが出た場合は
`grep -rn "FavToml {" fav/src/` で `backpressure: None` 追加漏れを確認すること。

### Step 5: `driver.rs` — `v63600_tests` 追加

`v63500_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63600_tests (v63.6.0) -- バックプレッシャー制御 W041 lint + [backpressure] 設定 --
#[cfg(test)]
mod v63600_tests {
    #[test]
    fn lint_w041_large_collect() {
        use crate::frontend::parser::Parser;
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

### Step 6: テスト・ドキュメント更新

```bash
cargo test --bin fav v63600_tests   # 2件 PASS を確認
cargo test -j 8 -- --test-threads=8  # 3418 tests passed, 0 failed を確認
```

1. `CHANGELOG.md` 先頭に v63.6.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.6.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.6.0（3418 tests）に更新
4. `tasks.md` を COMPLETE に更新

---

## 設計メモ

### 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/lint.rs` | `check_w041_perf_hint_large_collect` + 関連ヘルパー追加 + `lint_program_with_config` 更新 |
| `fav/src/toml.rs` | `BackpressureConfig` 構造体 + `FavToml` フィールド + `parse_fav_toml` 更新（6箇所） |
| `fav/src/driver.rs` | `v63600_tests` 追加 |
| `fav/src/middle/checker.rs` | `FavToml` リテラルに `backpressure: None` 追加（2箇所） |
| `fav/src/middle/resolver.rs` | `FavToml` リテラルに `backpressure: None` 追加（3箇所） |

### `BindStmt.expr` フィールド名

`ast.rs` の `BindStmt` は `pub expr: Expr`（`b.value` ではなく `b.expr`）。
`stmt_mentions_name_w041` / `check_w041_in_block` では `b.expr` を参照する。
