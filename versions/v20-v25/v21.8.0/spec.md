# v21.8.0 仕様書 — `fav migrate` 強化

## 概要

`fav migrate` コマンドを拡張し、バージョン間の自動コード移行ツールとして完成させる。

ロードマップ v21.8 の機能:
- `--from <ver> --to <ver>` 構文（バージョン指定による移行種別の明示的選択）
- `fav.toml` 形式の移行（`--config fav.toml`）
- `--dry-run` の正式ドキュメント化と実装修正
- 移行サマリー出力（変更 X 件 / 未変更 Y 件）

**スコープ外（v21.8 では実装しない）:**
- v14 以外のバージョン間移行（`--from v14 --to v15` 等）
- 構文レベルの AST 変換（テキストレベル変換のみ）
- `fav.toml` 以外の設定ファイル（`.favrc` 等）の移行
- 対話式移行ウィザード
- `--config` と `--check` の組み合わせ（`--config` が優先して早期 `return` するため `--check` は無視される）
- `--from v13 --to v2` のような矛盾した `--from`/`--to` 組み合わせへの警告（`--to` は将来拡張用に受け付けるが現在は無視）

---

## 現状分析

### 既存 `fav migrate` の状態

| 機能 | 状態 | 備考 |
|---|---|---|
| `trf` → `stage` / `flw` → `seq` 移行 | 実装済み | `migrate_source()` |
| `!Effect` → Ctx 移行 | 実装済み | `--from-effects` フラグ / `migrate_effects_in_source()` |
| `--in-place` 書き込み | 実装済み | |
| `--dry-run` | **未実装**（パラメータが `_dry_run`） | デフォルト動作（非 in-place）が事実上のドライラン |
| `--check` モード | 実装済み | 変更が必要なファイルを列挙して exit 1 |
| 移行サマリー | **部分実装** | 「All files are already...」のみ。変更件数が表示されない |
| `--from / --to` フラグ | **未実装** | |
| `fav.toml` 移行 | **未実装** | |

---

## アーキテクチャ

### `--from <ver> --to <ver>` フラグ

既存の移行関数への明示的なルーティングフラグ。

| フラグ | 等価な旧フラグ | 呼び出す関数 |
|---|---|---|
| `--from v1 --to v2` | （デフォルト） | `migrate_source()` |
| `--from v13 --to v14` | `--from-effects` | `migrate_effects_in_source()` |
| `--from v13`（`--to` 省略） | `--from-effects` | `migrate_effects_in_source()` |

**`migrate_effects_in_source` の動作:** `!Effect` 注記を関数シグネチャから除去し、W010 警告で `ctx` パラメータの手動追加を促す。`ctx` パラメータは自動追加されない（テキストレベル変換の限界）。

`--from-effects` は後方互換性のため残す（非推奨化はしない）。

### `fav.toml` 移行（`--config fav.toml`）

旧形式 `fav.toml` のキー/セクション名をテキストレベルで置換する。

**移行内容:**

| 旧形式 | 新形式 | 備考 |
|---|---|---|
| `[rune_dependencies]` | `[dependencies]` | セクション名変更 |
| `rune_version = "..."` | `version = "..."` | キー名変更 |
| `rune_path = "..."` | `path = "..."` | キー名変更 |

`migrate_fav_toml_source(src: &str) -> String` を新規実装。

`--config <file>` フラグ: 対象を fav.toml に絞って移行処理を実行。
`--in-place` と組み合わせると実際に書き込む。なしなら diff 表示（dry-run 相当）。

### `--dry-run` の正式実装

現在、`cmd_migrate` の `_dry_run: bool` パラメータは使われていない。
デフォルト（`--in-place` なし）の動作が事実上のドライランだが、CLI ドキュメント上で明示されていない。

v21.8.0 での変更:
- `_dry_run` → `dry_run` にリネーム（アンダースコア除去）
- `--dry-run` フラグを使った場合と `--in-place` なしの場合で動作を統一（どちらも diff 表示のみ）
- help テキスト相当のコメントを追加

**実装上の注意:**
`dry_run` パラメータは受け付けるが `--in-place` なしのデフォルト動作が既に等価なため、内部的に `let _ = (dry_run, to_version)` で受け流す（将来の拡張のために引数として残す）。`--dry-run --in-place` を同時に指定した場合は `--in-place` が優先される。

### 移行サマリー

変更の有無にかかわらず、最後に必ずサマリーを出力:

