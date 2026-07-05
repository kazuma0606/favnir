# v34.5.0 — 実装プラン

## 方針

W022 lint ルール追加 + IoCtx rune 新規作成 + 移行ガイド MDX の 3 本柱。
`cargo clean` は x.5.0 のため不要。

**前提**: `fav migrate --from-effects` は v13.10.0 で実装済み（main.rs / driver.rs）。
本バージョンでは新実装は lint.rs / runes/ctx/ / site/ に限定する。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.4.0` → `34.5.0` に変更。

---

### Step 2: 既存 lint テストへの影響調査

W022 を `lint_program()` に追加する前に、影響する既存テストを確認:

```bash
grep -n "fav_lint\|lint_program" fav/src/driver.rs | grep -v "//" | head -40
```

`!Effect` 付き fn を含むテストソースが `warnings.len()` を count している場合、
W022 追加後にカウントが変わる。該当テストのソースを純粋関数に変更するか、
`fav_lint(src, &["W022"])` で W022 を suppress するか選択する。

---

### Step 3: lint.rs — W022 追加（事前に影響調査）

W022 を `lint_program()` に追加する前に、影響するテストを特定する:

```bash
grep -n "lint_program" fav/src/driver.rs | grep -v "//" | head -20
```

影響するテストのソースに `!Effect` fn が含まれる場合:
- そのテストが `warnings.len()` を厳密にカウントしているなら、ソースを純粋関数に変更する
- または、そのテストの `warnings` 検査に W022 が含まれることを許容するよう条件を緩和する

**lint.rs ヘッダーコードリスト更新**: lint.rs 先頭のコメントリストに W022 を追記すること。

`fav/src/lint.rs` の末尾（W021 の後）に以下を追加:

```rust
// ── W022: deprecated_effect_annotation ──────────────────────────────────────

pub fn check_w022_deprecated_effect_annotation(program: &Program, errors: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let has_real_effect = fd.effects.iter().any(|e| e != &Effect::Pure);
            if has_real_effect {
                errors.push(LintError::new(
                    "W022",
                    format!(
                        "function `{}` uses deprecated `!Effect` annotation \
                         — migrate to Capability Context using `fav migrate --from-effects`",
                        fd.name
                    ),
                    fd.span.clone(),
                ));
            }
        }
    }
}
```

`lint_program()` に組み込む（W021 呼び出しの直後）:

```rust
check_w022_deprecated_effect_annotation(program, &mut errors);
```

---

### Step 4: runes/ctx/io.fav 作成

`runes/ctx/io.fav` を新規作成:

```favnir
// runes/ctx/io.fav — IoCtx interface（v34.5.0）
// !Io エフェクトの Capability Context 移行用インターフェース。
// `!Io` を使用している関数は ctx.io を通じて IO 操作を行う。

interface IoCtx {
    fn println(ctx: IoCtx, msg: String) -> Unit
    fn read_line(ctx: IoCtx) -> Result<String, String>
    fn read_file(ctx: IoCtx, path: String) -> Result<String, String>
    fn write_file(ctx: IoCtx, path: String, content: String) -> Result<Unit, String>
    fn env(ctx: IoCtx, key: String) -> Option<String>
}
```

---

### Step 5: site/content/docs/tools/migration-effects.mdx 作成

```markdown
---
title: "!Effect から Capability Context への移行ガイド"
description: "Favnir v34.5 で追加された W022 警告への対応と ctx ベース構文への移行手順"
---

# `!Effect` から Capability Context への移行ガイド

Favnir v34.5.0 から、`!Effect` アノテーションは非推奨となり W022 警告が発生する。
本ガイドでは `fav migrate --from-effects` を使った自動移行と手動移行の手順を説明する。

## W022 警告について

```bash
fav lint src/pipeline.fav
# W022: function `fetch_data` uses deprecated `!Effect` annotation
#        — migrate to Capability Context using `fav migrate --from-effects`
```

`!Effect` アノテーションは Capability Context（`ctx` パラメータ）に移行する。
`!Effect` は v35.x で削除予定。

## 自動移行: `fav migrate --from-effects`

```bash
# ドライラン（変更内容を確認）
fav migrate --from-effects --dry-run src/pipeline.fav

# インプレース変換
fav migrate --from-effects --in-place src/pipeline.fav

# ディレクトリ全体を変換
fav migrate --from-effects --dir src/
```

## Before / After 例

### Before（!Effect 構文）

```favnir
fn fetch_data(url: String) -> String !Http {
    Http.get(url)
}

fn write_result(data: String) -> Unit !Io {
    IO.println(data)
}
```

### After（ctx 構文）

```favnir
fn fetch_data(ctx: AppCtx, url: String) -> String {
    bind { http } <- ctx
    http.get(url)
}

fn write_result(ctx: AppCtx, data: String) -> Unit {
    bind { io } <- ctx
    io.println(data)
}
```

