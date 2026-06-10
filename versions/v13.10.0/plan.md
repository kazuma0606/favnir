# v13.10.0 Plan — `!` 記法廃止 + 糖衣構文追加

Date: 2026-06-11

---

## Phase A — E0025 エラーカタログ追加

**ファイル**: `fav/src/error_catalog.rs`

E0025 エントリを E0024 の直後に追加:

```rust
ErrorEntry {
    code: "E0025",
    title: "bang notation removed",
    category: "syntax",
    description: "The `!Effect` notation is no longer supported in standard mode. ...",
    example: "fn load() -> Result<Loaded, String> !Postgres  // E0025",
    fix: "Migrate to capability-context style: fn load(ctx: LoadCtx) -> Result<Loaded, String>\nRun `fav migrate --from-effects <file>` to auto-migrate.",
},
```

**ファイル**: `fav/src/driver.rs` — `get_help_text` に E0025 ヘルプを追加。

---

## Phase B — lint.rs: E0025 `check_bang_notation` 実装

**ファイル**: `fav/src/lint.rs`

1. `check_bang_notation(program: &Program) -> Vec<LintError>` を実装:
   - `program.fn_defs` と `program.trf_defs` を走査
   - `fn_def.effects` が空でない場合 → E0025 を生成
   - `LintError { code: "E0025", message: "...", line: fn_def.line, ... }` を返す
2. Effect 名 → 移行先 ctx 型のヒントテーブルを実装:
   - `Postgres` / `Snowflake` → `LoadCtx` または `WriteCtx`
   - `AWS` → `WriteCtx` または `LoadCtx`
   - `Io` → `ctx.io.println(...)`
   - `Http` → `ctx.http.*`

---

## Phase C — driver.rs: cmd_check への統合

**ファイル**: `fav/src/driver.rs`

1. `cmd_check` の E0024 ブロック後に E0025 チェックブロックを追加:
   ```rust
   if !legacy_check && !json {
       let e0025s = crate::lint::check_bang_notation(&prog);
       if !e0025s.is_empty() {
           // エラー出力 → process::exit(1)
       }
   }
   ```
2. `--legacy` モードでは E0025 を実行しない
3. JSON 出力モードでは E0025 チェックをスキップ

---

## Phase D — parser.rs: legacy モード分岐

**ファイル**: `fav/src/parser.rs`

現在の `!Effect` パース処理はそのまま維持。
`check_bang_notation` が lint フェーズで検出するアプローチを採用（parser 変更は最小限）。

> Note: parser レベルで E0025 を出すよりも、lint フェーズで出す方が
> `--legacy` フラグとの統合が容易。また AST に effects が残るため
> `fav migrate` での参照も可能。

---

## Phase E — `fav fmt --migrate` 実装

**ファイル**: `fav/src/driver.rs`

`cmd_fmt` に `--migrate` フラグ処理を追加:

1. ファイルをパース → `program.fn_defs` / `program.trf_defs` を走査
2. effects を持つ関数を発見したらシグネチャを変換:
   - effects セットから ctx 型を推定する `infer_ctx_type(effects) -> &str` を実装
   - `!Postgres` のみ → `LoadCtx`（保守的）
   - `!Postgres + !AWS` → `AppCtx`（W010 付き）
   - `!Io` のみ → `CommonCtx`
3. 変換後のソースを書き戻す（`--dry-run` フラグで確認のみ）
4. W010 箇所を列挙して出力

---

## Phase F — `fav migrate --from-effects` コマンド実装

**ファイル**: `fav/src/driver.rs`

`cmd_migrate` 関数を追加:
- 引数: `--from-effects <path>`（ファイルまたはディレクトリ）
- ディレクトリの場合は `walkdir` で `.fav` ファイルを再帰スキャン
- 各ファイルに `--migrate` と同じ変換を適用
- 元ファイルを `.fav.bak` として保存
- 変換サマリーを出力

---

## Phase G — `Ctx { db: DbRead }` 糖衣構文

**ファイル**: `self/compiler.fav`（または `fav/src/parser.rs`）

`parse_fn_def_params` で `Ctx { ... }` パターンを認識し、
`ParamKind::CtxDestructure { fields: Vec<(String, String)> }` に変換:

1. `Ctx { db: DbRead, io }` → `(ctx_type: LoadCtx, param_name: ctx)`
2. `ctx_type_from_fields(fields) -> &str` 脱糖テーブルを実装
3. 型チェックは脱糖後の通常 ctx 型として処理

> 実装は parser.rs か compiler.fav のどちらかで行う。
> parser.rs で行う場合は AST に `Param::CtxSugar` バリアントを追加。
> compiler.fav で行う場合は `parse_fn_params` を拡張。
> **方針**: Rust 側の parser.rs で実装（compiler.fav のパーサは手を入れにくい）。

---

## Phase H — テスト追加

**ファイル**: `fav/src/driver.rs`

`v13100_tests` モジュールを追加し以下を実装:

1. `version_is_13_10_0` — `CARGO_PKG_VERSION == "13.10.0"`
2. `e0025_bang_notation_error` — `!Postgres` を含む fn → E0025 検出
3. `e0025_legacy_mode_suppressed` — `--legacy` では E0025 なし
4. `e0025_multiple_effects_detected` — `!Postgres !AWS` を含む fn → E0025
5. `fmt_migrate_postgres_to_load_ctx` — `!Postgres` のみ → `LoadCtx` に変換
6. `fmt_migrate_appctx_with_w010` — 複数 `!` → `AppCtx` 変換 + W010
7. `ctx_destructure_sugar_parses` — `Ctx { db: DbRead }` → `ctx: LoadCtx` 脱糖
8. `ctx_destructure_io_only` — `Ctx { io }` → `ctx: CommonCtx`
9. `migrate_tool_scans_directory` — `fav migrate --from-effects` でディレクトリスキャン

---

## Phase I — バージョンバンプ + コミット

1. `fav/Cargo.toml` → `version = "13.10.0"`
2. `cargo test` 全件パス確認（目標: 1500+ tests）
3. `git commit -m "feat: v13.10.0 — ! 記法廃止 + 糖衣構文追加 (E0025)"`

---

## 実装順序と依存関係

```
A (error_catalog) ← B (lint.rs) ← C (driver/check)
                                 ← E (fmt --migrate)
                                 ← F (migrate cmd)
D (parser分岐) — 独立
G (Ctx sugar) — 独立
H (tests) ← A,B,C,E,F,G 完了後
I (bump+commit) ← H 完了後
```

---

## リスク・注意点

1. **parser.rs の `!Effect` 解析は削除しない**: lint フェーズ検出のため AST に effects を残す必要あり。削除は v14.0.0 で行う。
2. **`fav migrate` は非可逆操作**: `.fav.bak` バックアップを必ず作成。
3. **ctx 型推定の保守性**: 推定が難しい場合は `AppCtx`（広め）+ W010 で警告。過度に精確にしない。
4. **糖衣構文の checker.fav 対応**: `check_bang_notation` が糖衣構文の脱糖前 effects を誤検知しないよう注意。脱糖は parse フェーズで完了しているため問題なし。