```
Migration complete: 3 file(s) migrated, 5 file(s) already up-to-date.
```

`--check` モードでは変更が必要なファイル数のみ:

```
2 file(s) need migration.
```

---

## 変更ファイル一覧

### fav（Rust）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 更新 | `migrate_fav_toml_source()` 追加、`cmd_migrate` 更新（`dry_run` 修正・サマリー・`--from-effects` 追加呼び出し） |
| `fav/src/main.rs` | 更新 | `--from` / `--to` / `--config` フラグ解析追加 |
| `fav/Cargo.toml` | 更新 | `version = "21.7.0"` → `"21.8.0"` |
| `fav/src/driver.rs` | 更新 | `v218000_tests` 追加（8 件） |

### CHANGELOG / docs

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `CHANGELOG.md` | 更新 | v21.8.0 エントリ追加 |
| `site/content/docs/cli/migrate.mdx` | 新規 | `fav migrate` ドキュメント |

---

## `migrate_fav_toml_source` API

```rust
pub fn migrate_fav_toml_source(src: &str) -> String
```

行単位でテキスト置換:
1. `[rune_dependencies]` → `[dependencies]`
2. `rune_version = ` → `version = `（先頭スペース保持）
3. `rune_path = ` → `path = `（先頭スペース保持）

既存の `migrate_line` / `migrate_source` と同様のテキストレベル実装。

---

## CLI 仕様

### 追加フラグ

```
fav migrate [options] [<file>]

追加オプション:
  --from <version>      移行元バージョン（v1, v13）
  --to <version>        移行先バージョン（v2, v14）。--from v13 の場合デフォルト v14
  --config <file>       fav.toml のみを対象として移行（<file> = fav.toml のパス）
```

### 既存フラグ（変更なし）

```
  --in-place            ファイルを直接書き換える
  --dry-run             変更のプレビュー（デフォルト動作。--in-place なしと同等）
  --check               変更が必要なファイルを表示して exit 1
  --dir <dir>           対象ディレクトリを指定
  --from-effects        !Effect → Ctx 移行（--from v13 --to v14 の別名）
```

### 使用例

```bash
# !Effect → Ctx 移行（v13 → v14）— 3 つの等価な書き方
fav migrate --from v13 --to v14 src/
fav migrate --from v13 src/
fav migrate --from-effects src/

# trf/flw → stage/seq 移行（v1 → v2）
fav migrate --from v1 --to v2 src/

# fav.toml 形式の移行
fav migrate --config fav.toml --in-place

# ドライラン（デフォルト）
fav migrate --dry-run src/

# CI チェック
fav migrate --check src/
```

---

## テスト一覧（v218000_tests、8 件）

| テスト名 | 内容 |
|---|---|
| `version_is_21_8_0` | Cargo.toml に `"21.8.0"` が含まれる |
| `migrate_routing_v13_uses_effects` | `resolve_use_effects(Some("v13"), false)` が `true` — ルーティングヘルパーを直接テスト |
| `migrate_routing_v1_applies_migrate_source` | `migrate_source` / `migrate_effects_in_source` 各関数の動作確認 |
| `migrate_toml_rune_deps_section` | `[rune_dependencies]` → `[dependencies]` に変換される |
| `migrate_toml_rune_version_and_path_keys` | `rune_version`・`rune_path` 両キーが変換される |
| `migrate_toml_no_change_on_modern` | 現行 fav.toml フォーマットは変換されない（末尾改行あり/なし両方確認） |
| `changelog_has_v21_8_0` | CHANGELOG.md に `[v21.8.0]` が含まれる |
| `migrate_mdx_exists` | `site/content/docs/cli/migrate.mdx` が存在する |

---

## 完了条件

- [ ] `fav migrate --from v13 --to v14 src/` が `--from-effects` と同等に動作する
- [ ] `fav migrate --config fav.toml --in-place` が旧形式 fav.toml を移行できる
- [ ] `--dry-run` フラグが受け付けられる（`_dry_run` → `dry_run` にリネーム済み。`--in-place` なしのデフォルト動作と等価）
- [ ] 移行サマリーが常に表示される
- [ ] `--from-effects` が後方互換として引き続き動作する
- [ ] `cargo test v218000` — 8/8 PASS
- [ ] `cargo test` — リグレッションなし（1831 件以上合格）
- [ ] `CHANGELOG.md` に v21.8.0 エントリ
- [ ] `site/content/docs/cli/migrate.mdx` 作成済み
