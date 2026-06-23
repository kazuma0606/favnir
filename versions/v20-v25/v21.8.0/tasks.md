# v21.8.0 — `fav migrate` 強化 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `migrate_fav_toml_source(src: &str) -> String` 実装（driver.rs）

- [x] **事前確認**: `grep -n "pub fn migrate_source" fav/src/driver.rs` で挿入位置を確認
- [x] `migrate_fav_toml_source` を `migrate_source` の直後に実装:
  - `[rune_dependencies]` → `[dependencies]`（先頭インデント保持）
  - `rune_version = ...` → `version = ...`（先頭インデント保持）
  - `rune_path = ...` → `path = ...`（先頭インデント保持）
  - `migrate_source` と同じ末尾改行保持ロジック
- [x] 未変更行はそのままコピー（`else` ブランチで `out.push_str(line)`）
- [x] `pub fn` として公開（tests から直接呼ぶため）

---

### T2: `cmd_migrate` 更新（driver.rs）

- [x] **事前確認**: `grep -n "pub fn cmd_migrate" fav/src/driver.rs` でシグネチャを確認
- [x] `_dry_run: bool` → `dry_run: bool` にリネーム（アンダースコア除去）
- [x] 以下 3 パラメータを追加:
  - `from_version: Option<&str>`
  - `to_version: Option<&str>`
  - `config_file: Option<&str>`
- [x] ルーティングヘルパー `pub fn resolve_use_effects(from_version: Option<&str>, from_effects: bool) -> bool` を追加（テスト可能化）
- [x] `--config` ルーティング追加（先に処理して `return`）:
  - `config_file` が `Some(path)` → `migrate_fav_toml_source` を呼ぶ
  - `--in-place` あり → ファイルを直接書き込む + `"Migrated: {path}"` を表示
  - `--in-place` なし → 行単位 diff を表示し `"(dry-run: use --in-place to apply changes)"` を出力（書き込まない）
  - 変更なし → `"fav.toml is already up-to-date."` を表示
- [x] `--from v13` ルーティング追加:
  - `let use_effects = resolve_use_effects(from_version, from_effects);`
- [x] `--from v1` / unknown バージョンの処理（`use_effects == false` のとき）:
  - `Some("v1") | Some("1")` → `migrate_source()`（既存動作、警告なし）
  - それ以外の `Some(v)` → `eprintln!("warning: unknown --from version '{}', defaulting to v1→v2 migration", v)` + `migrate_source()`
- [x] 移行サマリー出力を追加（末尾に必ず出力）:
  - `!check` のとき: `"Migration complete: {} file(s) migrated, {} file(s) already up-to-date."`
  - `check && any_needs_migration` のとき: `"{} file(s) need migration."` + `process::exit(1)`
  - `check && !any_needs_migration` のとき: `"All files are already up-to-date."`
  - 既存の `if changed_count == 0 && !check { println!("All files are already...") }` を削除してサマリーに統合
- [x] `cargo check` を実行してコンパイルエラーがないことを確認

---

### T3: `main.rs` CLI フラグ追加

- [x] **事前確認**: `grep -n "cmd_migrate" fav/src/main.rs` で呼び出し箇所を確認
- [x] `Some("migrate")` ブランチに以下の変数を追加:
  ```rust
  let mut from_version: Option<String> = None;
  let mut to_version: Option<String> = None;
  let mut config_file: Option<String> = None;
  ```
- [x] `while` ループ内の `match` アームに追加（`i += 2` でスキップ）:
  - 値が欠落した場合 `eprintln! + process::exit(1)`（`--dir` と同じパターン。`unwrap_or_default()` 禁止）
- [x] `"--from"` / `"--to"` / `"--config"` が位置引数 `file` に誤判定されないことを確認
- [x] `cmd_migrate` 呼び出しを更新（新パラメータ 3 件追加）:
  ```rust
  cmd_migrate(
      file.as_deref(), in_place, dry_run, check, dir.as_deref(), from_effects,
      from_version.as_deref(), to_version.as_deref(), config_file.as_deref(),
  );
  ```
- [x] `cargo check` でコンパイルエラーがないことを確認

---

### T4: Cargo.toml + `#[ignore]` 追加

- [x] `fav/Cargo.toml` の `version = "21.7.0"` → `"21.8.0"` に変更
- [x] `driver.rs` の `v217000_tests::version_is_21_7_0` のみに `#[ignore]` を追加（他の v217000_tests テストは引き続き実行する）

---

### T5: `v218000_tests` — 8 件追加（driver.rs）

