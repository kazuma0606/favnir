# v5.4.0 Tasks — `rune_modules/` Import Resolution

## Phase A: `toml.rs` helper

- [x] A-1: `pub fn rune_entry_file(rune_dir: &Path, name: &str) -> PathBuf` を `toml.rs` に追加
- [x] A-2: テスト: `rune_entry_file` — entry 有り / entry 無し / rune.toml 無し の 3ケース

## Phase B: `resolver.rs` 変更

- [x] B-1: `resolve_rune_import_file()` で `rune_modules/<name>/` を最優先チェック
  - [x] `root` あり: `{root}/rune_modules/<name>/` を確認
  - [x] `root` なし（standalone）: `CWD/rune_modules/<name>/` を確認
  - [x] `rune_entry_file()` でエントリファイルを解決
  - [x] fallback: 従来の `runes/` → `~/.fav/registry/` の順を維持
- [x] B-2: テスト: `test_resolve_rune_from_rune_modules` — `rune_modules/csv/csv.fav` が解決される
- [x] B-3: テスト: `test_resolve_rune_entry_from_rune_toml` — entry フィールドが使われる
- [x] B-4: テスト: `test_resolve_rune_fallback_to_runes_dir` — `rune_modules/` になければ `runes/` を使う
- [x] B-5: `cargo test` 通過

## Phase C: `driver.rs` 変更

- [x] C-1: `load_all_items()` の `ImportDecl { is_rune: true }` 分岐を変更
  - [x] `(toml, root)` が Some のとき: `rune_modules/` → `runes/` の順でチェック
  - [x] `(toml, root)` が None のとき: source file dir の `rune_modules/` をチェック
- [x] C-2: `has_rune_imports(program: &Program) -> bool` ヘルパーを追加
- [x] C-3: `load_and_check_program()` で standalone + rune imports のケースを処理
  - [x] `has_rune_imports` が true のとき `load_all_items` を呼び出す
  - [x] type-checker の resolver に `rune_modules/` 解決パスを渡す
- [x] C-4: テスト: standalone スクリプト + `rune_modules/` の組み合わせ
- [x] C-5: `cargo test` 全件通過（971 件）

## Phase D: E2E 確認

- [x] D-1: `rune install csv` → `./rune_modules/csv/` に展開される（既存動作）
- [x] D-2: `import rune "csv"` を含む `main.fav` を作成
- [x] D-3: `fav run main.fav`（`fav.toml` あり / project mode）が正常動作する
- [x] D-4: `fav run main.fav`（`fav.toml` なし / standalone mode）が正常動作する
- [x] D-5: `fav check main.fav` でエラーなし

## Phase E: 全 15 Rune 公開

- [x] E-1: `rune publish` を各 Rune ディレクトリで実行
  - [x] `runes/auth/` — auth@0.1.0
  - [x] `runes/aws/` — aws@0.1.0
  - [x] `runes/csv/` — csv@0.1.0（再公開）
  - [x] `runes/db/` — db@0.1.0
  - [x] `runes/duckdb/` — duckdb@0.1.0
  - [x] `runes/env/` — env@0.1.0
  - [x] `runes/gen/` — gen@0.1.0
  - [x] `runes/grpc/` — grpc@0.1.0
  - [x] `runes/http/` — http@0.1.0
  - [x] `runes/incremental/` — incremental@0.1.0
  - [x] `runes/json/` — json@0.1.0（再公開）
  - [x] `runes/log/` — log@0.1.0
  - [x] `runes/parquet/` — parquet@0.1.0
  - [x] `runes/stat/` — stat@0.1.0
  - [x] `runes/validate/` — validate@0.1.0
- [x] E-2: `rune search` で 15 件が表示されることを確認

## Phase F: サイトドキュメント更新

- [x] F-1: `site/content/docs/rune-cli.mdx` を作成
  - [x] install / uninstall / list / info / search / update / publish の説明
  - [x] インストール手順（symlink / copy）
  - [x] `rune.toml` フォーマット説明
- [x] F-2: サイドバーに "Rune CLI" を追加（`site/lib/docs.ts`）
- [x] F-3: `npm run build` 成功確認
- [x] F-4: （任意）`site/app/runes/page.tsx` をライブ Registry API 取得に変更

## Phase G: まとめ

- [x] G-1: `cargo test` 全件通過
- [x] G-2: `versions/v5.4.0/tasks.md` にチェックを入れる
- [ ] G-3: `MEMORY.md` を更新
- [x] G-4: `feat: rune_modules import resolution + publish all runes (v5.4.0)` でコミット
