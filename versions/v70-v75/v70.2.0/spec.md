# v70.2.0 Spec — `fav migrate` 完成（構文自動移行ツール）

Date: 2026-08-09
Status: 計画中

---

## Background

`cmd_migrate` は driver.rs に既に存在し、以下が実装済み:
- ファイル収集（`file`, `dir`, fav.toml 自動探索）
- `in_place` / `dry_run` / `check` モード
- `!Effect` アノテーション → ctx-based 変換（`migrate_effects_in_line`）
- v1.x → v2.0 構文変換（`migrate_source`）
- fav.toml フォーマット移行（`migrate_fav_toml_source`）

**未実装の gap:**

1. `resolve_use_effects` が `"v13"` / `"13"` しか扱えない
   → `"v35"` / `"35"`（ctx スタイル正式導入バージョン）を追加する必要がある
2. IO stdlib コール書き換えが未実装
   - `IO.args()` → `ctx.io.argv()`
   - `IO.read_file(path)` → `ctx.io.read_file_raw(path)`
   - `IO.write_file(path, data)` → `ctx.io.write_file_raw(path, data)`（`IO.read_file` より先に処理すること）
   - `IO.println(msg)` → `ctx.io.println(msg)`
3. `dry_run` フラグが `let _ = (dry_run, ...)` で no-op になっている
   → デフォルト挙動（`in_place` 非指定時は差分表示）は既に実装済みのため、`dry_run` フラグを明示的に `!in_place` の alias として有効化する

**注記**: `!IO` / `!HTTP` / `!DB` 等すべてのエフェクトアノテーションの変換は既存の `migrate_effects_in_line` が処理済み。v70.2.0 では `resolve_use_effects` の対象バージョン拡張のみで動作する。

---

## Goals

1. `fav migrate --from v35 pipeline.fav` で `!IO` 等のエフェクトアノテーションを `ctx: AppCtx` に変換できる
2. 同時に IO stdlib コール（`IO.println()` 等）を `ctx.io.*` 形式に書き換える
3. dry-run 出力が「line N: !IO → ctx: AppCtx (fn signature updated)」形式で可読性が高い
4. 既存テストが全 pass（3561 件）
5. 新規 Rust テスト 2 件追加 → 3563 tests

---

## Syntax / API Examples

```bash
# エフェクトアノテーション + IO stdlib を一括変換（dry-run）
$ fav migrate --from v35 pipeline.fav
Migrating pipeline.fav...
  line 43: !IO → ctx: AppCtx (fn signature updated)
  line 54: IO.args() → ctx.io.argv()
  line 61: IO.read_file(path) → ctx.io.read_file_raw(path)
  line 74: IO.println(msg) → ctx.io.println(msg)
✓ Written: pipeline.fav.migrated

# インプレース変換
$ fav migrate --from v35 --in-place pipeline.fav
migrated: pipeline.fav

# v13 系（従来通り動作）
$ fav migrate --from v13 --in-place pipeline.fav
```

---

## 変換ルール

### エフェクトアノテーション（既存 `migrate_effects_in_line` を活用）

既存実装が `!IO` / `!HTTP` / `!DB` / `!AWS` 等すべての Effect を処理する。
単一 `!IO` は `CommonCtx`、複合エフェクトや複雑な組み合わせは `AppCtx` に変換される。

| 旧構文 | 新構文 |
|---|---|
| `fn f(x: T) -> R !IO` | `fn f(ctx: CommonCtx, x: T) -> R`（単体 IO）|
| `fn f(x: T) -> R !DB` | `fn f(ctx: LoadCtx, x: T) -> R`（読み取り専用 DB）|
| `fn f(x: T) -> R !HTTP` | `fn f(ctx: AppCtx, x: T) -> R`（HTTP は常に AppCtx）|
| `fn f(x: T) -> R !IO !DB` | `fn f(ctx: AppCtx, x: T) -> R`（複合は AppCtx）|

### IO stdlib コール（新規 `migrate_io_calls_in_source`）

> **置換順序**: `IO.write_file` を `IO.read_file` より先に処理すること。
> `IO.write_file` の文字列内に `IO.` で始まる部分文字列 `IO.read_file` が含まれないが、
> 実装の一貫性と将来の安全性のため、長い文字列を先に置換する原則を守る。

| 旧構文 | 新構文 |
|---|---|
| `IO.write_file(path, data)` | `ctx.io.write_file_raw(path, data)`（先に処理）|
| `IO.read_file(path)` | `ctx.io.read_file_raw(path)` |
| `IO.println(msg)` | `ctx.io.println(msg)` |
| `IO.args()` | `ctx.io.argv()` |

---

## Success Criteria

- [ ] `resolve_use_effects` が `"v35"` / `"35"` を `true` で返す
- [ ] `migrate_io_calls_in_source(src)` が IO stdlib コールを `ctx.io.*` に変換する
- [ ] `fav migrate --from v35` 実行時に `migrate_effects_in_source` と `migrate_io_calls_in_source` の両方が適用される
- [ ] `cargo test v702000` で 2 件 pass
- [ ] `cargo test` 全体で 3563 tests pass

---

## Error Codes

新規エラーコードなし（既存 W010 は引き続き使用）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `resolve_use_effects` に `"v35"` / `"35"` 追加、`migrate_io_calls_in_source` 新規追加、`cmd_migrate` で両変換を適用、`v702000_tests` モジュール追加 |
| `fav/Cargo.toml` | `version` を `"70.1.0"` → `"70.2.0"` に更新 |
| `CHANGELOG.md` | v70.2.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v70.2.0 に更新 |
