# Roadmap v60.1.0 〜 v65.0.0 — Performance & DX 2.0

Date: 2026-07-30
Status: 計画中（v60.0 完了後に開始）

---

## 前提

- 直前完了: v60.0.0「Enterprise 1.0」（tests = 3330、2026-07-30 COMPLETE）
- 本文書は v60.1〜v65.0 の**マスターロードマップ**
- 各マイルストーン開始時に対応するサブスプリントロードマップを作成する
- **着手前に実施**: `versions/current.md` の「現行マスターロードマップ」欄を
  `roadmap/roadmap-v60.1-v65.0.md` へ更新する

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v60.1-v61.0.md` | v60.1〜v60.9 + v61.0 | 作成済み（未着手） |
| `roadmap-v61.1-v62.0.md` | v61.1〜v61.9 + v62.0 | 作成済み（未着手） |
| `roadmap-v62.1-v63.0.md` | v62.1〜v62.9 + v63.0 | 作成済み（未着手） |
| `roadmap-v63.1-v64.0.md` | v63.1〜v63.9 + v64.0 | 作成済み（未着手） |
| `roadmap-v64.1-v65.0.md` | v64.1〜v64.9 + v65.0 | 作成済み（未着手） |

---

## 目標

v60.0「Enterprise 1.0」で「企業で安心して使われる言語」を宣言した。
このフェーズは **「選ばれ続ける言語」** を実現する。

2 つの柱を段階的に積み上げ、v65.0「Performance 1.0」として宣言する：

1. **Developer Experience 2.0** — エラーメッセージの品質・LSP の深化・REPL・`fav fmt`・
   `fav doc` の統合成熟。データエンジニアが毎日使いたいと思う体験品質を実現する。
2. **Performance & Scale** — AOT ネイティブコンパイル（cranelift object 出力）・
   差分コンパイルキャッシュ・大規模パイプライン最適化・ベンチマーク体系の確立。

### 既存機能との位置づけ

以下は v60.0 時点で実装済みであり、本ロードマップでは「追加」ではなく
「統合・拡張・本番品質化」として扱う：

| 機能 | 既存状態 | 本ロードマップでの方針 |
|---|---|---|
| `error_catalog.rs` suggestion フィールド | E0001〜E0426 に部分的に記載 | 全コードへの span 表示統合（位置・アンダーライン） |
| `fav check --json` | v12.5 実装済み | LSP Diagnostic との完全統合 |
| LSP hover / completion / inlay_hints | v9.11〜v32.9 で実装済み | Code Action（Quick Fix / Rename）追加 |
| `fav repl` | v9.10 実装済み | `:load` / `:debug` / マルチライン入力拡張 |
| `fav fmt` | v9.2 実装済み（compiler.fav 経由） | コメント保持・行長制限・設定ファイル対応 |
| `fav doc` | v9.8 実装済み | HTML 出力・Rune ドキュメント統合 |
| cranelift-object | Cargo.toml に依存済み（v23 〜） | `fav build` コマンドで native binary 出力 |
| `par [A, B]` Tokio 並列 | v52.0 で Tokio 並列化完了 | 動的スレッドプール・バックプレッシャー制御 |
| `fav profile` flamegraph | v9.9 実装済み | メモリプロファイリング・AOT ベンチ統合 |
| `fav watch` | v9.9 実装済み | 差分コンパイルキャッシュと統合 |

---

## バージョン計画

---

## v61.0 — Developer Experience 2.0（v60.1〜v60.9）

### v60.1.0 — エラーメッセージ span 表示（ソース位置・アンダーライン）

```
// 改善前
E0001: undefined variable: user_id

// 改善後
E0001: undefined variable: `user_id`
 --> pipeline.fav:12:15
   |
12 |   transform(user_id, name)
   |             ^^^^^^^ この変数は定義されていません
   |
   help: もしかして `userId` のことですか？
