# Roadmap v72.1.0 〜 v73.0.0 — Developer Experience 2.0 宣言

Date: 2026-08-08
Status: 未着手（v72.0.0 完了後に開始）

マスターロードマップ: [roadmap-v70.1-v75.0.md](roadmap-v70.1-v75.0.md)

---

## 前提

- 直前完了: v72.0.0「Type System 2.0」（tests = 3612）
- 本スプリントは Phase 3「Developer Experience 2.0」の詳細計画
- 目標: v73.0.0「Developer Experience 2.0 宣言」（tests = 3634）

### スプリントの性格

Phase 3 は「データエンジニアが Favnir を選ぶ開発体験」のスプリントである。
型システムが強力でも、開発体験が貧しければ選ばれない。
VS Code 拡張・AI アシスタント・REPL・Playground——
実際に手を動かすすべての場面で Favnir が寄り添う開発環境を整える。
C（実証・UX）80% + B（AI コード生成等）20% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v72.1.0 | VS Code 拡張（本格実装） | 3612 + 2 = 3614 | 完了（実測 3614） |
| v72.2.0 | AI エラーアシスタント（`fav ai explain` / `fav ai fix`） | 3614 + 2 = 3616 | 完了（実測 3616） |
| v72.3.0 | `fav ai generate`（自然言語 → Favnir パイプライン） | 3616 + 4 = 3620 | 完了（実測 3620、code-reviewer +2） |
| v72.4.0 | REPL 2.0 | 3620 + 5 = 3625 | 完了（実測 3625、code-reviewer +3） |
| v72.5.0 | Playground 2.0 | 3625 + 2 = 3627 | 完了（実測 3630、code-reviewer 対応で +3 追加） |
| v72.6.0 | `fav init` テンプレートギャラリー拡充 | 3630 + 2 = 3632 | 完了（実測 3635、code-reviewer 対応で +3 追加） |
| v72.7.0 | Hot Reload 改善（`fav watch` 2.0） | 3635 + 2 = 3637 | 完了（実測 3638、code-reviewer 対応で +1 追加） |
| v72.8.0 | インタラクティブチュートリアル（`fav learn`） | 3638 + 2 = 3640 | 完了（実測 3640） |
| v72.9.0 | 安定化・コードフリーズ（Developer Experience 2.0 前調整） | 3640 + 2 = 3642 | 完了（実測 3642） |
| v73.0.0 | Developer Experience 2.0 宣言 ★クリーンアップ | 3642 + 4 = 3646 | 完了（実測 3646） |

---

## v72.1.0 — VS Code 拡張（本格実装）

既存 LSP を VS Code Extension として完全統合する。
マーケットプレイス公開を視野に入れた品質で実装する。

```
機能一覧:
✓ シンタックスハイライト（.fav ファイル）
✓ 型ホバー（変数・関数にカーソルを当てると型を表示）
✓ 定義ジャンプ（F12）・参照検索（Shift+F12）
✓ インライン型ヒント（引数名・戻り値型）
✓ エラーアンダーライン + 修正ヒント（Quick Fix）
✓ Rune メソッド補完（ctx.io. → argv / println / read_file_raw ...）
✓ コードフォーマット（保存時 fav fmt 自動実行）
✓ fav run / fav check をエディタから実行（Run Task）
```

**実装内容:**
- `editors/vscode/` ディレクトリ — `package.json` / `extension.ts` / `syntaxes/favnir.tmGrammar.json`
- LSP クライアントとして既存 `fav lsp` サーバーに接続
- VS Code Marketplace 公開設定（`publisher` / `categories`）

**完了条件**: Rust テスト 2 件（3612 + 2 = 3614）
- `vscode_extension_package_json_valid`
- `vscode_extension_lsp_integration`

---

## v72.2.0 — AI エラーアシスタント（`fav ai explain` / `fav ai fix`）

コンパイルエラーを AI に渡し、自然言語での説明と修正案を得る。

```bash
$ fav check pipeline.fav --ai-explain
E0374 detected at line 43.

[AI Explanation]
このエラーは `!IO` というエフェクトアノテーション構文が使われているために
発生しています。v35.4.0 でこの構文は廃止され、代わりに `ctx: AppCtx` を
関数の第1引数として渡す方式に変わりました。

[Suggested Fix]
Before: fn write_results_md(data: JsonValue) -> Result<Unit, String> !IO
After:  fn write_results_md(ctx: AppCtx, data: JsonValue) -> Result<Unit, String>

さらに `IO.write_file(...)` → `ctx.io.write_file_raw(...)` への変更も必要です。

Apply this fix? [y/N]: y
✓ Applied. Run `fav check pipeline.fav` to verify.

# または自動修正のみ
$ fav ai fix pipeline.fav
```

**実装内容:**
- `cmd_ai_explain(path, error_code)` — エラーコード + ソースを Claude API に送信
- `cmd_ai_fix(path)` — 提案修正をファイルに適用（diff プレビュー付き）
- `fav check --ai-explain` フラグとの統合

