# v34.9.0 — 実装プラン

## 方針

upgrade-guide.mdx（新規）と ctx_migration フィクスチャ（新規 2 ファイル）の 3 本柱。
`cargo clean` は x.9.0 のため不要。

**前提**: v34.8.0 の `cmd_upgrade` / `MIGRATION.md` 完了済み。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.8.0` → `34.9.0` に変更。

---

### Step 2: upgrade-guide.mdx 作成

`site/content/docs/tools/upgrade-guide.mdx` を新規作成:

```markdown
---
title: "fav upgrade — プロジェクトアップグレードガイド"
description: "fav upgrade コマンドを使って既存プロジェクトを最新の Favnir スタイルに移行する"
---

# fav upgrade — プロジェクトアップグレードガイド

`fav upgrade` はプロジェクト全体を一括アップグレードするコマンドです。

## !Effect → Capability Context 移行

### 基本的な使い方

```bash
# 変更内容をプレビュー（ファイルは変更されません）
fav upgrade --from-effects --dry-run

# プロジェクト全体にインプレースで適用
fav upgrade --from-effects --in-place
```

### `fav migrate` との使い分け

| コマンド | 対象 | 用途 |
|---|---|---|
| `fav upgrade --from-effects` | プロジェクト全体 | 一括移行 |
| `fav migrate --from-effects` | 単一ファイル | ファイルごとの移行 |

## ワークフロー

1. W022 警告を確認: `fav lint src/`
2. ドライランで変更内容を確認: `fav upgrade --from-effects --dry-run`
3. インプレースで適用: `fav upgrade --from-effects --in-place`
4. 型チェック: `fav check`
5. テスト実行: `fav test`

## トラブルシューティング

**Q: 一部のファイルが移行されなかった**
A: 高階関数でエフェクトを受け渡すパターンは手動移行が必要です。
`fav migrate --from-effects` で個別に確認してください。

**Q: 移行後に型エラーが出る**
A: `fav check` でエラー箇所を確認し、`ctx: AppCtx` 引数が正しく追加されているかを確認してください。

詳細は [移行ガイド](/docs/tools/migration-effects) と [ctx 構文リファレンス](/docs/ctx-syntax-guide) を参照。
```

---

### Step 3: ctx_migration フィクスチャ作成

#### 3-1. `fav/tests/fixtures/ctx_migration/before.fav`

```favnir
// ctx 移行前: !Http エフェクトを使用（W022 警告が発生する）
fn fetch_orders(url: String) -> Result<String, String> !Http {
    HTTP.get(url)
}
```

#### 3-2. `fav/tests/fixtures/ctx_migration/after.fav`

```favnir
// ctx 移行後: AppCtx パラメータを使用
import runes/ctx

fn fetch_orders(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

---

### Step 4: driver.rs 更新

#### 4-1. `cargo_toml_version_is_34_8_0` スタブ化

```rust
fn cargo_toml_version_is_34_8_0() {
    // stubbed out in v34.9.0
}
```

#### 4-2. `v349000_tests` 挿入

v348000_tests 直後・`// ── v31.7.0 tests` の前に挿入。
`use super::*` なし（`include_str!` のみ使用）。

フィクスチャの `include_str!` パス:
- `upgrade-guide.mdx`: `include_str!("../../site/content/docs/tools/upgrade-guide.mdx")`
- `before.fav`: `include_str!("../tests/fixtures/ctx_migration/before.fav")`
- `after.fav`: `include_str!("../tests/fixtures/ctx_migration/after.fav")`

---

### Step 5: CHANGELOG.md 更新

```markdown
## [v34.9.0] — 2026-07-04

### Added
- `site/content/docs/tools/upgrade-guide.mdx` — fav upgrade コマンド公式ドキュメント
- `fav/tests/fixtures/ctx_migration/before.fav` — ctx 移行前テストフィクスチャ
- `fav/tests/fixtures/ctx_migration/after.fav` — ctx 移行後テストフィクスチャ

### Changed
- `versions/current.md` — 最新安定版を v34.9.0 に更新
```

---

### Step 6: benchmarks/v34.9.0.json 作成

```json
{
  "version": "34.9.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2581,
  "tests_failed": 0,
  "notes": "upgrade-guide.mdx 追加。ctx_migration フィクスチャ追加。v349000_tests 5 件追加。"
}
```

---

### Step 7: versions/current.md 更新

- `最新安定版`: `**v34.8.0**` → `**v34.9.0** — fav upgrade ドキュメント + ctx フィクスチャ`
- `cargo install` 行: `"34.8.0"` → `"34.9.0"`
- `進行中バージョン`: `なし（v34.8.0 完了直後）` → `なし（v34.9.0 完了直後）`
- `次に切る版`: `**v34.9.0**` → `**v35.0.0** — Production Ready 宣言`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v349000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.9.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