```

`error_catalog.rs` の `DiagEntry` に `span: Option<Span>` フィールドを追加。
checker.rs / parser.rs からエラー生成時にソース位置を付与するよう更新。
CLI 出力に `-->` / `|` / `^` のアンダーライン形式を実装（`main.rs` の print_diag 関数）。

**完了条件**: Rust テスト 2 件（`error_span_display_e0001` / `error_span_underline_format`）

**実績**: — （未実施）

---

### v60.2.0 — `fav check --fix` 自動修正 Phase 1（typo 修正・未使用 bind 削除）

```bash
$ fav check --fix pipeline.fav
[auto-fixed] E0001: `user_id` → `userId` (pipeline.fav:12)
[auto-fixed] W001: unused bind `tmp` removed (pipeline.fav:8)
2 fixes applied.
```

`fav check --fix` フラグを `main.rs` に追加。
E0001（typo 候補が 1 件のみの場合）と W001（未使用 bind）の自動修正を実装。
`--fix --dry-run` で変更箇所のプレビューのみ表示する。

**完了条件**: Rust テスト 2 件（`check_fix_typo_single_candidate` / `check_fix_unused_bind`）

**実績**: — （未実施）

---

### v60.3.0 — LSP Code Action（Quick Fix / Rename Symbol）

```json
// LSP textDocument/codeAction レスポンス例
{
  "title": "Did you mean `userId`?",
  "kind": "quickfix",
  "edit": { ... }
}
```

LSP の `textDocument/codeAction` ハンドラを `lsp/` に追加。
E0001 typo 修正・W001 未使用 bind 削除を Quick Fix として提供。
`textDocument/rename` ハンドラを `lsp/references.rs` に追加（変数・関数のリネーム）。

**完了条件**: Rust テスト 2 件（`lsp_code_action_e0001_quickfix` / `lsp_rename_variable`）

**実績**: — （未実施）

---

### v60.4.0 — LSP Diagnostic 完全統合（全エラーコードの位置情報付与）

E0001〜E0426 の全エラーコードに span 情報を付与する（v60.1 の続き）。
LSP `textDocument/publishDiagnostics` で位置情報付き diagnostic を送出。
`fav check --json` の出力に `"span": {"file": ..., "line": ..., "col": ...}` フィールドを追加。

**完了条件**: Rust テスト 2 件（`lsp_diagnostic_has_span` / `check_json_includes_span`）

**実績**: — （未実施）

---

### v60.5.0 — `fav repl` 強化（`:load` / `:debug` / マルチライン入力）

```
favnir> :load pipeline.fav
loaded: pipeline.fav (3 stages)
favnir> :debug LoadCsv
[debug] stage LoadCsv: input=(), output=List<Row>
favnir> bind x <-
      |   42 +
      |   58
x : Int = 100
```

`:load <file>` でパイプライン定義を REPL に読み込み、stage を対話的に実行できるように。
`:debug <stage>` でステージの入出力型を表示。
`\` または未閉じカッコでマルチライン入力を継続できるよう lexer を拡張。

**完了条件**: Rust テスト 2 件（`repl_load_pipeline_file` / `repl_multiline_input`）

**実績**: — （未実施）

---

### v60.6.0 — `fav explain-error` 全コード対応 + ドキュメントリンク

```bash
$ fav explain-error E0001
E0001: undefined variable

  変数が未定義です。スペルミスや、スコープ外の変数を参照していないか確認してください。

  よくある原因:
  - bind 前に変数を参照した
  - スペルミス（`user_Id` vs `user_id`）

  ドキュメント: https://favnir.dev/docs/errors/E0001
