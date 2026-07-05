# v33.7.0 — Spec: エフェクトシステム移行準備 確認・テスト補強

## 概要

v33.7.0 は **エフェクトシステム移行準備**（`!Effect` → ctx 移行ツール）の確認・テスト補強バージョン。

ロードマップ v33.7 のテーマ「v34 系での `!Effect` 廃止に向けた準備」の中核となる
移行ツールは v13.10.0 で既に実装済みである。

| コンポーネント | 実装済み | バージョン |
|---|---|---|
| `check_bang_notation` — `!Effect` 記法検出 → E0025 | ✓ | v13.10.0 |
| `migrate_effects_in_source(src)` — `!Effect` → ctx 自動書き換え | ✓ | v13.10.0 |
| `resolve_use_effects(from_version, from_effects)` — 移行モード判定 | ✓ | v13.10.0 |
| `fav migrate --from-effects` CLI コマンド | ✓ | v13.10.0 |
| W010 — 複数エフェクト → AppCtx への移行警告 | ✓ | v13.10.0 |
| E0025 error_catalog メッセージ | ✓ | v13.10.0 |
| `v13100_tests` — 7 件（`e0025_bang_notation_error` 等）| ✓ | v13.10.0 |

v33.7.0 では新規実装は行わず、`v337000_tests` で動作を確認・記録するにとどまる
（v33.1〜v33.6 と同じ「確認・記録」パターン）。

> **ロードマップとの対応**: ロードマップ v33.7 では「W022 deprecated_effect_annotation 追加」
> 「IoCtx interface」「fav migrate --effects」を列挙しているが、機能の実質は
> v13.10.0 の `check_bang_notation`（E0025）/ `migrate_effects_in_source` で既に提供されている。
> 本バージョンはこれらを Performance & Tooling フェーズの記録として確認する。

---

## エフェクトシステム移行準備 仕様確認

### migrate_effects_in_source の動作

```rust
pub fn migrate_effects_in_source(src: &str) -> (String, Vec<String>)
```

- `!Effect` を含む fn 宣言を ctx 引数スタイルに書き換える
- 戻り値: `(書き換え後ソース, W010 警告メッセージリスト)`
- 単一エフェクト: `!Postgres` → 効果削除（W010 なし）
- 複数エフェクト: `!Postgres !Io` → `AppCtx` + W010 警告
- エフェクトなし（純粋関数）: 変更なし（W010 なし）

### resolve_use_effects の動作

```rust
pub fn resolve_use_effects(from_version: Option<&str>, from_effects: bool) -> bool
```

- `from_effects: true` または `from_version = Some("v13")` / `Some("13")` のとき `true` を返す
- それ以外（`None`、`Some("v12")` 等）は `false` を返す
- v13 系からの移行のみを自動検出するバージョン判定（v12 以前は対象外）

### 冪等性保証

`migrate_effects_in_source` を同一ソースに 2 回適用しても結果は変わらない（冪等性）。
移行後のソースはすでに `!Effect` を含まないため、2 回目は no-op になる。

---

## 追加するテスト（v337000_tests — 4 件）

`v337000_tests` は v33.x 系テストの標準パターン:
- `use super::*` **なし**
- 必要なものだけモジュール冒頭で明示 import

```rust
mod v337000_tests {
    use crate::driver::{migrate_effects_in_source, resolve_use_effects};
}
```

テスト名は v13100_tests（`e0025_bang_notation_error` / `fmt_migrate_postgres_to_load_ctx` /
`fmt_migrate_appctx_with_w010` / `ctx_destructure_sugar_parses` / `ctx_destructure_io_only` /
`migrate_tool_scans_directory`）と被らないよう設計する。

### テスト 1: バージョン確認

```rust
fn cargo_toml_version_is_33_7_0() {
    let src = include_str!("../Cargo.toml");
    assert!(src.contains("33.7.0"), "Cargo.toml must contain '33.7.0'");
}
```

### テスト 2: ベンチマーク存在確認

```rust
fn benchmark_v33_7_0_exists() {
    let src = include_str!("../../benchmarks/v33.7.0.json");
    assert!(src.contains("33.7.0"), "benchmarks/v33.7.0.json must contain '33.7.0'");
}
```

### テスト 3: migrate_effects_in_source の冪等性

v13100_tests は「1 回の移行」を確認する。
v33.7.0 では 2 回連続適用で結果が変わらない（冪等性）ことを記録する。

```rust
fn migrate_effects_idempotent() {
    // 2回連続適用しても結果が変わらない（冪等性保証）
    let src = "fn load() -> Result<List<String>, String> !Postgres {\n    Result.ok(List.empty())\n}\n";
    let (first, _) = migrate_effects_in_source(src);
    let (second, w010s) = migrate_effects_in_source(&first);
    assert_eq!(first, second, "migrate_effects_in_source should be idempotent");
    assert!(w010s.is_empty(), "second pass should produce no W010 warnings");
}
```

### テスト 4: resolve_use_effects — v13 バージョン指定で true を返す

v13100_tests は `resolve_use_effects` を直接テストしていない。
v33.7.0 では `from_version = Some("v13")` が `true` を返すことを確認し、
バージョン判定ロジックを記録する。

```rust
fn resolve_use_effects_from_v13() {
    // "v13" または "13" を指定すると effects 移行モードになる
    assert!(resolve_use_effects(Some("v13"), false), "v13 should activate effects migration");
    assert!(resolve_use_effects(Some("13"), false), "\"13\" should also activate effects migration");
    assert!(!resolve_use_effects(Some("v12"), false), "v12 should NOT activate effects migration");
    assert!(!resolve_use_effects(None, false), "no version + no flag should NOT activate");
}
```

---

## テストモジュールの配置

`v337000_tests` は `v336000_tests` の閉じ括弧（`}`）の直後、
かつ `// ── v31.7.0 tests` コメントの前に挿入する。

---

## 完了条件

- `Cargo.toml` version = `"33.7.0"`
- `cargo_toml_version_is_33_6_0` が空スタブになっていること
- `cargo test --bin fav v337000` — 4/4 PASS
- `cargo test` — 全件 PASS（2524 件、0 failures）
- `CHANGELOG.md` に `[v33.7.0]` セクション
- `benchmarks/v33.7.0.json` 存在かつ `tests_passed` が実測値
- `benchmarks/v33.7.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- `versions/current.md` を v33.7.0 に更新
- `tasks.md` がすべて `[x]` で COMPLETE に更新されていること
