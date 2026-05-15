# Favnir v2.0.0 タスクリスト

作成日: 2026-05-09

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.0.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.0.0` に更新
- [x] `src/main.rs`: `migrate` コマンドを HELP テキストに追加
- [x] `src/backend/artifact.rs`: FVC_VERSION を `0x20` に更新

---

## Phase 1 — 旧キーワード削除

### パーサー変更（`src/frontend/parser.rs`）

- [x] `parse_item`: `TokenKind::Trf` を E2001 エラーに置き換える
- [x] `parse_item`: `TokenKind::Flw` を E2002 エラーに置き換える
- [x] `parse_item`: `TokenKind::Cap` を E2003 エラーに置き換える（`parse_cap_def` 呼び出し削除）
- [x] `parse_abstract_item`: `TokenKind::Trf` を E2001 エラーに置き換える
- [x] `parse_abstract_item`: `TokenKind::Flw` を E2002 エラーに置き換える
- [x] `parse_trf_def`: `expect_any([Trf, Stage])` → `expect(&Stage)` に変更
- [x] `parse_abstract_trf_def`: 同上
- [x] `parse_flw_def`: `expect_any([Flw, Seq])` → `expect(&Seq)` に変更
- [x] `parse_abstract_flw_def`: 同上
- [x] `parse_flw_def_or_binding`: 同上
- [x] エラーメッセージに `fav migrate` 案内文を含める

### テスト・example 更新

- [x] `src/middle/checker.rs` の全 inline テスト: `trf`/`flw` → `stage`/`seq`
- [x] `src/middle/checker.rs` の全 inline テスト: `cap` → `interface`
- [x] `src/frontend/parser.rs` の全 inline テスト: `trf`/`flw` → `stage`/`seq`
- [x] `src/backend/vm_stdlib_tests.rs`: 該当テストを更新
- [x] `src/driver.rs` の driver tests: 該当テストを更新
- [x] `examples/abstract_flw_basic.fav`: `abstract flw` → `abstract seq`
- [x] `examples/abstract_flw_inject.fav`: `abstract flw` → `abstract seq`
- [x] `examples/dynamic_inject.fav`: `abstract flw` / `flw` → `abstract seq` / `seq`
- [x] `examples/pipeline.fav`: `trf` / `flw` → `stage` / `seq`
- [x] `examples/` 内の残り全 `.fav` ファイルを `grep` で確認し更新

### 新規テスト追加（`src/frontend/parser.rs`）

- [x] `trf_keyword_removed_e2001`: `trf F: Int -> Int = |x| x` がパースエラー
- [x] `flw_keyword_removed_e2002`: `flw P = F` がパースエラー
- [x] `cap_keyword_removed_e2003`: `cap Eq<T> = { ... }` がパースエラー
- [x] `abstract_trf_removed_e2001`: `abstract trf F: Int -> Int` がパースエラー
- [x] `abstract_flw_removed_e2002`: `abstract flw P { ... }` がパースエラー
- [x] `stage_still_works`: `stage` がパースと型検査を通る
- [x] `seq_still_works`: `seq` がパースと型検査を通る
- [x] `abstract_stage_still_works`: `abstract stage` が動く
- [x] `abstract_seq_still_works`: `abstract seq` が動く

---

## Phase 2 — `abstract stage` / `abstract seq` 確認

- [x] Phase 1 完了後に `fav check examples/abstract_flw_basic.fav` が通ることを確認
- [x] Phase 1 完了後に `fav check examples/abstract_flw_inject.fav` が通ることを確認
- [x] `cargo test` 全テスト通過を確認

---

## Phase 3 — `fav migrate` コマンド

### ドライバー（`src/driver.rs`）

- [x] `migrate_source(source: &str) -> String` 関数を実装
- [x] `migrate_line(line: &str) -> String` 関数を実装
  - [x] `abstract trf ` → `abstract stage ` の変換
  - [x] `abstract flw ` → `abstract seq ` の変換
  - [x] `public trf ` / `private trf ` → `public stage ` / `private stage ` の変換
  - [x] `public flw ` / `private flw ` → `public seq ` / `private seq ` の変換
  - [x] 行頭の `trf ` → `stage ` の変換（インデント考慮）
  - [x] 行頭の `flw ` → `seq ` の変換
  - [x] `cap` 定義行に `// TODO(fav-migrate):` コメントを挿入
  - [x] コメント行（`//` 始まり）はスキップ
- [x] `show_migration_diff(path, original, migrated)` 関数を実装
- [x] `cmd_migrate(path, in_place, dry_run, check_mode)` 関数を実装
- [x] `cmd_migrate_dir(dir, in_place, dry_run, check_mode)` 関数を実装

### CLI（`src/main.rs`）

- [x] `"migrate"` サブコマンドの解析を追加
  - [x] `--in-place` フラグ
  - [x] `--dry-run` フラグ
  - [x] `--check` フラグ
  - [x] `--dir <path>` オプション
  - [x] 位置引数（ファイルパス）
