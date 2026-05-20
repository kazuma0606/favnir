# v5.4.0 Tasks — `rune_modules/` Import Resolution

## Phase A: `toml.rs` helper

- [ ] A-1: `pub fn rune_entry_file(rune_dir: &Path, name: &str) -> PathBuf` を `toml.rs` に追加
- [ ] A-2: テスト: `rune_entry_file` — entry 有り / entry 無し / rune.toml 無し の 3ケース

## Phase B: `resolver.rs` 変更

- [ ] B-1: `resolve_rune_import_file()` で `rune_modules/<name>/` を最優先チェック
  - [ ] `root` あり: `{root}/rune_modules/<name>/` を確認
  - [ ] `root` なし（standalone）: `CWD/rune_modules/<name>/` を確認
  - [ ] `rune_entry_file()` でエントリファイルを解決
  - [ ] fallback: 従来の `runes/` → `~/.fav/registry/` の順を維持
- [ ] B-2: テスト: `test_resolve_rune_from_rune_modules` — `rune_modules/csv/csv.fav` が解決される
- [ ] B-3: テスト: `test_resolve_rune_entry_from_rune_toml` — entry フィールドが使われる
- [ ] B-4: テスト: `test_resolve_rune_fallback_to_runes_dir` — `rune_modules/` になければ `runes/` を使う
- [ ] B-5: `cargo test` 通過

## Phase C: `driver.rs` 変更

- [ ] C-1: `load_all_items()` の `ImportDecl { is_rune: true }` 分岐を変更
  - [ ] `(toml, root)` が Some のとき: `rune_modules/` → `runes/` の順でチェック
  - [ ] `(toml, root)` が None のとき: source file dir の `rune_modules/` をチェック
- [ ] C-2: `has_rune_imports(program: &Program) -> bool` ヘルパーを追加
- [ ] C-3: `load_and_check_program()` で standalone + rune imports のケースを処理
  - [ ] `has_rune_imports` が true のとき `load_all_items` を呼び出す
  - [ ] type-checker の resolver に `rune_modules/` 解決パスを渡す
- [ ] C-4: テスト: standalone スクリプト + `rune_modules/` の組み合わせ
- [ ] C-5: `cargo test` 全件通過（965 件 + 新規テスト）

## Phase D: E2E 確認

- [ ] D-1: `rune install csv` → `./rune_modules/csv/` に展開される（既存動作）
- [ ] D-2: `import csv` を含む `main.fav` を作成
- [ ] D-3: `fav run main.fav`（`fav.toml` あり / project mode）が正常動作する
- [ ] D-4: `fav run main.fav`（`fav.toml` なし / standalone mode）が正常動作する
- [ ] D-5: `fav check main.fav` でエラーなし

## Phase E: 全 15 Rune 公開

- [ ] E-1: `rune publish` を各 Rune ディレクトリで実行
  - [ ] `runes/auth/` — auth@0.1.0
  - [ ] `runes/aws/` — aws@0.1.0
  - [ ] `runes/csv/` — csv@0.1.0（再公開）
  - [ ] `runes/db/` — db@0.1.0
  - [ ] `runes/duckdb/` — duckdb@0.1.0
  - [ ] `runes/env/` — env@0.1.0
  - [ ] `runes/gen/` — gen@0.1.0
  - [ ] `runes/grpc/` — grpc@0.1.0
  - [ ] `runes/http/` — http@0.1.0
  - [ ] `runes/incremental/` — incremental@0.1.0
  - [ ] `runes/json/` — json@0.1.0（再公開）
  - [ ] `runes/log/` — log@0.1.0
  - [ ] `runes/parquet/` — parquet@0.1.0
  - [ ] `runes/stat/` — stat@0.1.0
  - [ ] `runes/validate/` — validate@0.1.0
- [ ] E-2: `rune search` で 15 件が表示されることを確認

## Phase F: サイトドキュメント更新

- [ ] F-1: `site/content/docs/rune-cli.mdx` を作成
  - [ ] install / uninstall / list / info / search / update / publish の説明
  - [ ] インストール手順（symlink / copy）
  - [ ] `rune.toml` フォーマット説明
- [ ] F-2: サイドバーに "Rune CLI" を追加（`site/lib/docs.ts`）
- [ ] F-3: `npm run build` 成功確認
- [ ] F-4: （任意）`site/app/runes/page.tsx` をライブ Registry API 取得に変更

## Phase G: まとめ

- [ ] G-1: `cargo test` 全件通過
- [ ] G-2: `versions/v5.4.0/tasks.md` にチェックを入れる
- [ ] G-3: `MEMORY.md` を更新
- [ ] G-4: `feat: rune_modules import resolution + publish all runes (v5.4.0)` でコミット
