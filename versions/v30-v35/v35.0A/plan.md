# v35.0A plan

## 実装順序

### フェーズ A — MDX 一括変換（Python スクリプト）

```python
# fenced code block 内の !Effect を除去するスクリプト
# - ```favnir ブロック内のみ変換
# - stage / fn シグネチャの !Effect 除去
# - fn に ctx がなければ追加
```

対象ディレクトリ:
- `site/content/docs/`（73 件）
- `site/content/cookbook/`（48 件）
- `site/content/learn/`（2 件）

スクリプトは `fav/tmp/migrate_mdx.py` に保存して実行する。

### フェーズ B — ctx-syntax-guide.mdx 更新

既存の `site/content/docs/ctx-syntax-guide.mdx` を spec の 6 セクション構成で書き直す。
特に「旧 `!Effect` 構文はコンパイルエラー（E0374）になる」という明示が重要。

### フェーズ C — README.md 更新

`README.md` の以下のセクションを更新:
- "Effect System" または "!Effect" 記述箇所 → ctx パターン説明に書き換え
- Quick Start のコードサンプルが ctx 構文を使っていることを確認

### フェーズ D — MILESTONE.md 更新

v35.0 Production Ready 宣言を追記。

### フェーズ E — driver.rs + テスト

- `cargo_toml_version_is_35_5_0` スタブ化
- `v35600_tests`（5 件）追加

## v35600_tests の内容（5 件）

1. `cargo_toml_version_is_35_6_0` — バージョン確認
2. `ctx_syntax_guide_has_no_effect_annotations` — `ctx-syntax-guide.mdx` のコードブロックに `!Effect` がないこと
3. `getting_started_uses_ctx_syntax` — `learn/getting-started.mdx` に `ctx: AppCtx` が含まれること
4. `readme_mentions_ctx_appctx` — `README.md` に `ctx: AppCtx` が含まれること
5. `milestone_mentions_v35_production_ready` — `MILESTONE.md` に `v35.0` と `Production Ready` が含まれること

## 注意点

- MDX の通常テキスト（コードブロック外）に `!Effect` への言及が説明として残る場合がある。
  これは「廃止された旧構文」として説明する文章なので残してよい（誤解を与えないよう注記を添える）。
- `getting-started.mdx` と `pipeline-basics.mdx` は入門向けのため手動で丁寧に確認・修正する。

## cargo clean

v35.0A は v35.0 マイルストーン宣言のバージョンである。
完了後に以下を実施する:

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings
du -sh target/
echo "=== v35.0 Production Ready クリーンアップ完了 ==="
```