## `!Effect` → ctx フィールド 対応表

| 廃止する `!Effect` | 代替 ctx フィールド | 型 |
|---|---|---|
| `!Io` | `ctx.io` | `IoCtx` |
| `!Http` | `ctx.http` | `HttpClient` |
| `!DbRead` / `!DbWrite` | `ctx.db` | `DbRead` / `DbWrite` |
| `!Postgres` | `ctx.db` | `PgConn` |
| `!Redis` | `ctx.redis` | `RedisClient` |
| `!Llm` | `ctx.llm` | `LlmClient` |
| `!Snowflake` | `ctx.warehouse` | `SnowflakeConn` |
| `!Trace` | `ctx.tracer` | `Tracer` |

## W022 の一時抑制

移行が完了するまで W022 を抑制するには:

```bash
fav lint --allow W022 src/pipeline.fav
```

## AppCtx を使った完全な例

```favnir
import runes/ctx

fn run_pipeline(ctx: AppCtx) -> Unit {
    bind result <- fetch_data(ctx, "https://api.example.com/data")
    write_result(ctx, result)
}
```

`AppCtx` は `IoCtx` / `HttpClient` / `DbRead` / `DbWrite` 等の全フィールドを持つ汎用コンテキスト。
```

---

### Step 6: driver.rs 更新

1. `cargo_toml_version_is_34_4_0` を空スタブ化
2. `v344000_tests` 直後・`// ── v31.7.0 tests` の前に `v345000_tests` を挿入

挿入位置の確認:

```bash
grep -n "v344000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
```

```rust
// ── v34.5.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v345000_tests {
    #[test]
    fn cargo_toml_version_is_34_5_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.5.0"), "Cargo.toml must contain '34.5.0'");
    }

    #[test]
    fn w022_deprecated_effect_annotation_fires() {
        let src = "fn fetch(url: String) -> String !Http { url }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        let mut warnings = Vec::new();
        crate::lint::check_w022_deprecated_effect_annotation(&prog, &mut warnings);
        assert!(
            warnings.iter().any(|w| w.code == "W022"),
            "W022 must fire when !Http is used: {:?}", warnings
        );
    }

    #[test]
    fn io_ctx_rune_exists() {
        let src = include_str!("../../runes/ctx/io.fav");
        assert!(
            src.contains("IoCtx"),
            "runes/ctx/io.fav must define IoCtx interface"
        );
    }

    #[test]
    fn migration_guide_page_exists() {
        let src = include_str!("../../site/content/docs/tools/migration-effects.mdx");
        assert!(
            src.contains("W022"),
            "migration-effects.mdx must mention W022"
        );
    }

    #[test]
    fn migration_guide_covers_ctx_syntax() {
        let src = include_str!("../../site/content/docs/tools/migration-effects.mdx");
        assert!(
            src.contains("AppCtx") || src.contains("ctx"),
            "migration-effects.mdx must cover ctx-based syntax"
        );
    }
}
```

**注意**: `w022_deprecated_effect_annotation_fires` は `crate::frontend::lexer::Lexer` / `Parser` /
`crate::lint::check_w022_deprecated_effect_annotation` を絶対パスで呼び出す（W021 テストと同一パターン）。
`use super::*` は**不要**。

---

### Step 7: CHANGELOG.md 更新

```markdown
## [v34.5.0] — 2026-07-04

### Added
- `fav/src/lint.rs` — W022 `deprecated_effect_annotation` lint ルール追加
- `runes/ctx/io.fav` — IoCtx interface 定義（`!Io` → `ctx.io` 移行用）
- `site/content/docs/tools/migration-effects.mdx` — `!Effect` → ctx 移行ガイド

### Changed
- `versions/current.md` — 最新安定版を v34.5.0 に更新
```

---

### Step 8: benchmarks/v34.5.0.json 作成

```json
{
  "version": "34.5.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2561,
  "tests_failed": 0,
  "notes": "W022 deprecated_effect_annotation lint ルール追加。runes/ctx/io.fav (IoCtx) 追加。migration-effects.mdx 追加。v345000_tests 5 件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

---

### Step 9: versions/current.md 更新

- `最新安定版` 行: `**v34.4.0** — セキュリティ審査 v2` → `**v34.5.0** — !Effect 廃止・コンテキスト構文統一`
- `cargo install` 行: `"34.4.0"` → `"34.5.0"`
- `進行中バージョン`: `なし（v34.4.0 完了直後）` → `なし（v34.5.0 完了直後）`
- `次に切る版`: `**v34.5.0**` → `**v34.6.0** — Rune ファイル ctx 移行`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v345000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.5.0.json` の `tests_passed` を実測値で確定
  （実測値が想定 2561 と異なる場合は spec.md の完了条件も更新する）
- `tasks.md` を COMPLETE に更新
