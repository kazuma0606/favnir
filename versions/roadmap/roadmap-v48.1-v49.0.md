# Roadmap v48.1.0 〜 v49.0.0 — Module & Package 2.0

Date: 2026-07-15
Status: 計画中（v48.0 完了後に開始）

---

## 前提

- 直前完了: v48.0.0「Standard Library 2.0」（v48.0 宣言後、tests ≥ 3040）
- マスターロードマップ: `roadmap-v45.1-v50.0.md`
- 本文書はマスターの v49.0 スプリント部分の詳細版

---

## 目標

パッケージ import とローカル import を構文で明確に分離し、
**`fav.toml` が依存関係の唯一の真実となるモジュールシステムを完成させる**。

---

## バージョン計画

### v48.1.0 — import 構文刷新 AST + parser（パッケージ）

```favnir
// fav.toml [runes] に宣言されたパッケージ → 引用符なし
import kafka
import postgres as db
```

`ImportStmt` ノードに `kind: ImportKind` フィールド追加（`Package` / `Local`）。
`parser.rs` で引用符なし import をパッケージ import として解析。
既存の `import rune "kafka"` 構文との共存を一時的に維持。

**完了条件**: Rust テスト 2 件（実績推定 3042 tests passed, 0 failed）
- `import_package_parses`
- `import_package_with_alias`

**実績**: 3047 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.2.0 — import 構文刷新（ローカルファイル）

```favnir
// ./ から始まる → ローカル .fav ファイル
import "./src/helpers" as helpers
import "./stages/validate" as validate
```

`parser.rs` で `"./"` prefix を持つ import を `ImportKind::Local` として解析。
`driver.rs` のファイル解決ロジックを Local import に対応させる。

**完了条件**: Rust テスト 2 件（実績推定 3049 tests passed, 0 failed）
- `import_local_parses`
- `import_local_relative_path`

**実績**: 3049 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.3.0 — `fav.toml [runes]` 解決ロジック

```toml
[runes]
kafka    = "2.1.0"
postgres = "1.0.0"
```

`fav.toml` の `[runes]` テーブルを `HashMap<String, String>` としてパース。
E0417（パッケージ未登録エラー）を `error_catalog.rs` に正式追加。

**スコープ分割（合意済み）**:
- v48.3.0: `toml.rs` パース + `error_catalog.rs` E0417 定義のみ
- v48.5.0: `checker.rs` での `ImportKind::Package` × `FavToml.runes` 突き合わせ（E0417 実発行）

**完了条件**: Rust テスト 2 件（実績推定 3051 tests passed, 0 failed）
- `rune_resolution_from_toml`
- `e0417_rune_not_in_toml`

**実績**: 3051 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.4.0 — `fav install` コマンド

```bash
fav install kafka       # runes/kafka/ にローカル展開
fav install             # fav.toml [runes] 全件インストール
```

`fav.toml [runes]` を読んで `runes/<name>/` にスタブディレクトリを作成する MVP 実装。
`driver.rs` に `install_rune_stubs`（内部）と `cmd_install_runes`（CLI エントリ）追加。
`main.rs` に `"install-rune"` アーム追加。
既存の `cmd_install`（`[dependencies]` 専用）は変更しない。

**完了条件**: Rust テスト 2 件（実績推定 3053 tests passed, 0 failed）
- `fav_install_creates_rune_dir`
- `fav_install_all_from_toml`

**実績**: 3053 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.5.0 — import エイリアス完全化 + 旧構文 deprecation

旧 `import rune "kafka"` 構文を W035 警告（非推奨）化。
`import kafka as k` の完全サポート確認。
`lint.rs` に `W035: legacy_import_rune` ルール追加。

**完了条件**: Rust テスト 2 件（実績推定 3055 tests passed, 0 failed）
- `import_alias_resolves`
- `legacy_import_rune_w035`

**実績**: 3056 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.6.0 — 循環 import 検出 + E0418

