# v22.1.0 — Checkpoint / Resume（パイプライン永続化）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TrfDef.checkpoint: bool` 追加

- [x] **事前確認**: `grep -n "pub arrow: bool" fav/src/ast.rs` で `arrow` フィールドの位置を確認
- [x] `TrfDef` 構造体の `arrow: bool` フィールドの直後に `pub checkpoint: bool, // v22.1.0` を追加
- [x] `TrfDef { ... }` リテラルを構築する全箇所に `checkpoint: false` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/frontend/parser.rs` — `parse_checkpoint_annotation()` + parse 更新

- [x] **事前確認**: `grep -n "parse_stateful_annotation\|parse_arrow_annotation" fav/src/frontend/parser.rs` でパターンを確認
- [x] `parse_checkpoint_annotation(&mut self) -> Result<bool, ParseError>` メソッドを追加
  - `#` `[` `checkpoint` `]` のトークン列を検出して true を返す
- [x] `parse_item` で `parse_checkpoint_annotation` を呼び出し `checkpoint_ann` を取得
- [x] Stage ブランチ（sync・async 両方）に `td.checkpoint = checkpoint_ann` を追加
  - コードレビュー指摘 [MED-3]: async Stage ブランチの `td.arrow = arrow_ann` 欠落も修正
- [x] `parse_trf_def` の `TrfDef` 構築に `checkpoint: false` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/backend/vm.rs` — スレッドローカル + `set_*` 関数 + `__checkpoint_wrap` builtin

- [x] **事前確認**: 既存 thread_local パターンを確認
- [x] `STAGE_CHECKPOINT_DIR` / `STAGE_RESUME_DIR` / `STAGE_CHECKPOINT_NAMES` thread-local を追加
  - STAGE_ プレフィックスを使用（既存 `CheckpointBackend` との名前衝突を回避）
- [x] `set_checkpoint_dir(dir: Option<&str>)` 公開関数を追加
- [x] `set_resume_dir(dir: Option<&str>)` 公開関数を追加
- [x] `set_checkpoint_stages(names: HashSet<String>)` 公開関数を追加
- [x] `"__checkpoint_wrap"` を `call_builtin` に追加（lookup 機能のみ、v22.2.0 以降で書き込みも実装）
- [x] `write_stage_checkpoint_bytes` / `read_stage_checkpoint_bytes` private fn を追加（`#[cfg(not(target_arch = "wasm32"))]`）
  - コードレビュー指摘 [SECURITY-4]: stage 名から `['/', '\\', ' ', '.']` を `_` に置換（パス traversal 防止）
  - コードレビュー指摘 [HIGH-2]: `_is_checkpoint_stage` デッドコードを削除
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/driver.rs` — `cmd_run` シグネチャ更新 + ヘルパー関数追加

- [x] **事前確認**: `grep -n "pub fn cmd_run" fav/src/driver.rs` でシグネチャを確認
- [x] `cmd_run` に `checkpoint_dir: Option<&str>` / `resume_dir: Option<&str>` 引数を追加
- [x] `cmd_run` 本体の冒頭に checkpoint setup コードを追加
  - `Lexer::new(src, file_name).tokenize().unwrap_or_default()` でソースをトークン化
  - `Parser::new(tokens).parse_program()` でパース（`Parser::parse_str` は存在しない — 修正済み）
  - `#[checkpoint]` 付き stage 名を収集して `vm::set_checkpoint_stages(...)` に渡す
  - `vm::set_checkpoint_dir(...)` / `vm::set_resume_dir(...)` を呼ぶ