**完了条件**: Rust テスト 2 件（3614 + 2 = 3616）
- `ai_explain_e0374_returns_hint`
- `ai_fix_applies_ctx_migration`

---

## v72.3.0 — `fav ai generate`（自然言語 → Favnir パイプライン）

自然言語の要求仕様から Favnir パイプラインの雛形を生成する。

```bash
$ fav ai generate "S3のCSVを読んでスキーマ検証しPostgresに挿入するパイプライン"
Generating pipeline...

# Generated: pipeline.fav
import rune "csv"
import rune "postgres"

schema OrderRow {
    order_id: String
    amount:   Float
    status:   String
}

fn main(ctx: AppCtx) -> Result<Unit, String> {
    bind raw   <- ctx.io.read_file_raw("s3://bucket/data.csv")
    bind rows  <- Csv.parse_typed(raw, OrderRow)
    bind valid <- Schema.validate_all(rows)
    bind _     <- Postgres.execute_raw("INSERT INTO orders ...", valid)
    ctx.io.println("Done.")
}

```

※ エディタ起動・`fav check` 自動検証は v72.4.0 以降に実装。

**実装内容:**
- `cmd_ai_generate(description)` — 自然言語 → Favnir コード生成（キーワードベースのテンプレート生成）
- スキーマ推論（説明文からフィールド名・型を推定）
- `fav check` 自動検証は v72.4.0 以降

**完了条件**: Rust テスト 2 件（3616 + 2 = 3618）
- `ai_generate_returns_valid_fav_code`
- `ai_generate_schema_inferred_from_description`

---

## v72.4.0 — REPL 2.0

既存 `fav repl` を大幅強化する。

```bash
$ fav repl
Favnir v72.4.0 REPL — :help でヘルプ

fav> :import rune "json"
rune "json" loaded.

fav> bind data <- Json.parse("[1,2,3]")
data: JsonValue = [1, 2, 3]

fav> List.length(data)
3: Int

fav> :timing on
fav> List.map([1..100], |x| x * x)
[1, 4, 9, ...] : List<Int>  (0.3ms)

fav> :history          # 入力履歴表示
fav> :save session.fav # セッションをファイルに保存
fav> :load session.fav # セッションを再現
```

**新機能（v72.4.0 実装範囲）:**
- `:timing on/off` モード（式評価時間を ms 表示）
- TAB 補完ヘルパー（`repl_tab_complete` — rustyline 統合は v72.5.0 以降）
- `needs_continuation` pub 化（マルチライン継続は v60.5.0 で実装済み）

**実装内容:**
- `repl_tab_complete(prefix, scope)` ヘルパー関数を `driver.rs` に追加
- `ReplSession.timing_enabled` フィールド + `:timing on/off` ハンドラ追加
- `needs_continuation` を `pub fn` に変更

※ `rustyline` 統合・`~/.fav_history` 永続化・Rune メソッド補完は依存クレート追加コストを考慮し v72.5.0 以降に延期。

**完了条件**: Rust テスト 2 件（3620 + 2 = 3622）
- `repl2_multiline_input`
- `repl2_tab_completion`

---

## v72.5.0 — Playground 2.0

ブラウザ内の Favnir Playground を全面強化する。

```
新機能:
- AI 補完（GitHub Copilot 風のインライン提案）
- 共有リンク（実行結果 + コード を永続 URL で共有）
- テンプレートギャラリー（AI ETL / 分散 / データ品質 / 時系列）
- 実行結果の可視化（List<Record> → テーブル表示、List<Float> → グラフ）
- WASM ビルド対応（ブラウザ内で完全実行）
```

**実装内容:**
- `site/content/playground/` の強化（Monaco エディタ統合）
- 共有リンク生成（`/playground?code=<base64>` 形式）
- テンプレートギャラリー（最低 5 エントリ）

**完了条件**: Rust テスト 2 件（3625 + 2 = 3627）
- `playground2_template_gallery_has_5_entries`
- `playground2_share_url_format`

---

## v72.6.0 — `fav init` テンプレートギャラリー拡充

> `fav init` コマンドおよび `cmd_new` は driver.rs / main.rs に既に存在する。
> 本バージョンでは v73 以降の新機能（AI ETL / distributed / data-quality）に
> 対応したテンプレートを追加する（コマンド自体の新規実装ではなく拡充）。

```bash
$ fav init --template ai-etl          # LLM 抽出 → VectorDB
$ fav init --template streaming       # Kafka + ML スコアリング
$ fav init --template enterprise      # マルチテナント + 監査ログ
$ fav init --template data-quality    # データ品質検証パイプライン
$ fav init --template distributed     # マルチノード par
```

各テンプレートに `README.md`・動作確認コマンド・`fav.toml` を同梱。

**実装内容:**
- `TEMPLATE_GALLERY` に 5 テンプレートを追加（driver.rs）
- 各テンプレートの `create_<name>_project` 関数を実装
- `try_cmd_new` に新テンプレートのアームを追加