- [x] `migrate` を `driver::cmd_migrate` / `cmd_migrate_dir` に接続

### テスト（`src/driver.rs` の driver tests）

- [x] `migrate_trf_to_stage`: `trf F: ...` → `stage F: ...`
- [x] `migrate_flw_to_seq`: `flw P = F` → `seq P = F`
- [x] `migrate_abstract_trf`: `abstract trf` → `abstract stage`
- [x] `migrate_public_trf`: `public trf` → `public stage`
- [x] `migrate_no_false_positive_in_idents`: `trf_count` が変換されない
- [x] `migrate_comment_lines_skipped`: `// trf example` が変換されない
- [x] `migrate_cap_gets_todo_comment`: `cap Eq<T>` 行に TODO が挿入される
- [x] `migrate_already_v2_no_change`: `stage`/`seq` のみのファイルが変更されない

---

## Phase 4 — セルフホスト・マイルストーン

### 標準ライブラリ確認・追加（`src/backend/vm.rs`）

- [x] `String.length` が実装済みか確認（未実装なら追加）
- [x] `String.char_at` が実装済みか確認
- [x] `String.slice(s, start, end)` が実装済みか確認
- [x] 不足する関数を `vm_call_builtin` に追加
- [x] 追加した関数を `compiler.rs` のビルトイン登録に追加

### Favnir 製レキサー（`examples/selfhost/`）

- [x] `examples/selfhost/` ディレクトリを作成
- [x] `examples/selfhost/lexer.fav` を実装
  - [x] `type Token = IntLit(Int) | Ident(String) | Plus | Minus | Star | Slash | ...` 型定義
  - [x] `type LexResult = { token: Token  rest: String }` 型定義
  - [x] `skip_spaces: String -> String` stage を実装
  - [x] `next_token: String -> LexResult` stage を実装（`+`, `-`, `*`, `/`, EOF）
  - [x] `public fn main()` でデモ動作を確認
  - [x] `fav run examples/selfhost/lexer.fav` が動くことを確認

### Favnir 製レキサーのテスト（`examples/selfhost/lexer.test.fav`）

- [x] `examples/selfhost/lexer.test.fav` を実装
  - [x] `test "next_token: Plus"` ‐ `+` → `Plus` トークン
  - [x] `test "next_token: Minus"` — `-` → `Minus` トークン
  - [x] `test "next_token: Eof on empty"` — 空文字 → `Eof`
  - [x] `test "skip_spaces: strips leading spaces"` — `" abc"` → `"abc"`
  - [x] `test "next_token: ignores leading spaces"` — `" +"` → `Plus`
- [x] `fav test examples/selfhost/lexer.test.fav` で全テスト通過を確認

---

## Phase 5 — テスト・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（テスト数が v1.9.0 の 523 以上）
- [x] `fav check examples/stage_seq_demo.fav` が通ることを確認
- [x] `fav check examples/coalesce_demo.fav` が通ることを確認
- [x] `fav migrate --dry-run examples/pipeline.fav` の出力が正しいことを確認

### ドキュメント作成

- [x] `versions/v2.0.0/langspec.md` を作成
  - [x] v2.0.0 の全キーワード一覧（`stage`/`seq`/`interface` のみ記載）
  - [x] 削除されたキーワードの一覧と移行方法
  - [x] `fav migrate` の使い方
  - [x] セルフホスト・マイルストーンの説明
  - [x] エラーコード一覧（E2001-E2003 追加）
- [x] `versions/v2.0.0/migration-guide.md` を作成
  - [x] v1.x → v2.0.0 移行チェックリスト
  - [x] `trf` → `stage` 移行方法（`fav migrate` で自動）
  - [x] `flw` → `seq` 移行方法（`fav migrate` で自動）
  - [x] `cap` → `interface` 移行方法（手動、サンプル付き）
  - [x] `.fvc` artifact の再コンパイル手順
  - [x] よくある移行エラーと対処法
- [x] `RELEASE_NOTES.md` に v2.0.0 セクションを追加

---

## 完了条件チェック

- [x] `trf F: Int -> Int = |x| x` がコンパイルエラー（E2001 相当のメッセージ）
- [x] `flw P = F` がコンパイルエラー（E2002 相当のメッセージ）
- [x] `cap Eq<T> = { ... }` がコンパイルエラー（E2003 相当のメッセージ）
- [x] `abstract trf F: Int -> Int` がコンパイルエラー
- [x] `stage`/`seq`/`interface` で書かれたコードが正常動作する
- [x] `fav migrate` コマンドが動く（`--dry-run`, `--in-place` 両対応）
- [x] `examples/selfhost/lexer.fav` が `fav run` で動く
- [x] `examples/selfhost/lexer.test.fav` の全テストが `fav test` で通る
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.0.0"`
- [x] `versions/v2.0.0/langspec.md` 作成済み
- [x] `versions/v2.0.0/migration-guide.md` 作成済み
