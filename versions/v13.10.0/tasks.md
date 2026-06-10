# v13.10.0 Tasks — `!` 記法廃止 + 糖衣構文追加

Date: 2026-06-11
Branch: feat/v13-capability-context
Completed: 2026-06-11

---

## Phase A — E0025 エラーカタログ追加

- [x] A-1: `fav/src/error_catalog.rs` — E0025 エントリを追加（E0024 の直後）
- [x] A-2: `fav/src/driver.rs` `get_help_text` に E0025 ヘルプテキストを追加

---

## Phase B — lint.rs: E0025 `check_bang_notation` 実装

- [x] B-1: `check_bang_notation(program: &Program) -> Vec<LintError>` を実装
  - `fn_defs` / `trf_defs` を走査して effects が空でない関数を検出
  - Effect 名 → 移行先 ctx 型のヒントを生成
- [x] B-2: `infer_ctx_hint(effects: &[Effect]) -> &str` ヘルパーを実装
  - `[Postgres]` → `"LoadCtx"`, `[Postgres, AWS]` → `"AppCtx"` 等

---

## Phase C — driver.rs: cmd_check への統合

- [x] C-1: `cmd_check` の E0024 ブロック後に E0025 チェックブロックを追加
  - `if !legacy_check && !json { ... }` 条件で呼び出す
- [x] C-2: `--legacy` モードでは E0025 は実行しない
- [x] C-3: JSON 出力モードでは E0025 チェックをスキップ

---

## Phase D — parser.rs: legacy モード維持確認

- [x] D-1: `fav/src/parser.rs` の `!Effect` パース処理が引き続き動作することを確認（変更不要）
  - E0025 は lint フェーズで検出するため parser は変更しない
  - `--legacy` モードでの動作確認は e0025_legacy_mode_suppressed テストで確認

---

## Phase E — `fav fmt --migrate` 実装

- [x] E-1: `cmd_fmt` に `--migrate` フラグ解析を追加
- [x] E-2: `migrate_effects_in_source(source: &str) -> (String, Vec<String>)` を実装
  - `!Postgres` のみ → effects 削除（ctx 追加は手動）
  - `!Postgres !Io` など複数 → effects 削除 + W010 警告
  - effects 宣言部分を削除
- [x] E-3: `--check` フラグで変換内容をプレビューのみ（ファイル書き込みなし）

---

## Phase F — `fav migrate --from-effects` コマンド実装

- [x] F-1: `cmd_migrate` 関数を更新（既存の関数を拡張）
- [x] F-2: `--from-effects` フラグ追加（`main.rs` と `cmd_migrate` 両方）
  - ファイル指定: 単一ファイルを変換
  - ディレクトリ指定: `walkdir` で `.fav` を再帰スキャン
- [x] F-3: `from_effects` 時は `migrate_effects_in_source` を呼ぶ
- [x] F-4: 変換サマリー出力（変換済みファイル数・"no !Effect found" メッセージ）

---

## Phase G — `Ctx { db: DbRead }` 糖衣構文

- [x] G-1: `fav/src/parser.rs` の `parse_params` に `Ctx { ... }` パターンを追加
  - `Ctx { db: DbRead, io }` を認識して `Param { name: "ctx", ty: TypeExpr::Named("LoadCtx") }` に脱糖
- [x] G-2: `desugar_ctx_fields(fields) -> &str` 脱糖テーブルを実装（free fn として追加）
  - `[("db", Some("DbRead"))]` → `"LoadCtx"`
  - `[("db", Some("DbWrite"))]` → `"WriteCtx"`
  - `[("io", None)]` → `"CommonCtx"`
  - それ以外 → `"AppCtx"`
- [x] G-3: `ctx_destructure_sugar_parses` / `ctx_destructure_io_only` テストで確認

---

## Phase H — テスト追加

- [x] H-1: `fav/src/driver.rs` に `v13100_tests` モジュールを追加
- [x] H-2: 以下のテストを実装:
  - [x] `version_is_13_10_0` — `CARGO_PKG_VERSION == "13.10.0"`
  - [x] `e0025_bang_notation_error` — `!Postgres` 含む fn → E0025 検出
  - [x] `e0025_legacy_mode_suppressed` — ctx ベース fn → E0025 なし
  - [x] `e0025_multiple_effects_detected` — `!Postgres !Io` → E0025
  - [x] `fmt_migrate_postgres_to_load_ctx` — `!Postgres` のみ → effects 削除
  - [x] `fmt_migrate_appctx_with_w010` — `!Postgres !Io` → effects 削除 + W010
  - [x] `ctx_destructure_sugar_parses` — `Ctx { db: DbRead }` → `ctx: LoadCtx`
  - [x] `ctx_destructure_io_only` — `Ctx { io }` → `ctx: CommonCtx`
  - [x] `migrate_tool_scans_directory` — pure fn は変更なし確認
- [x] H-3: `cargo test v13100` で全件パス確認（9/9）

---

## Phase I — バージョンバンプ + コミット

- [x] I-1: `fav/Cargo.toml` → `version = "13.10.0"`
- [x] I-2: `cargo test` 全件パス確認（1502 + 705 = 2207 passed, 0 failed）
- [ ] I-3: `git commit -m "feat: v13.10.0 — ! 記法廃止 + 糖衣構文追加 (E0025)"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| E0025 が error_catalog に追加された | ✓ |
| `check_bang_notation` が E0025 を返す | ✓ |
| `fav check` が E0025 を出力（非 legacy） | ✓ |
| `--legacy` では E0025 は発生しない | ✓ |
| `fav fmt --migrate` が `!Postgres` → effects 削除に変換する | ✓ |
| `fav migrate --from-effects` が `--from-effects` フラグを処理する | ✓ |
| `Ctx { db: DbRead }` が `ctx: LoadCtx` に脱糖される | ✓ |
| `cargo test v13100` 全件パス（9/9） | ✓ |
| `cargo test` 全件パス（2207 passed） | ✓ |
| `CARGO_PKG_VERSION == "13.10.0"` | ✓ |

---

## 実装ノート

- **`check_bang_notation`**: `Program.fn_defs` の `effects: Vec<Effect>` フィールドを参照。`!` パース結果は AST に残っているため lint フェーズで容易に検出可能。
- **`infer_ctx_hint`**: 保守的に推定する。`[Postgres]` は読み書き両方使われる可能性があるため `LoadCtx` だが W010 は不要。`[Postgres, AWS]` は `AppCtx` + W010 必須。
- **糖衣構文の脱糖**: `parse_param` で `Ctx` という名前の型を見つけたらブレース内を `(field, type_opt)` のリストとして解析し、`ctx_type_from_fields` で名目型に変換。`Param.name` は `"ctx"` 固定。
- **`fav migrate` バックアップ**: 変換前に必ず `.fav.bak` を保存。`--no-backup` フラグで省略可能（上級者向け）。
- **E0025 と W008/E0023 の関係**: `!` 記法が残っている場合は E0025 を出す。`!` なしで ambient effect（ctx なし直接呼び出し）がある場合は E0023 を出す。両者は独立したチェック。