```

`error_catalog.rs` の全エントリに `long_description` フィールドを追加（Markdown 形式）。
`fav explain-error <CODE>` でターミナル表示。
ドキュメントサイト用の `site/content/docs/errors/` MDX を自動生成する
`fav generate-error-docs` コマンドを追加。

**完了条件**: Rust テスト 2 件（`explain_error_all_codes_have_long_desc` / `cmd_generate_error_docs`）

**実績**: — （未実施）

---

### v60.7.0 — `fav fmt` ルール拡張（コメント保持・行長制限・`.favfmt` 設定）

```toml
# .favfmt — フォーマット設定ファイル
max_line_length = 100
indent_width = 2
preserve_comments = true
trailing_comma = "always"
```

`fav fmt` がコメントを正しく保持するよう `fmt.rs` を修正。
行長制限（デフォルト 100）を超える式を自動折り返し。
プロジェクトルートの `.favfmt` ファイルを読み込んでフォーマット設定を適用。

**完了条件**: Rust テスト 2 件（`fmt_preserves_comments` / `fmt_respects_favfmt_config`）

**実績**: — （未実施）

---

### v60.8.0 — `fav doc` 強化（HTML 出力・Rune ドキュメント統合）

```bash
$ fav doc --format html --out docs/
Generated: docs/index.html
Generated: docs/pipeline/LoadCsv.html
Generated: docs/runes/postgres.html
```

`fav doc --format html` で静的 HTML を生成する出力バックエンドを追加。
`runes/*/rune.toml` の `description` フィールドを読み込み Rune のドキュメントページを生成。
`/// @param` / `/// @returns` タグのパーサーを追加し型情報と統合表示。

**完了条件**: Rust テスト 2 件（`doc_html_output_generated` / `doc_rune_description_included`）

**実績**: — （未実施）

---

### v60.9.0 — 安定化・DX チェックリスト

v60.1〜v60.8 の全機能が統合され、以下が動作することを確認する：
- `fav check --fix` → `fav check --json` → LSP diagnostic の出力が一貫している
- `.favfmt` 設定が `fav fmt` / `fav check` の両方で読まれる
- REPL で `:load` → stage 実行 → `:debug` の E2E フロー

**完了条件**: Rust テスト 2 件（`dx_e2e_check_fix_lsp_consistent` / `dx_repl_pipeline_e2e`）

**実績**: — （未実施）

---

### v61.0 — DX 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「エラーはソース位置を指し、修正候補は即座に現れる。
>  エディタは意図を理解し、フォーマッタはコメントを守る。
>  REPL でパイプラインを対話的に探索でき、ドキュメントは自動生成される。
>
>  Favnir のエラーメッセージはデータエンジニアの道標になった。
>
>  これが Favnir v61.0 — Developer Experience 2.0 の姿である。」

**完了条件**:
- v60.1〜v60.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3352**）
- `v61000_tests` 4 件 pass（ベース 3348 + 4 = 3352 tests passed, 0 failed）:
  - `cargo_toml_version_is_61_0_0`
  - `changelog_has_v61_0_0`
  - `milestone_has_dx2`
  - `readme_mentions_dx2`
- `MILESTONE.md` に `"Developer Experience 2.0"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## v62.0 — Language Polish（v61.1〜v61.9）

### v61.1.0 — OR パターン強化（ネスト・型チェック・lint 統合）

```favnir
match status {
  "active" | "pending" => process(row)
  "deleted" | "archived" => skip(row)
  _ => error("unknown status")
}
```

`Pattern::Or(Vec<Pattern>, Span)` は v17.2.0 時点で実装済み（`ast.rs` L297-298）。
AST 変更なし。本バージョンでは以下を拡張する：
- OR パターン内の各アームが同一型を持つことを checker.rs で検証（型不一致時 E0009 を発行）
- lint.rs の W037（到達不能パターン）を OR パターンに対応拡張
- E2E テストで 3 段階ネスト OR パターンが正しく動作することを確認

**完了条件**: Rust テスト 2 件（`pattern_or_type_check_arms_same` / `pattern_or_lint_w037_integration`）

**実績**: — （未実施）

---

### v61.2.0 — as-pattern 拡張（ネストパターン・LSP hover 統合）

```favnir
match value {
  Some({ id, name }) as opt => { log(opt); transform(id, name) }
  None => default()
}
```

`Pattern::As(String, Box<Pattern>, Span)` は v56.6.0 で実装済み（`ast.rs` L305-307）。
AST 変更なし。本バージョンでは以下を拡張する：
- as-pattern と Record パターンのネストを checker.rs で正しく型チェック（内部フィールドと全体の型を同時に束縛）
- LSP hover で as-pattern の束縛変数 `opt` の型を表示（`inlay_hints.rs` 拡張）
- W038 lint（ワイルドカードインポートによる名前衝突）と as-name の衝突チェック

**完了条件**: Rust テスト 2 件（`pattern_as_nested_record` / `pattern_as_lsp_hover_type`）

**実績**: — （未実施）

---

### v61.3.0 — パターンガード拡張（OR パターン各アームへの個別ガード）

```favnir
// 新機能: OR パターンの各アームに独立したガードを付与
match row {
  ("active" if score > 90) | ("pending" if score > 50) => process(row)
  _ => skip(row)
}
```

`MatchArm.guard` は単一ガードとして実装済み。複合条件（`&&` / `||`）は `Expr::BinOp` で
既に表現可能。本バージョンでは新機能として OR パターンの**各アームに独立したガード**を
付与できる構文 `(pat if guard) | (pat if guard)` を追加する：
- `ast.rs` の `Pattern::Or` を `Vec<(Pattern, Option<Expr>)>` に拡張（各アームのガード保持）
- parser.rs で `(pat if cond) | (pat if cond)` を解析
- vm.rs でアーム別ガード評価ロジックを実装

**完了条件**: Rust テスト 2 件（`guard_or_pattern_per_arm` / `guard_or_pattern_fallthrough`）

**実績**: — （未実施）

---

### v61.4.0 — record update 式（`{ base | field: new_value }`）

```favnir
bind updated <- { row | status: "active", score: row.score + 10 }
bind enriched <- { order | total: order.price * order.qty, currency: "JPY" }
```

ETL で「既存レコードの一部フィールドだけを書き換えた新レコードを作る」操作を
簡潔に記述できるようにする。

`ast.rs` に `Expr::RecordUpdate { base: Box<Expr>, fields: Vec<(String, Expr)> }` を追加。
parser.rs で `{ expr | field: val, ... }` を解析（`{` の直後に識別子でなく式が来る場合に判別）。
checker.rs で `base` の型から全フィールドを継承し、`fields` で上書きされたフィールドの型を検証。

新しい `Expr` バリアントを追加するため、exhaustive match が必要な以下の全ファイルを更新する：
`compiler.rs` / `checker.rs` / `vm.rs` / `fmt.rs` / `lint.rs` /
`backend/codegen.rs` / `emit_python.rs` / `middle/ast_lower_checker.rs`

**完了条件**: Rust テスト 2 件（`record_update_basic` / `record_update_type_check`）

**実績**: — （未実施）

---

### v61.5.0 — 文字列補間強化（ネスト・マルチライン）

```favnir
bind msg <- f"user={user.name} score={Float.format(score, decimals: 2)}"
bind report <- f"""
  Summary for {user.name}:
  - Total: {total}
  - Avg:   {avg}
"""
```

`ast.rs` の `FString` をネストした式（関数呼び出し・メソッドチェーン）に対応。
`"""..."""` 形式のマルチライン文字列補間を lexer / parser に追加。

**完了条件**: Rust テスト 2 件（`fstring_nested_call` / `fstring_multiline`）

**実績**: — （未実施）

---

### v61.6.0 — 型エラーメッセージ品質（期待型 vs 実際型の差分表示）

```
E0009: type mismatch in stage output
  expected: List<Row>
  found:    List<String>
            ^^^^^^^^^^^^
  difference: Row has fields { id: Int, name: String }, but String is a scalar type.
  help: Did you forget to wrap the string in a Row record?
```

checker.rs の `unify` 失敗時に構造的差分（フィールドの有無・型の不一致箇所）を計算し
表示するロジックを追加。`error_catalog.rs` E0009 の `suggestion` を動的生成に切り替え。

**完了条件**: Rust テスト 2 件（`type_error_diff_display_record` / `type_error_suggestion_e0009`）

**実績**: — （未実施）

---

### v61.7.0 — `_` 型プレースホルダー（部分型注釈・推論ヒント）

```favnir
fn process(rows: List<_>) -> _ {
  rows |> List.filter(|r| r.active)
}
// `_` は型推論が埋める
```

checker.rs に `TypeExpr::Hole` バリアントを追加。
`_` を型注釈位置で使用した場合に型推論を走らせ、inlay hints で推論結果を表示。
W039 `type_hole_inferred` lint を追加。**W039 は `--strict` / `--perf` フラグ下でのみ
有効化される（通常の `fav lint` では無効）。** 推論結果を明示的に書くよう促す。

**完了条件**: Rust テスト 2 件（`type_hole_infers_correctly` / `type_hole_inlay_hint`）

**実績**: — （未実施）

---

### v61.8.0 — `fav check --strict` モード（追加 lint の有効化）

```bash
$ fav check --strict pipeline.fav
W039: type hole `_` inferred as `Row` — consider making explicit (pipeline.fav:3)
```

`--strict` フラグを `main.rs` に追加。通常の lint に加えて W039 を有効化（W040 は v63.6.0 で定義後に `--strict`/`--perf` 対象に追加）。
`fav.toml` の `[lint]` セクションに `strict = true` オプションを追加。

**完了条件**: Rust テスト 2 件（`check_strict_mode_enables_w039` / `fav_toml_lint_strict`）

**実績**: — （未実施）

---

### v61.9.0 — 安定化・Language Polish チェックリスト

v61.1〜v61.8 の全機能確認：
- OR パターン・as-pattern・ガード強化が既存パイプラインと共存する
- record update 式と `bind` の混在が型チェックを通過する
- `--strict` で W039 が正しく発火する（W040 は v63.6.0 以降に対象追加）

**完了条件**: Rust テスト 2 件（`pattern_all_forms_coexist` / `record_update_bind_mixed`）

**実績**: — （未実施）

---

### v62.0 — Language Polish 宣言 ★クリーンアップ

**宣言文**:

> 「パターンは OR で分岐し、as で束縛される。
>  レコードは `{ base | field: value }` で一部だけ書き換えられる。
>  型注釈に `_` を置けば推論が答えを返す。
>  エラーは期待値と実際値の差分を語り、修正の道筋を示す。
>
>  Favnir の型システムはデータエンジニアの思考を助ける存在になった。
>
>  これが Favnir v62.0 — Language Polish の姿である。」

**完了条件**:
- v61.1〜v61.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3374**）
- `v62000_tests` 4 件 pass（ベース 3370 + 4 = 3374 tests passed, 0 failed）:
  - `cargo_toml_version_is_62_0_0`
  - `changelog_has_v62_0_0`
  - `milestone_has_language_polish`
  - `readme_mentions_language_polish`
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## v63.0 — AOT Native（v62.1〜v62.9）

### v62.1.0 — `fav build` コマンド追加（cranelift object ファイル出力）

```bash
$ fav build pipeline.fav -o pipeline.o
Compiling pipeline.fav...
Linking...
Output: pipeline.o (ELF x86_64, 128 KB)
```

`main.rs` に `Some("build")` アームを追加。
`fav/src/backend/aot.rs` を新規作成。`cranelift-object` は Cargo.toml に v23 時点で
登録済みのため **Cargo.toml への追加は不要**（ただし feature フラグの有効化を確認すること）。
cranelift-object クレートを使い pipeline の IR を native object ファイルとして出力する基盤を実装。
`driver.rs` に `cmd_build_basic` 関数を追加。

**完了条件**: Rust テスト 2 件（`cmd_build_outputs_object_file` / `aot_basic_pipeline_compiles`）

**実績**: — （未実施）

---

### v62.2.0 — native binary 生成（`fav build --link`・Linux x86_64）

```bash
$ fav build pipeline.fav --link -o pipeline
$ ./pipeline
[stage LoadCsv] loaded 1000 rows
[stage Transform] processed 1000 rows
```

`aot.rs` に object ファイルのリンク処理（`cc` クレートまたは `ld` 呼び出し）を追加。
`--link` フラグで実行可能バイナリを生成。
Favnir ランタイムスタブ（`fav_rt.rs`）を静的リンクしてスタンドアロン実行を可能にする。

**完了条件**: Rust テスト 2 件（`aot_binary_executable` / `aot_runtime_stub_linked`）

**実績**: — （未実施）

---

### v62.3.0 — `fav build --target` クロスコンパイルサポート

```bash
$ fav build pipeline.fav --target aarch64-unknown-linux-gnu -o pipeline-arm
```

cranelift の `aarch64` バックエンドを有効化。
`--target <triple>` フラグを `main.rs` / `aot.rs` に追加。
Cargo.toml に `cranelift-codegen` の `arm64` feature を有効化。

**完了条件**: Rust テスト 2 件（`aot_cross_compile_aarch64` / `aot_target_triple_parsed`）

**実績**: — （未実施）

---

### v62.4.0 — AOT エフェクトディスパッチ最適化（`!Pure` ステージのインライン化）

```favnir
// !Pure ステージは AOT でインライン展開される
stage Transform !Pure: List<Row> -> List<Row> = |rows|
  rows |> List.map(transform_row)
```

`aot.rs` の IR 変換で `!Pure` フラグ付きステージを caller へインライン展開。
エフェクトのある stage（`!IO` / `!Kafka` 等）は runtime dispatch のまま維持。
`fav build --aot-stats` でインライン化された stage 数を表示。

**完了条件**: Rust テスト 2 件（`aot_pure_stage_inlined` / `aot_effectful_stage_not_inlined`）

**実績**: — （未実施）

---

### v62.5.0 — `fav bench` コマンド（AOT vs VM 速度比較）

```bash
$ fav bench pipeline.fav --runs 10
Mode     | Mean (ms) | P99 (ms) | Throughput
---------|-----------|----------|-----------
VM       |    142.3  |   158.1  | 7,030 rows/s
AOT      |     23.8  |    27.4  | 42,017 rows/s
Speedup  |     5.98x |          |
```

`fav bench` サブコマンドを追加。VM モードと AOT モードで同一パイプラインを N 回実行して
スループット・レイテンシを比較表示。結果を `bench-results.json` に出力。

**完了条件**: Rust テスト 2 件（`cmd_bench_runs_both_modes` / `bench_results_json_generated`）

**実績**: — （未実施）

---

### v62.6.0 — Docker / OCI イメージ生成（`fav build --docker`）

```bash
$ fav build pipeline.fav --docker --tag my-pipeline:latest
Building AOT binary...
Generating Dockerfile...
Building image: my-pipeline:latest
```

`fav build --docker` で Dockerfile を自動生成し `docker build` を呼び出す。
ベースイメージは `debian:slim`、AOT binary のみを含む最小構成。
`driver.rs` に `cmd_build_docker` 関数を追加。

**完了条件**: Rust テスト 2 件（`build_docker_dockerfile_generated` / `build_docker_tag_format`）

**実績**: — （未実施）

---

### v62.7.0 — `fav.toml` `[build]` セクション（AOT 設定）

```toml
[build]
target = "x86_64-unknown-linux-gnu"
opt_level = 2          # 0 | 1 | 2 | 3
inline_pure_stages = true
output_dir = "dist/"
```

`toml.rs` に `BuildConfig` 構造体を追加してパース。
`fav build` が `fav.toml` の `[build]` セクションを読み込みデフォルト設定として使用。
CLI フラグが `fav.toml` の設定を上書きする優先順位を実装。

**完了条件**: Rust テスト 2 件（`build_toml_config_parsed` / `build_cli_overrides_toml`）

**実績**: — （未実施）

---

### v62.8.0 — AOT エラーコード E0427（AOT 未サポート機能検出）

```
E0427: unsupported feature in AOT mode
  --> pipeline.fav:5:3
  |
5 |   fav eval(dynamic_expr)
  |   ^^^^^^^^^^^^^^^^^^^^^^ `eval` は AOT コンパイルではサポートされていません
  |
  help: `fav build` の代わりに `fav run` を使用するか、eval を除去してください。
```

`aot.rs` の IR 変換フェーズで AOT 未サポート機能（`eval` / 動的ディスパッチ等）を
検出して E0427 を発行するバリデーターを追加。`error_catalog.rs` に E0427 を登録。
E0427 エントリには v60.6.0 で追加した `long_description` フィールドを必ず含めること。

**完了条件**: Rust テスト 2 件（`aot_e0427_eval_detected` / `error_catalog_has_e0427`）

**実績**: — （未実施）

---

### v62.9.0 — 安定化・AOT E2E デモ

`infra/e2e-demo/aot/` を新規作成。`fav build --link` → native binary → Docker イメージ化
の E2E デモスクリプト（`scripts/build-aot.sh`）を作成。
README と `site/content/docs/runtime/aot.mdx` を追加。

**完了条件**: Rust テスト 2 件（`aot_e2e_demo_structure` / `docs_aot_mdx_exists`）

**実績**: — （未実施）

---

### v63.0 — AOT Native 宣言 ★クリーンアップ

**宣言文**:

> 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
>  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
>
>  Favnir は型安全なコンパイル言語として新たな段階に達した。
>
>  これが Favnir v63.0 — AOT Native の姿である。」

**完了条件**:
- v62.1〜v62.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3396**）
- `v63000_tests` 4 件 pass（ベース 3392 + 4 = 3396 tests passed, 0 failed）:
  - `cargo_toml_version_is_63_0_0`
  - `changelog_has_v63_0_0`
  - `milestone_has_aot_native`
  - `readme_mentions_aot_native`
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## v64.0 — Incremental & Scale（v63.1〜v63.9）

### v63.1.0 — 差分コンパイルキャッシュ（`.fav-cache/`）

```bash
$ fav run pipeline.fav      # 初回: 全 stage コンパイル
$ fav run pipeline.fav      # 2 回目: キャッシュヒット（unchanged stages skipped）
[cache] LoadCsv: hit (0ms)
[cache] Transform: miss — recompiling (12ms)
[cache] Write: hit (0ms)
```

`.fav-cache/` ディレクトリにステージ単位の bytecode と型シグネチャ hash を保存。
再実行時にソース hash と依存 hash を比較し、変更のない stage はキャッシュから読み込む。
`driver.rs` に `IncrementalCache` 構造体を追加。

**完了条件**: Rust テスト 2 件（`incremental_cache_hit_unchanged` / `incremental_cache_miss_on_change`）

**実績**: — （未実施）

---

### v63.2.0 — `fav watch` 改善（差分再コンパイルと統合）

```bash
$ fav watch pipeline.fav
[watch] pipeline.fav: monitoring...
[watch] Transform changed — recompiling 1 stage (14ms)
[watch] re-running pipeline...
```

既存の `fav watch`（500ms ポーリング）を差分コンパイルキャッシュと統合。
変更されたステージのみ再コンパイルして即座に再実行。
ファイル変更検知を `notify` クレート（既存依存）で改善（ポーリング → inotify）。

**完了条件**: Rust テスト 2 件（`watch_incremental_recompile` / `watch_notify_integration`）

**実績**: — （未実施）

---

### v63.3.0 — キャッシュ型シグネチャ不整合検出 E0428

```
E0428: incremental cache signature mismatch
  stage `Transform` の型シグネチャがキャッシュと一致しません。
  cached:  List<Row> -> List<Row>
  current: List<Row> -> List<EnrichedRow>
  キャッシュを無効化して再コンパイルします。
```

`error_catalog.rs` に E0428 を追加（v60.6.0 で導入した `long_description` フィールドを
必ず含めること）。`IncrementalCache` のシグネチャ検証で不整合を検出した際に E0428 を
警告表示し、自動的にキャッシュを無効化して再コンパイルする。

**完了条件**: Rust テスト 2 件（`incremental_e0428_signature_mismatch` / `cache_auto_invalidated`）

**実績**: — （未実施）

---

### v63.4.0 — `par` 動的スレッドプール（`[parallel]` fav.toml 設定）

```toml
[parallel]
max_threads = 8       # デフォルト: CPU コア数
queue_depth = 1000    # ステージ間バッファサイズ
```

`fav.toml` の `[parallel]` セクションを `toml.rs` に追加。
`vm.rs` の `par` 実行エンジンがスレッド数とキュー深度を設定から読み込むよう修正。
`fav run --parallel-stats` でスレッドごとの処理件数を表示。

**完了条件**: Rust テスト 2 件（`parallel_toml_config_parsed` / `parallel_stats_output`）

**実績**: — （未実施）

---

### v63.5.0 — メモリプロファイリング（`fav profile --memory`）

```bash
$ fav profile --memory pipeline.fav
Stage         | Peak RSS | Alloc/row | GC pauses
--------------|----------|-----------|----------
LoadCsv       |  42 MB   |   420 B   | 0
Transform     |  18 MB   |   180 B   | 0
Write         |   8 MB   |    80 B   | 0
Total peak    |  62 MB   |           |
```

既存の `fav profile`（stage 別実行時間）に `--memory` フラグを追加。
`driver.rs` の `cmd_profile` を拡張し、ステージ実行中の RSS と割り当てバイト数を計測。
`jemalloc` / `tracking-allocator` の組み込みは行わず `procfs` / `sysinfo` で近似計測。

**完了条件**: Rust テスト 2 件（`profile_memory_flag_works` / `profile_memory_per_stage`）

**実績**: — （未実施）

---

### v63.6.0 — バックプレッシャー制御（W040 + `[backpressure]` 設定）

```toml
[backpressure]
strategy = "drop"     # drop | block | sample
max_queue_depth = 500
warn_threshold = 400  # W041 発行閾値
```

W040 `perf_hint_large_collect` lint を実装（`collect()` の前に filter がない場合に警告）。
W041 `backpressure_queue_full` 実行時警告を `vm.rs` に追加。
`fav.toml` の `[backpressure]` セクションでキュー戦略を設定できるようにする。

**完了条件**: Rust テスト 2 件（`lint_w040_large_collect` / `backpressure_toml_parsed`）

**実績**: — （未実施）

---

### v63.7.0 — パイプライン DAG 最適化（dead stage elimination）

```
[optimizer] stage `DebugLog` has no downstream consumers — eliminated
[optimizer] stages `A -> B -> C` merged (all !Pure) — 1 stage emitted
```

`compiler.rs` にパイプライン DAG 解析パスを追加。
出力が未使用のステージ（dead stage）を IR 生成前に除去。
連続する `!Pure` ステージをひとつの stage にマージするフュージョン最適化を実装。

**完了条件**: Rust テスト 2 件（`optimizer_dead_stage_eliminated` / `optimizer_pure_stages_fused`）

**実績**: — （未実施）

---

### v63.8.0 — 標準 ETL ベンチマークスイート

```bash
$ fav bench --suite etl-standard
Benchmark: csv-to-postgres (1M rows)
  VM:  4,230 ms  (236k rows/s)
  AOT: 1,180 ms  (847k rows/s)
Benchmark: kafka-transform (10M events)
  ...
```

`fav/benchmarks/` ディレクトリに標準 ETL ベンチ（CSV 変換・Kafka 処理・JOIN）を追加。
`fav bench --suite etl-standard` で一括実行し JSON レポートを生成。
CI で `--baseline` との回帰検出を行う `bench_regression_check` テストを追加。

**完了条件**: Rust テスト 2 件（`bench_suite_etl_standard` / `bench_regression_check`）

**実績**: — （未実施）

---

### v63.9.0 — 安定化・Scale チェックリスト

差分コンパイル / `fav watch` / 動的スレッドプール / DAG 最適化の統合確認：
- 10 stage パイプラインで差分コンパイルが正しく機能する
- `par` + DAG 最適化が共存する
- W040 / W041 が `--strict` モードで正しく発火する

**完了条件**: Rust テスト 2 件（`scale_e2e_incremental_par` / `scale_dag_opt_with_par`）

**実績**: — （未実施）

---

### v64.0 — Incremental & Scale 宣言 ★クリーンアップ

**宣言文**:

> 「変更されたステージだけが再コンパイルされ、未使用のステージは除去される。
>  スレッドはコアの数だけ走り、キューはバックプレッシャーで制御される。
>  ベンチマークは数字で真実を語る。
>
>  Favnir は大規模 ETL を安心して任せられるエンジンになった。
>
>  これが Favnir v64.0 — Incremental & Scale の姿である。」

**完了条件**:
- v63.1〜v63.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3418**）
- `v64000_tests` 4 件 pass（ベース 3414 + 4 = 3418 tests passed, 0 failed）:
  - `cargo_toml_version_is_64_0_0`
  - `changelog_has_v64_0_0`
  - `milestone_has_incremental_scale`
  - `readme_mentions_incremental_scale`
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## v65.0 — Performance 1.0 宣言（v64.1〜v64.9）

### v64.1.0 — AOT ビルドの CI 統合（`fav build --ci`）

```yaml
# .github/workflows/build.yml
- run: fav build pipeline.fav --link --ci -o dist/pipeline
- run: ./dist/pipeline --validate
```

`fav build --ci` フラグで CI 向け出力（色なし・機械可読エラー形式・exit code 厳格化）を追加。
GitHub Actions / GitLab CI 向けのワークフローテンプレートを `fav new` に追加。

**完了条件**: Rust テスト 2 件（`build_ci_flag_output_format` / `new_template_has_ci_workflow`）

**実績**: — （未実施）

---

### v64.2.0 — パフォーマンスリグレッションテスト自動化

```bash
$ fav bench --compare main..HEAD
Regression detected: Transform stage +18% slower (was 12ms, now 14ms)
```

`fav bench --compare <ref>` で git ref 間のベンチ比較を実装。
`driver.rs` に `cmd_bench_compare` を追加。
`fav.toml` の `[bench]` セクションで許容劣化率（`regression_threshold_pct`）を設定。

**完了条件**: Rust テスト 2 件（`bench_compare_detects_regression` / `bench_toml_threshold`）

**実績**: — （未実施）

---

### v64.3.0 — パフォーマンスガイド（`site/content/docs/runtime/performance.mdx`）

AOT コンパイル・差分コンパイル・並列最適化・DAG 最適化・バックプレッシャーの
使い方をまとめたパフォーマンスチューニングガイドを作成。
`fav bench` / `fav profile` の出力の読み方・ボトルネック特定手順を掲載。

**完了条件**: Rust テスト 2 件（`docs_performance_guide_exists` / `docs_performance_has_aot_section`）

**実績**: — （未実施）

---

### v64.4.0 — `fav profile` flamegraph 改善（AOT / VM 統合・ブラウザ表示）

```bash
$ fav profile --flamegraph pipeline.fav
Generated: fav-profile.svg
Opening in browser...
```

既存の `fav profile`（inferno クレート）を AOT バイナリのプロファイル結果にも対応。
`--flamegraph` フラグで SVG を生成し `open` クレートでブラウザ表示。
VM と AOT の flamegraph を並べて比較表示するモードを追加。

**完了条件**: Rust テスト 2 件（`profile_flamegraph_aot` / `profile_flamegraph_svg_generated`）

**実績**: — （未実施）

---

### v64.5.0 — 外部ベンチマーク比較（Python pandas / dbt との比較レポート）

```
Benchmark: 1M row CSV transform
  Favnir AOT: 1,180 ms  (847k rows/s)  ✓
  pandas:     8,340 ms  (120k rows/s)  7.1× slower
  dbt (SQL):  3,210 ms  (312k rows/s)  2.7× slower
```

`site/content/docs/runtime/benchmarks.mdx` に比較ベンチマーク結果ページを作成。
再現可能なベンチマークスクリプト（`fav/benchmarks/compare/`）を公開。

**完了条件**: Rust テスト 2 件（`docs_benchmarks_page_exists` / `benchmark_compare_script_exists`）

**実績**: — （未実施）

---

### v64.6.0 — `fav lint --perf`（パフォーマンス lint 一括実行）

```bash
$ fav lint --perf pipeline.fav
W040: large `collect()` without filter (pipeline.fav:22) [perf]
W039: type hole `_` reduces AOT optimization (pipeline.fav:5) [perf]
```

`fav lint` に `--perf` フラグを追加して W039 / W040 / W041 を一括有効化。
`fav.toml` の `[lint]` セクションで `perf = true` を設定できるようにする。

**完了条件**: Rust テスト 2 件（`lint_perf_flag_enables_w039_w040` / `lint_toml_perf_setting`）

**実績**: — （未実施）

---

### v64.7.0 — `fav build` wasm32 出力（Playground 向け）

```bash
$ fav build pipeline.fav --target wasm32-unknown-unknown -o pipeline.wasm
```

既存の WASM ビルド基盤（`wasm-encoder` クレート）を `fav build --target wasm32` と統合。
Playground の `@favnir/wasm` パッケージに AOT WASM を組み込めるよう準備。

**完了条件**: Rust テスト 2 件（`build_wasm_target_outputs_wasm` / `wasm_build_compat_check`）

**実績**: — （未実施）

---

### v64.8.0 — ドキュメントサイト Performance 1.0 総括記事

`site/content/docs/performance/performance1-overview.mdx` を作成。
v61〜v64 の全パフォーマンス機能（DX 2.0 / Language Polish / AOT / Incremental & Scale）を
統括する概観記事とクイックスタートガイドを記述。

**完了条件**: Rust テスト 2 件（`docs_performance1_overview_exists` / `docs_performance1_has_quickstart`）

**実績**: — （未実施）

---

### v64.9.0 — 安定化・コードフリーズ（Performance 1.0 前調整）

v61〜v64 の全テストが通過していることを確認。
全 lint / clippy クリーン確認。
`site/content/docs/performance/performance1-overview.mdx` の最終確認。

**完了条件**: Rust テスト 2 件（`scale_all_v64_features_stable` / `performance1_overview_doc_complete`）

**実績**: — （未実施）

---

### v65.0 — Performance 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデータパイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

**完了条件**:
- v64.1〜v64.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3440**）
- `v65000_tests` 4 件 pass（ベース 3436 + 4 = 3440 tests passed, 0 failed）:
  - `cargo_toml_version_is_65_0_0`
  - `changelog_has_v65_0_0`
  - `milestone_has_performance1`
  - `readme_mentions_performance1`
- `MILESTONE.md` に `"Performance 1.0"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## テスト数推移

| バージョン | 想定テスト数 | 累積増加 | 備考 |
|---|---|---|---|
| v60.0.0（ベース） | 3330 | — | 実績値（2026-07-30 COMPLETE） |
| v60.1〜v60.9 | 3330 + 18 = 3348 | +18 | サブスプリント 9 件 × 2 |
| v61.0.0 | 3352 | +4 | DX 2.0 宣言（★クリーンアップ） |
| v61.1〜v61.9 | 3352 + 18 = 3370 | +18 | |
| v62.0.0 | 3374 | +4 | Language Polish 宣言（★クリーンアップ） |
| v62.1〜v62.9 | 3374 + 18 = 3392 | +18 | |
| v63.0.0 | 3406 | +4 | AOT Native 宣言（★クリーンアップ）✅ |
| v63.1〜v63.9 | 3406 + 18 = 3424 | +18 | |
| v64.0.0 | 3428 | +4 | Incremental & Scale 宣言（★クリーンアップ） |
| v64.1〜v64.9 | 3418 + 18 = 3436 | +18 | |
| v65.0.0 | 3440 | +4 | Performance 1.0 宣言（★クリーンアップ） |

各サブスプリント 2 件追加、各マイルストーン 4 件追加（x.0.0 テストモジュール）。
実際の件数はサブスプリントロードマップ作成時に確定する。

---

## 追加されるエラーコード・警告コード

| コード | バージョン | 内容 |
|---|---|---|
| W039 | v61.7.0 | `type_hole_inferred` — `_` 型プレースホルダーが推論された |
| W040 | v63.6.0 | `perf_hint_large_collect` — filter なし大規模 collect |
| W041 | v63.6.0 | `backpressure_queue_full` — バックプレッシャーキュー超過 |
| E0427 | v62.8.0 | `aot_dynamic_feature` — AOT 未サポート動的機能の使用 |
| E0428 | v63.3.0 | `incremental_cache_conflict` — 差分キャッシュ型シグネチャ不一致 |

---

## 参考リンク

- 前マスターロードマップ（完了）: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント詳細（完了）: `versions/roadmap/roadmap-v59.1-v60.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