```
E0418: circular import detected
  a.fav -> b.fav -> a.fav
```

import グラフを構築しトポロジカルソートで循環検出。
循環時 E0418 を `error_catalog.rs` に追加して発行。
`driver.rs` の依存グラフ解析ロジックに組み込む。

**完了条件**: Rust テスト 2 件（実績推定 3058 tests passed, 0 failed）
- `circular_import_e0418`
- `non_circular_import_ok`

**実績**: 3058 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.7.0 — rune.toml 標準化

全公式 rune の `rune.toml` を統一フォーマットに規定する。
`[rune]` セクション必須・`[connection]` 非標準セクション除去確認。
`toml.rs` に `validate_rune_toml` ヘルパー追加（必須フィールドチェック）。

**スコープ注記**: 全公式 rune の `rune.toml` 実ファイル更新は v48.7.0 スコープ外。
v48.4.0 の `install_rune_stubs` が生成するスタブは `[rune]+name+version+entry+description` の
標準フォーマット済みのため、実ファイル変更は不要。本バージョンは `validate_rune_toml` 関数追加のみ。

**完了条件**: Rust テスト 2 件（実績推定 3060 tests passed, 0 failed）
- `rune_toml_standard_format`
- `rune_toml_no_connection_section`

**実績**: 3061 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.8.0 — `fav rune` コマンド群

```bash
fav rune list           # インストール済み rune 一覧
fav rune info kafka     # rune の詳細（バージョン・関数一覧）
fav rune remove kafka   # rune 削除
```

`driver.rs` に `cmd_rune_list` / `cmd_rune_info` / `cmd_rune_remove` 追加。
`main.rs` に `"rune"` アームと各サブコマンド分岐を追加。

**実装注記（v48.8.0 スコープ修正）**: `main.rs "rune"` アームと `rune_cmd.rs` の `cmd_rune_list` / `cmd_rune_info` / `cmd_rune_uninstall` は既存実装済み（`rune_modules/` 対象）。v48.8.0 では v48.4.0 系 `runes/` ディレクトリ向けの純粋ヘルパー関数（`list_installed_runes` / `get_rune_version`）を `driver.rs` に追加する。`main.rs` / `rune_cmd.rs` への変更は行わない。

**完了条件**: Rust テスト 2 件（実績推定 3063 tests passed, 0 failed）
- `fav_rune_list_shows_installed`
- `fav_rune_info_shows_version`

**実績**: 3063 tests passed, 0 failed（2026-07-18 完了）✅

---

### v48.9.0 — Module ドキュメント + migration guide + v49.0 前調整

import 構文移行ガイド MDX（旧 `import rune "X"` → 新 `import X` の手順）。
`site/content/docs/module-system.mdx` および `migration-guide-import.mdx` 作成。
v49.0 前コードフリーズ。

**完了条件**: Rust テスト 2 件（実績推定 3065 tests passed, 0 failed）
- `module_system_doc_exists`
- `import_migration_guide_exists`

**実績**: 3065 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.0.0 — Module & Package 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「パッケージ import とローカル import が構文で明確に分離され、
>  `fav.toml` が依存関係の唯一の真実となった。
>
>  これが Favnir v49.0 — Module & Package 2.0 の姿である。」

**完了条件**:
- v48.1〜v48.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3065**）
- `v49000_tests` 4 件 pass:
  - `cargo_toml_version_is_49_0_0`
  - `changelog_has_v49_0_0`
  - `milestone_has_module_package_v2` — MILESTONE.md に `"Module & Package 2.0"` が含まれる
  - `readme_mentions_module_package_v2`
- `MILESTONE.md` に `"Module & Package 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3069 tests passed, 0 failed（2026-07-18 完了）✅

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント（アクティブ）: `versions/roadmap/roadmap-v47.1-v48.0.md`
- 次サブスプリント（v49.0 完了後に開始）: `versions/roadmap/roadmap-v49.1-v50.0.md`
- 達成宣言: `MILESTONE.md`