- [x] **事前確認**: `grep -n "mod v217000_tests" fav/src/driver.rs | head -3` で追加位置を確認
- [x] `v217000_tests` の後に `mod v218000_tests` を追加
- [x] `repo_path` ヘルパーを実装（`Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(rel)`）
- [x] 以下 8 件のテストを実装（plan.md T5 の最終コードを参照）:
  - [x] `version_is_21_8_0` — `Cargo.toml` に `"21.8.0"` が含まれる
  - [x] `migrate_routing_v13_uses_effects` — `resolve_use_effects(Some("v13"), false)` が `true`、`resolve_use_effects(None, false)` が `false` 等（ルーティングヘルパーを直接テスト）
  - [x] `migrate_routing_v1_applies_migrate_source` — `migrate_source` で `trf→stage`、`migrate_effects_in_source` で `!Effect→Ctx`（各移行関数の動作確認）
  - [x] `migrate_toml_rune_deps_section` — `[rune_dependencies]` → `[dependencies]` に変換される
  - [x] `migrate_toml_rune_version_and_path_keys` — `rune_version`・`rune_path` 両方が変換される
  - [x] `migrate_toml_no_change_on_modern` — 現行 fav.toml は変換されない（末尾改行あり・なし両方確認）
  - [x] `changelog_has_v21_8_0` — `CHANGELOG.md` に `[v21.8.0]` が含まれる
  - [x] `migrate_mdx_exists` — `site/content/docs/cli/migrate.mdx` が存在する
- [x] `cargo test v218000` — 8/8 PASS を確認

---

### T6: CHANGELOG + `site/content/docs/cli/migrate.mdx`

- [x] `CHANGELOG.md` の先頭に v21.8.0 エントリを追加:
  ```
  ## [v21.8.0] — 2026-06-20
  ### Added
  - `fav migrate --from v13 --to v14` — バージョン指定による移行種別の明示的選択
  - `fav migrate --from v1 --to v2` — trf/flw → stage/seq 移行の明示指定
  - `fav migrate --config fav.toml` — `fav.toml` 形式の自動移行
  - `migrate_fav_toml_source()` — `[rune_dependencies]`/`rune_version` 等の旧形式を変換
  - 移行サマリー出力（`Migration complete: X file(s) migrated, Y file(s) already up-to-date.`）
  ### Fixed
  - `--dry-run` フラグのパラメータ名を `_dry_run` → `dry_run` に修正
  ```
- [x] `site/content/docs/cli/migrate.mdx` を新規作成（以下のセクションを含む）:
  - 概要
  - コマンドリファレンス（全フラグ一覧表）
  - 移行パス表（`--from/--to` の組み合わせ）
  - 使用例（5 例）
  - `fav.toml` 移行の詳細

---

## テスト一覧（v218000_tests、8件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_8_0` | Cargo.toml に `"21.8.0"` が含まれる |
| `migrate_routing_v13_uses_effects` | `resolve_use_effects` のルーティングロジックを直接テスト |
| `migrate_routing_v1_applies_migrate_source` | `migrate_source`・`migrate_effects_in_source` の各動作確認 |
| `migrate_toml_rune_deps_section` | `[rune_dependencies]` → `[dependencies]` |
| `migrate_toml_rune_version_and_path_keys` | `rune_version`・`rune_path` 両キーが変換される |
| `migrate_toml_no_change_on_modern` | 現行 fav.toml は変換されない（べき等性・末尾改行あり/なし） |
| `changelog_has_v21_8_0` | `CHANGELOG.md` に `[v21.8.0]` が含まれる |
| `migrate_mdx_exists` | `site/content/docs/cli/migrate.mdx` が存在する |

---

## 完了条件チェックリスト

- [x] `fav migrate --from v13 --to v14 src/` が `--from-effects` と同等に動作する
- [x] `fav migrate --config fav.toml --in-place` が旧形式 fav.toml を移行できる
- [x] `--dry-run` フラグが明示的に機能する（`_dry_run` → `dry_run`）
- [x] 移行サマリーが常に表示される
- [x] `--from-effects` が後方互換として引き続き動作する
- [x] `cargo test v218000` — 8/8 PASS
- [x] `cargo test` — リグレッションなし（1831 件以上合格）
- [x] `CHANGELOG.md` に v21.8.0 エントリ
- [x] `site/content/docs/cli/migrate.mdx` 作成済み

---

## 優先度

```
T1（migrate_fav_toml_source）  ← 最初。T5 の基盤
T2（cmd_migrate 更新）         ← T1 後。cargo check 即確認
T3（main.rs CLI フラグ）       ← T2 後。cargo check 即確認
T4（Cargo.toml）               ← いつでも
T5（tests）                    ← T1, T2 完了後
T6（CHANGELOG + MDX）          ← 最後
```
