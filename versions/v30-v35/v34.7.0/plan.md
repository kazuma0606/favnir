# v34.7.0 — 実装プラン

## 方針

ctx 構文リファレンスガイド（新規）、getting-started.mdx AppCtx セクション追加、
README.md ctx 移行シリーズ追記の 3 本柱。
`cargo clean` は x.7.0 のため不要。

**前提**: IoCtx / DbCtx / HttpClient / StreamClient は v34.5〜v34.6 で実装済み。
本バージョンは既存ファイルへの破壊的変更なし（additive のみ）。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.6.0` → `34.7.0` に変更。

---

### Step 2: site/content/docs/ctx-syntax-guide.mdx 作成

```markdown
---
title: "ctx 構文リファレンス"
description: "Favnir の Capability Context（AppCtx）構文の完全リファレンス"
---

# ctx 構文リファレンス

Favnir v34.5.0 以降、副作用は `!Effect` アノテーションではなく
Capability Context（`AppCtx` パラメータ）で表現します。

## 設計思想

`capability 引数がなければ純粋` — ctx を渡さない関数は副作用を持てません。
テスト時は `Ctx.mock(...)` で各フィールドをモック化できます。

## 基本構文

```favnir
import runes/ctx

fn fetch_data(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

`bind { field } <- ctx` で ctx から必要なフィールドだけを取り出します。

## 複数フィールドの分解

```favnir
fn run_etl(ctx: AppCtx, url: String) -> Result<Unit, String> {
    bind { http, db, io } <- ctx
    bind data <- http.get(url)
    bind _    <- db.execute("INSERT INTO log VALUES (?)", data)
    io.println("done")
    Result.ok(())
}
```

## AppCtx フィールド一覧

| フィールド | 型 | 対応する旧 !Effect | ファイル |
|---|---|---|---|
| `ctx.io` | `IoCtx` | `!Io` | `runes/ctx/io.fav` |
| `ctx.db` | `DbCtx` | `!DbRead` / `!DbWrite` | `runes/ctx/db.fav` |
| `ctx.http` | `HttpClient` | `!Http` | `runes/ctx/http.fav` |
| `ctx.stream` | `StreamClient` | `!Stream` | `runes/ctx/stream.fav` |

## Before / After 対比

### Before（!Effect 構文 — W022 警告が発生）

```favnir
fn fetch(url: String) -> Result<String, String> !Http {
    HTTP.get(url)
}
```

### After（ctx 構文）

```favnir
fn fetch(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

## テストでの利用

```favnir
import runes/ctx

fn test_fetch_data() -> Bool {
    bind ctx <- Ctx.mock({
        http: MockHttp.returns_ok("test data")
    })
    bind result <- fetch_data(ctx, "https://example.com")
    result == "test data"
}
```

## 自動移行

`!Effect` を使用しているファイルは `fav migrate --from-effects` で自動変換できます:

```bash
fav migrate --from-effects --in-place src/pipeline.fav
```

詳細は [migration-effects](/docs/tools/migration-effects) を参照。
```

---

### Step 3: getting-started.mdx 更新（追記）

`## 次のステップ` セクションの**前**に以下を追加:

```markdown
## Capability Context を使う

v34.5.0 以降、副作用は `AppCtx` パラメータで表現します:

```favnir
import runes/ctx

fn fetch_and_save(ctx: AppCtx, url: String) -> Unit {
    bind { http, io } <- ctx
    bind data <- http.get(url)
    io.println(data)
}
```

詳細は [ctx 構文リファレンス](/docs/ctx-syntax-guide) を参照。
```

---

### Step 4: README.md 更新（追記）

`**v34.0**...マイルストーンを宣言しました。` 行の**直後**に以下を追加:

```markdown
v34.5.0〜v34.7.0 で、`!Effect` アノテーションを廃止し Capability Context（AppCtx）に一本化しました。
`fav migrate --from-effects` で既存コードを自動移行できます。
```

---

### Step 5: driver.rs 更新

1. `cargo_toml_version_is_34_6_0` を空スタブ化
2. `v346000_tests` 直後・`// ── v31.7.0 tests` の前に `v347000_tests` を挿入

挿入位置の確認:

```bash
grep -n "v346000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
```

```rust
// ── v34.7.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v347000_tests {
    #[test]
    fn cargo_toml_version_is_34_7_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.7.0"), "Cargo.toml must contain '34.7.0'");
    }

    #[test]
    fn ctx_syntax_guide_exists() {
        let src = include_str!("../../site/content/docs/ctx-syntax-guide.mdx");
        assert!(
            src.contains("AppCtx"),
            "ctx-syntax-guide.mdx must document AppCtx"
        );
    }

    #[test]
    fn ctx_syntax_guide_covers_bind() {
        let src = include_str!("../../site/content/docs/ctx-syntax-guide.mdx");
        assert!(
            src.contains("bind"),
            "ctx-syntax-guide.mdx must show bind destructure syntax"
        );
    }

    #[test]
    fn getting_started_updated() {
        let src = include_str!("../../site/content/learn/getting-started.mdx");
        assert!(
            src.contains("AppCtx"),
            "getting-started.mdx must mention AppCtx"
        );
    }

    #[test]
    fn readme_has_ctx_migration_ref() {
        let src = include_str!("../../README.md");
        assert!(
            src.contains("v34.5"),
            "README.md must reference v34.5 ctx migration series"
        );
    }
}
```

**注意**: `use super::*` は**不要**（`include_str!` のみ使用）。
`readme_has_ctx_migration_ref` のパス `"../../README.md"` は
`fav/src/` → `fav/` → `favnir/README.md` を指す（2 階層上）。

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v34.7.0] — 2026-07-04

### Added
- `site/content/docs/ctx-syntax-guide.mdx` — ctx 構文完全リファレンスガイド

### Changed
- `site/content/learn/getting-started.mdx` — AppCtx を使ったパイプライン例を追加
- `README.md` — v34.5〜v34.7 ctx 移行シリーズの記録を追加
- `versions/current.md` — 最新安定版を v34.7.0 に更新
```

---

### Step 7: benchmarks/v34.7.0.json 作成

```json
{
  "version": "34.7.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2571,
  "tests_failed": 0,
  "notes": "ctx-syntax-guide.mdx 追加。getting-started.mdx AppCtx セクション追加。README.md ctx 移行シリーズ追記。v347000_tests 5 件追加。"
}
```

---

### Step 8: versions/current.md 更新

- `最新安定版` 行: `**v34.6.0** — Rune ファイル ctx 移行` → `**v34.7.0** — ドキュメント・examples ctx 移行`
- `cargo install` 行: `"34.6.0"` → `"34.7.0"`
- `進行中バージョン`: `なし（v34.6.0 完了直後）` → `なし（v34.7.0 完了直後）`
- `次に切る版`: `**v34.7.0**` → `**v34.8.0** — 安定化・最終調整`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v347000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.7.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
