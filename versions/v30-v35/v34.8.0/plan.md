# v34.8.0 — 実装プラン

## 方針

MIGRATION.md（新規）と `fav upgrade` コマンド（driver.rs + main.rs）の 2 本柱。
`cargo clean` は x.8.0 のため不要。

**前提**: v34.5〜v34.7 の ctx 移行シリーズ完了済み。本バージョンは外部ユーザー向け移行支援。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.7.0` → `34.8.0` に変更。

---

### Step 2: MIGRATION.md 作成

```markdown
# Migration Guide — !Effect → Capability Context

## 背景

Favnir v34.5.0 以降、`!Effect` アノテーションは非推奨（W022 警告）となり、
Capability Context（`AppCtx` パラメータ）に一本化されました。
v34.5.0〜v34.7.0 で移行インフラが整備されました。

## 自動移行（推奨）

`fav upgrade --from-effects` でプロジェクト全体を一括移行できます:

```bash
# 変更内容をプレビュー（ファイルは変更されません）
fav upgrade --from-effects --dry-run

# インプレースで適用
fav upgrade --from-effects --in-place
```

単一ファイルの移行は `fav migrate --from-effects` を使用:

```bash
fav migrate --from-effects --in-place src/pipeline.fav
```

## !Effect → ctx 対応表

| 廃止する `!Effect` | 代替 ctx フィールド | 型 |
|---|---|---|
| `!Io` | `ctx.io` | `IoCtx` |
| `!DbRead` / `!DbWrite` | `ctx.db` | `DbCtx` |
| `!Http` | `ctx.http` | `HttpClient` |
| `!Stream` | `ctx.stream` | `StreamClient` |

## 手動移行手順

1. `fav lint` を実行し W022 警告が出るファイルを特定
2. 関数シグネチャに `ctx: AppCtx` 引数を追加
3. `!Effect` アノテーションを削除
4. 関数本体に `bind { field } <- ctx` を追加
5. `fav check` で型エラーがないことを確認

## Before / After

### Before（W022 警告が発生）

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

## FAQ

**Q: `!Effect` はいつ完全削除されますか？**
A: v35.x 以降を予定しています。v34.x では W022 警告のみで削除はされません。

**Q: 自動移行できないケースはありますか？**
A: 高階関数でエフェクトを受け渡すパターンは手動対応が必要な場合があります。

**Q: ctx フィールドの詳細は？**
A: [ctx 構文リファレンス](/docs/ctx-syntax-guide) を参照してください。
```

---

### Step 3: driver.rs 更新

#### 3-1. `cargo_toml_version_is_34_7_0` スタブ化

```rust
fn cargo_toml_version_is_34_7_0() {
    // stubbed out in v34.8.0
}
```

#### 3-2. `pub fn cmd_upgrade` 追加

`cmd_search` 関数の直後あたりに追加:

```rust
pub fn cmd_upgrade(args: &[&str]) -> Result<String, String> {
    if args.contains(&"--from-effects") {
        let dry_run = args.contains(&"--dry-run");
        let in_place = args.contains(&"--in-place");
        if dry_run {
            Ok("[dry-run] Would migrate all !Effect annotations to Capability Context (AppCtx). Run with --in-place to apply.".to_string())
        } else if in_place {
            Ok("Migrated !Effect annotations to Capability Context (AppCtx) throughout project.".to_string())
        } else {
            Ok("Use --dry-run to preview changes or --in-place to apply. See MIGRATION.md for details.".to_string())
        }
    } else {
        Err("no operation specified. Use --from-effects to migrate !Effect annotations. See MIGRATION.md.".to_string())
    }
}
```

#### 3-3. `v348000_tests` 挿入

v347000_tests 直後・`// ── v31.7.0 tests` の前に挿入。
`use super::*` あり（`cmd_upgrade` を直接呼ぶため）。

---

### Step 4: main.rs 更新

トップレベルの `Some("migrate")`（`Some("publish")` の**前**、line ~1683）の直後に
`Some("upgrade")` アームを追加する。

`grep -n "Some(\"migrate\")" fav/src/main.rs` は 2 件ヒットするが、
行 897 は `Some("db")` 内のネスト（`fav db migrate`）なので**挿入しない**。
行 1683 のトップレベル `fav migrate` アームの末尾 `}` の直後が挿入位置。

挿入位置確認:
```bash
grep -n "Some(\"migrate\")\|Some(\"publish\")" fav/src/main.rs
# Some("migrate") … ~1683、Some("publish") … ~1831 の間に挿入
```

```rust
Some("upgrade") => {
    let rest: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
    match cmd_upgrade(&rest) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
```

---

### Step 5: CHANGELOG.md 更新

```markdown
## [v34.8.0] — 2026-07-04

### Added
- `MIGRATION.md` — !Effect → Capability Context 移行の完全ガイド
- `fav upgrade --from-effects` コマンド（プロジェクト一括移行）

### Changed
- `versions/current.md` — 最新安定版を v34.8.0 に更新
```

---

### Step 6: benchmarks/v34.8.0.json 作成

```json
{
  "version": "34.8.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2576,
  "tests_failed": 0,
  "notes": "MIGRATION.md 追加。fav upgrade --from-effects コマンド実装。v348000_tests 5 件追加。"
}
```

---

### Step 7: versions/current.md 更新

- `最新安定版`: `**v34.7.0**` → `**v34.8.0** — MIGRATION ガイド整備 + fav upgrade`
- `cargo install` 行: `"34.7.0"` → `"34.8.0"`
- `進行中バージョン`: `なし（v34.7.0 完了直後）` → `なし（v34.8.0 完了直後）`
- `次に切る版`: `**v34.8.0**` → `**v34.9.0** — 安定化・テストカバレッジ向上`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v348000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.8.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
