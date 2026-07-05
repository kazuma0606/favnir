# v35.0A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.5.0` であること
- [x] v34.9A が COMPLETE であること
- [x] `grep -rn "\bEffect\b" fav/src/ --include="*.rs" | grep -v "//"` が 0 件であること
- [x] `driver.rs` に `mod v35600_tests` が存在しないこと
- [x] `cargo_toml_version_is_35_5_0` が v35500_tests 内に存在すること（スタブ化対象）
- [x] `grep -rl "!Effect\|!Http\|!Io\|!Db" site/content/ --include="*.mdx" | wc -l` が 125 件であること（変換対象確認）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.5.0` → `35.6.0` に更新
- [x] **T2** `fav/tmp/migrate_mdx.py` — MDX 用移行スクリプトを作成・実行（125 件一括変換）
       対象: `site/content/docs/`（73 件）、`site/content/cookbook/`（48 件）、`site/content/learn/`（2 件）
       変換後: `grep -rl "!Effect\|!Http\|!Io\|!Db\|!Postgres" site/content/ --include="*.mdx"` が 0 件
- [x] **T3** `site/content/docs/ctx-syntax-guide.mdx` — spec の 6 セクション構成で完成版に更新
       必須: 「旧 `!Effect` 構文はコンパイルエラー E0374 になる」を明示
- [x] **T4** `site/content/learn/getting-started.mdx` — 手動確認・ctx 構文への更新（入門向け）
- [x] **T5** `README.md` — `!Effect` 記述を ctx 構文説明に書き換え
- [x] **T6** `MILESTONE.md` — v35.0 Production Ready 宣言を追記
- [x] **T7** `fav/src/driver.rs` — `cargo_toml_version_is_35_5_0` をスタブ化
- [x] **T8** `fav/src/driver.rs` — `v35600_tests`（5 件）を追加（`v35500_tests` 直後に挿入）
- [x] **T9** `CHANGELOG.md` — `[v35.6.0]` セクションを先頭に追記
- [x] **T10** `benchmarks/v35.6.0.json` — 新規作成
- [x] **T11** `versions/current.md` — 最新安定版を v35.6.0 に更新

---

## テスト確認

- [x] **T12** `cargo test --bin fav v35600 2>&1 | tail -8` — 5/5 PASS
- [x] **T13** `cargo test 2>&1 | grep "test result"` — 全件 PASS（0 failures）
- [x] **T14** `cargo clippy --locked -- -D warnings` — PASS

---

## cargo clean（マイルストーン完了時）

- [x] **T15** `cargo clean && cargo build && cargo test 2>&1 | grep "test result"` — クリーンビルドで全件 PASS

---

## 完了処理

- [x] **T16** `benchmarks/v35.6.0.json` の `tests_passed` を実測値で確定
- [x] **T17** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = `"35.6.0"`
- [x] `site/content/` 全 MDX のコードブロックに `!Effect` アノテーションが残存しないこと
- [x] `ctx-syntax-guide.mdx` に 6 セクションが揃っていること
- [x] `README.md` が `ctx: AppCtx` パターンを説明していること
- [x] `MILESTONE.md` に v35.0 Production Ready 宣言があること
- [x] `cargo_toml_version_is_35_5_0` が空スタブになっていること
- [x] `cargo test --bin fav v35600` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `cargo clippy --locked -- -D warnings` — PASS
- [x] `cargo clean` 後のクリーンビルド + 全テスト PASS
- [x] `CHANGELOG.md` に `[v35.6.0]` セクション
- [x] `benchmarks/v35.6.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.6.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] MDX のコードブロック外テキストに残る `!Effect` 言及は「廃止された旧構文」として文脈が明確であること
- [x] `ctx-syntax-guide.mdx` の「旧構文はコンパイルエラー E0374」の説明が正確であること
- [x] `v35600_tests` に `use super::*` が**ない**こと
- [x] cargo clean 後のビルドが通ること（キャッシュ依存のバグがないこと）
- [x] `MILESTONE.md` の宣言文が spec.md の宣言文と整合していること

---

## 完了記録（2026-07-05）

- v35.6.0 として実装完了（tests: 2616 pass, 0 failures）
- MDX 128 ファイル・317 コードブロックの !Effect 除去（両パス）
- ctx-syntax-guide.mdx 6 セクション構成で完成
- MILESTONE.md に v35.0 Production Ready 宣言追記
- cargo clean 後のクリーンビルドで全テスト PASS
- cargo clippy -- -D warnings PASS

### コードレビュー指摘
なし（Clippy clean）