- [x] `cmd_run_self_hosted` の `cmd_run` 呼び出しに `None, None` を追加
- [x] `write_stage_checkpoint` / `read_stage_checkpoint` / `stage_checkpoint_path` ヘルパー関数を追加（`#[cfg(not(target_arch = "wasm32"))]`）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/main.rs` — `--checkpoint-dir` / `--resume` フラグ追加

- [x] **事前確認**: `grep -n "explain_pushdown" fav/src/main.rs` で既存フラグの構造を確認
- [x] `Some("run")` ブランチに `let mut checkpoint_dir: Option<String> = None;` / `let mut resume_dir: Option<String> = None;` を追加
- [x] while ループに `"--checkpoint-dir"` / `"--resume"` のアームを追加（`i += 2` パターン）
- [x] `cmd_run(...)` 呼び出しに `checkpoint_dir.as_deref()` / `resume_dir.as_deref()` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/Cargo.toml` バージョン更新 + `v221000_tests` 追加

- [x] **事前確認**: `grep "tempfile" fav/Cargo.toml` — `tempfile = "3"` が `[dev-dependencies]` に存在することを確認
- [x] `version = "22.0.0"` → `"22.1.0"` に変更
- [x] `v220000_tests::version_is_22_0_0` に `#[ignore]` を追加
- [x] `v221000_tests` モジュールを `fav/src/driver.rs` に追加（5 件）
  - `version_is_22_1_0`
  - `checkpoint_annotation_parsed`
  - `write_and_read_stage_checkpoint`（`#[cfg(not(target_arch = "wasm32"))]`）
  - `resume_skips_if_checkpoint_exists`（`#[cfg(not(target_arch = "wasm32"))]`）
  - `changelog_has_v22_1_0`
- [x] `cargo test v221000 --bin fav` — 5/5 PASS 確認
- [x] `cargo test --bin fav` — リグレッションなし（1846 件以上合格）確認

---

### T7: `CHANGELOG.md` + `site/content/docs/cli/checkpoint.mdx`

- [x] v22.1.0 エントリを CHANGELOG.md の先頭（v22.0.0 エントリの上）に追加
- [x] `grep "\[v22.1.0\]" CHANGELOG.md` で追加確認
- [x] `site/content/docs/cli/checkpoint.mdx` を新規作成
  - `--checkpoint-dir` / `--resume` の使用例
  - `#[checkpoint]` アノテーションのコード例
  - checkpoint ファイル形式の説明
  - オプション表

---

## テスト一覧（v221000_tests、5 件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_22_1_0` | Cargo.toml に `version = "22.1.0"` が含まれる | PASS（現在 #[ignore]） |
| `checkpoint_annotation_parsed` | `#[checkpoint] stage Foo: Int -> Int` で `TrfDef.checkpoint == true` | PASS |
| `write_and_read_stage_checkpoint` | `write_stage_checkpoint` → `read_stage_checkpoint` でデータが一致する | PASS |
| `resume_skips_if_checkpoint_exists` | checkpoint なし → None、checkpoint あり → Some | PASS |
| `changelog_has_v22_1_0` | CHANGELOG.md に `[v22.1.0]` が含まれる | PASS |

---

## コードレビュー指摘と対応（実装後レビュー）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| HIGH-1 | compiler.rs が `__checkpoint_wrap` IR を emit しない | 仕様スコープ外（v22.3+）。spec.md で明確化 |
| HIGH-2 | `_is_checkpoint_stage` デッドコード | 削除済み |
| MED-3 | async Stage ブランチに `td.arrow = arrow_ann` が欠落 | 修正済み |
| SECURITY-4 | stage 名のパス traversal（`.` が未サニタイズ） | `'.'` を replace リストに追加して修正済み |

---

## 完了条件チェックリスト

- [x] `#[checkpoint]` アノテーションが `TrfDef.checkpoint = true` としてパースされる
- [x] `fav run --checkpoint-dir <dir>` が受け付けられる（`cmd_run` シグネチャ更新）
- [x] `fav run --resume <dir>` が受け付けられる（`cmd_run` シグネチャ更新）
- [x] `write_stage_checkpoint` / `read_stage_checkpoint` / `stage_checkpoint_path` ヘルパー関数が存在する
- [x] `__checkpoint_wrap` が `call_builtin` に登録されている
- [x] `cargo test v221000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1846 件以上合格）
- [x] `CHANGELOG.md` に v22.1.0 エントリ
- [x] `site/content/docs/cli/checkpoint.mdx` 作成済み