**完了条件**: Rust テスト 2 件（3630 + 2 = 3632）
- `init_template_ai_etl_valid`
- `init_template_data_quality_valid`

---

## v72.7.0 — Hot Reload 改善（`fav watch` 2.0）

ファイル変更を検知して任意コマンドを実行できるようにする（`--on-change` フラグ追加）。

```bash
$ fav watch pipeline.fav --on-change "fav check && fav run --dry-run"
Watching pipeline.fav... (Ctrl+C to stop)
[watch] Running: fav check && fav run --dry-run
[10:32:01] Change detected: pipeline.fav
[watch] Running: fav check && fav run --dry-run
[10:32:01] Ready.
```

**実装内容:**
- `WatchSession` 構造体（`file` / `on_change_cmd` / `debounce_ms` フィールド）
- `watch_session_on_change_label` — 変更時のコンソールラベル生成（fs 非依存）
- `cmd_watch2(file, on_change, debounce_ms)` — `--on-change` 対応の新ウォッチ関数
- `main.rs` に `--on-change` フラグ解析を追加
- 500ms デバウンス（既存の `notify` 統合を `cmd_watch2` でも継承）
- ※ 差分ステージ検出（変更ステージの上流のみ再実行）は複雑度が高く v73.x 以降に延期

**完了条件**: Rust テスト 2 件（3635 + 2 = 3637）
- `watch2_session_field_defaults`
- `watch2_runs_custom_command`

---

## v72.8.0 — インタラクティブチュートリアル（`fav learn`）

```bash
$ fav learn
Favnir インタラクティブチュートリアル v1.0

Chapter 1: 最初のパイプライン
[1/5] fn main(ctx: AppCtx) -> Result<Unit, String> を書いてみましょう
>>> _
（正解するまでヒントを出しながら次へ進む）

Chapter 2: 型システムの力
Chapter 3: Rune を使ったデータ処理
Chapter 4: AI パイプライン
Chapter 5: 分散実行
```

**実装内容:**
- `LearnChapter` 構造体 + `LEARN_CHAPTERS` 静的データ（5 章分）
- `cmd_learn` in driver.rs — チュートリアル Chapter 1〜5 のコンテンツ定義と対話ループ
- 対話的な入力検証（正解・ヒント・次のステップへの誘導）
- ※ 進捗保存（`~/.fav_learn_progress`）は複雑度のため v73.x 以降に延期

**完了条件**: Rust テスト 2 件（3638 + 2 = 3640）
- `learn_chapter1_exists`
- `learn_chapter5_exists`

---

## v72.9.0 — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

v72.1〜v72.8 の全機能が正常動作することを確認する安定化バージョン。
VS Code 拡張・REPL 2.0・Playground 2.0 の E2E テストを実施する。

**完了条件**: Rust テスト 2 件（3640 + 2 = 3642）
- `dev_exp2_all_stable`
- `vscode_repl2_playground2_e2e`

---

## v73.0.0 — Developer Experience 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「VS Code がパイプラインを補完し、AI がエラーを修正し、
>  REPL が型を即座に返し、Playground がコードを世界と共有する。
>  自然言語一文が、型安全なパイプラインの雛形になる。
>
>  これが Favnir v73.0 — Developer Experience 2.0 の姿である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `73.0.0` に更新
- `CHANGELOG.md` に v73.0.0 エントリを追加
- `MILESTONE.md` に「Developer Experience 2.0」を追記
- `README.md` に v73.0 達成を追記
- `versions/current.md` を更新（進行中 → v73.1.0）

**完了条件**: `v73000_tests` 4 件（3642 + 4 = 3646）
- `cargo_toml_version_is_73_0_0`
- `changelog_has_v73_0_0`
- `milestone_has_dev_exp2`
- `readme_mentions_dev_exp2`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v72.0.0（ベース） | 3,612 | — | |
| v72.1.0 | 3,614 | +2 | 実測 |
| v72.2.0 | 3,616 | +2 | 実測 |
| v72.3.0 | 3,620 | +4 | 実測（code-reviewer 対応で +2 追加） |
| v72.4.0 | 3,625 | +5 | 実測（code-reviewer 対応で +3 追加） |
| v72.5.0 | 3,630 | +5 | 実測（code-reviewer 対応で +3 追加） |
| v72.6.0 | 3,635 | +5 | 実測（code-reviewer 対応で +3 追加） |
| v72.7.0 | 3,638 | +3 | 実測（code-reviewer 対応で +1 追加） |
| v72.8.0 | 3,640 | +2 | |
| v72.9.0 | 3,642 | +2 | |
| v73.0.0（宣言） | 3,646 | +4 | |

**本スプリント合計**: +34 tests（3,612 → 3,646）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v70.1-v75.0.md`
- 前スプリント（完了予定）: `versions/roadmap/roadmap-v71.1-v72.0.md`
- 次スプリント: `versions/roadmap/roadmap-v73.1-v74.0.md`
- 進行状況: `versions/current.md`
