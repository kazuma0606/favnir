# Roadmap v67.1.0 〜 v68.0.0 — Developer Intelligence

Date: 2026-08-04
Status: 未着手（v67.0.0 完了後に開始）

マスターロードマップ: [roadmap-v65.1-v70.0.md](roadmap-v65.1-v70.0.md)

---

## 前提

- 直前完了: v67.0.0「AI-Native Stage Layer」（tests = 3497）
- 本スプリントは Phase 3「Developer Intelligence」の詳細計画
- 目標: v68.0.0「Developer Intelligence 宣言」（tests = 3519）

### 設計方針

**デバッガの哲学**: パイプラインのステップを「止めて見る」ことができる。
時間を遡って過去の状態を確認し、問題の根本原因を特定する。

**AI アシスタントの哲学**: プロファイリングデータを読んで「次の一手」を提案する。
AI が提案し、ユーザーが承認し、`fav fix` が適用する。人間が主導権を持つ。

**可視化の哲学**: テキストでも、グラフィカルでも。
CI 環境では `--ascii`、ブラウザでは `--format svg` を使い分ける。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v67.1.0 | `fav debug`（ステップ実行デバッガ） | 3497 + 2 = 3499 | 完了 |
| v67.2.0 | Time-Travel Debugging（記録 & リプレイ） | 3499 + 2 = 3501 | 完了 |
| v67.3.0 | `fav viz`（パイプライン DAG 可視化） | 3501 + 2 = 3503 | 完了 |
| v67.4.0 | `fav suggest`（AI 最適化アドバイザー） | 3503 + 2 = 3505 | 完了 |
| v67.5.0 | `fav simulate`（合成データパイプラインテスト） | 3505 + 2 = 3507 | 完了 |
| v67.6.0 | Pipeline Property Testing（`Rune.proptest`） | 3507 + 2 = 3509 | 完了 |
| v67.7.0 | Interactive Profiling（`fav profile --interactive`） | 3509 + 2 = 3511 | 完了 |
| v67.8.0 | Math-Aware Doc Generation（`fav doc --math`） | 3511 + 2 = 3513 | 完了 |
| v67.9.0 | 安定化・コードフリーズ | 3513 + 2 = 3515 | 完了 |
| v68.0.0 | Developer Intelligence 宣言 ★クリーンアップ | 3515 + 4 = 3519 | 完了 |

---

## v67.1.0 — `fav debug`（ステップ実行デバッガ）

**概要**: パイプラインをステップ単位で実行し、各ステージの入出力を確認できるデバッガ。
AI パイプラインの LLM 呼び出し・ベクトル変換を「見える化」する。

```bash
$ fav debug pipeline.fav
[fav debug] v67.1.0 — ステップ実行モード
> run
[step 1/4] LoadCsv       → 1000 rows  (2ms)    ← 自動停止
> inspect row[0]         # { id: 1, text: "...", amount: 42.0 }
> continue
[step 2/4] EmbedText     → Vec[1536] x 1000  (1240ms)
> inspect embedding[0]   # [0.021, -0.134, 0.082, ...]
> breakpoint "Validate"  # 次のステップで停止
> continue
[step 3/4] Validate      → 998 rows   (8ms)    ← ブレークポイント停止
> diff row[0]            # 入力と出力の差分表示
> continue
[step 4/4] InsertDB      → 998 rows   (120ms)
[done] Pipeline completed successfully.
```

**実装内容**:

- `cmd_debug(src, args)` — デバッグモードの実行
- ステップ単位の一時停止（各 stage 実行後に自動停止）
- `inspect <expr>` — レコード・ベクトルの内容確認
- `breakpoint <stage_name>` — 特定ステージで停止
- `continue` / `step` / `quit` コマンド
- `diff` — ステージ前後のレコード差分表示

**ファイル**:
- `fav/src/debug.rs`（デバッガ実装）
- `site/content/docs/tools/debug.mdx`

**完了条件**: Rust テスト 2 件（3497 + 2 = **3499**）

```rust
// driver.rs mod v67100_tests
fn debug_step_execution()    // cmd_debug 呼び出しで "step" / "inspect" キーワードを返す
fn debug_breakpoint_stage()  // "breakpoint" 機能のヘルプ文字列を含む
```

---

## v67.2.0 — Time-Travel Debugging（記録 & リプレイ）

**概要**: パイプライン実行を `.fav-trace` ファイルに記録し、任意のステップに巻き戻す。
本番障害の再現に威力を発揮。「再現性のある調査」を実現する。

```bash
# 記録
$ fav run pipeline.fav --record session.fav-trace
[record] Tracing execution to session.fav-trace
[step 3/4] Validate: FAILED (E0042: schema mismatch)
[record] 4 steps recorded.

# リプレイ
$ fav debug --replay session.fav-trace
[replay] 4 steps available.
> rewind 2           # step 2 に巻き戻す
[replay] Rewound to step 2 (EmbedText).
> inspect embedding[5]   # 問題のある行を確認
> forward             # step 3 に進む
[replay] step 3: Validate — FAILED
> inspect error       # エラーの詳細確認
```

**実装内容**:

- `--record <path>` フラグ — 実行トレースの記録（各ステージの入出力をシリアライズ）
- `--replay <path>` フラグ — トレースファイルからのリプレイ
- `rewind <step>` コマンド — 任意のステップに巻き戻し
- `forward` コマンド — 1 ステップ進む
- トレースファイルフォーマット: `.fav-trace`（バイナリ、gzip 圧縮）
- メモリ効率: ラージデータの参照のみ記録（コピーしない）

**完了条件**: Rust テスト 2 件（3499 + 2 = **3501**）

```rust
// driver.rs mod v67200_tests
fn debug_record_replay()      // --record / --replay フラグの説明文字列を含む
fn debug_rewind_to_step()     // "rewind" / "forward" / ".fav-trace" キーワードを含む
```

---

## v67.3.0 — `fav viz`（パイプライン DAG 可視化）

**概要**: パイプラインの依存関係を DAG（有向非巡回グラフ）として可視化する。
CI ではアスキーアート、ブラウザでは SVG。ステージ別実行時間も表示。

```bash
$ fav viz pipeline.fav --ascii
LoadCsv ──► EmbedText ──► Validate ──┬──► InsertDB
                                      └──► SendSlack

$ fav viz pipeline.fav --format svg -o pipeline.svg
# ブラウザで開ける SVG（ステージ別色分け + 実行時間付き）

$ fav viz pipeline.fav --format mermaid
# Mermaid 形式出力（GitHub README に埋め込み可能）
```

**実装内容**:

- `cmd_viz(src, format, output)` — DAG 可視化
- アスキーアート出力（`--ascii` / `--format ascii`）
- SVG 出力（`--format svg`）— ステージ別色分け、実行時間付き
- Mermaid 出力（`--format mermaid`）— GitHub/Notion に貼り付け可能
- `par` ステージの並列表示（分岐・合流を視覚化）
- プロファイルデータとの統合: `fav viz --from-profile fav-profile.json`（ホットスポットをハイライト）

**ファイル**:
- `fav/src/viz.rs`

**完了条件**: Rust テスト 2 件（3501 + 2 = **3503**）

```rust
// driver.rs mod v67300_tests
fn viz_ascii_dag()           // cmd_viz で ascii 出力に "──►" / stage 名を含む
fn viz_svg_with_timing()     // "svg" / "mermaid" フォーマット出力の説明を含む
```

---

## v67.4.0 — `fav suggest`（AI 最適化アドバイザー）

**概要**: プロファイリング結果 + LLM でボトルネックの自動分析と最適化提案。
AI が提案し、`fav fix --apply` が適用する。人間が承認主導。

```bash
$ fav suggest pipeline.fav --from-profile fav-profile.json

Suggestion 1 [HIGH IMPACT] Transform stage: 847ms (72% of total)
  Pattern detected: `collect { yield }` は AOT コンパイルで最適化不可
  Fix: List.map / List.filter に書き換え → AOT で 3× 高速化
  → fav fix --apply suggestion-1.patch

Suggestion 2 [MED] EmbedText: 1240ms, sequential
  Pattern: 1000 件を逐次処理中
  Fix: par [EmbedText x 4] に変更 → スループット 4× 向上
  → fav fix --apply suggestion-2.patch

Suggestion 3 [LOW] InsertDB: N+1 クエリ検出（1000 回の個別 INSERT）
  Fix: Rune.postgres.insert_batch に変更 → レイテンシ 10× 削減
```

**実装内容**:

- `cmd_suggest_profile(src, profile_path)` — プロファイルを読んで提案生成（既存の `cmd_suggest(error_code, location)` は変更しない）
- パターン検出: `collect { yield }` / N+1 クエリ / 逐次処理可能な並列化
- LLM 連携: プロファイルデータを Claude API に送信して提案文生成（v67.4.0 ではスタブ実装）
- パッチ生成: `--apply <patch>` で自動適用可能な diff を出力
- 提案の優先度付け: [HIGH IMPACT] / [MED] / [LOW]

**完了条件**: Rust テスト 2 件（3503 + 2 = **3505**）

```rust
// driver.rs mod v67400_tests
fn suggest_from_profile()  // cmd_suggest が "Suggestion" / "[HIGH IMPACT]" を含む出力を返す
fn suggest_applies_fix()   // "--apply" / "patch" キーワードを含む出力を返す
```

---

## v67.5.0 — `fav simulate`（合成データパイプラインテスト）

**概要**: `Rune.gen` で生成した合成データを使ってパイプラインをテストする。
本番データなしに挙動を検証し、エッジケースを発見する。

```favnir
// pipeline.test.fav
simulate SemanticSearch {
    input: Rune.gen.text(count: 100, seed: 42),
    assert: |result| { result.len() <= 10 && result[0].score > 0.8 }
}

simulate EmbedText {
    input: Rune.gen.string(length: 500, seed: 1),
    assert: |vec| { vec.len() == 1536 && Rune.linalg.norm(vec) > 0.0 }
}
```

```bash
$ fav simulate pipeline.test.fav
[simulate] SemanticSearch: 100 cases... PASS (avg 23ms, max 87ms)
[simulate] EmbedText: 1 case... PASS (vec[1536], norm=1.0)
[done] 2/2 simulations passed.
```

**実装内容**（v67.5.0 はスタブ実装。parser 拡張・gen 実装は将来フェーズ）:

- `simulate <StageName> { input: ..., assert: ... }` 構文（parser 拡張）— 将来フェーズ
- 合成データジェネレータ: `Rune.gen.text`, `Rune.gen.string`, `Rune.gen.int_list`, `Rune.gen.record` — 将来フェーズ
- シード再現性: 同一 seed で同一データを生成 — 将来フェーズ
- アサーション失敗時: 失敗した入力データと出力を表示（出力文字列スタブで実装）

**完了条件**: Rust テスト 2 件（3505 + 2 = **3507**）

```rust
// driver.rs mod v67500_tests
fn simulate_pipeline_with_synthetic() // cmd_simulate が "simulate" / "PASS" キーワードを含む
fn simulate_assertion_failure()       // アサーション失敗時の "FAIL" / 入力データ表示を含む
```

---

## v67.6.0 — Pipeline Property Testing（`Rune.proptest`）

**概要**: プロパティベーステスト（PBT）でパイプラインの不変条件を検証。
ランダム入力でエッジケースを自動探索し、最小反例を自動縮小（shrink）する。

```favnir
proptest stage Transform {
    // 正の入力に対して出力も正
    forall x: Int where x > 0 { Transform(x) > 0 }
    // ゼロ入力はゼロ出力
    forall x: Int where x == 0 { Transform(x) == 0 }
}

proptest stage EmbedText {
    // 埋め込みベクトルは常に正規化済み（norm ≈ 1.0）
    forall text: String where text.len() > 0 {
        |Rune.linalg.norm(EmbedText(text)) - 1.0| < 0.001
    }
}
```

**実装内容**:

- `proptest` 構文（parser 拡張）
- ランダム入力生成: `forall x: T` でランダムサンプリング
- 反例縮小（shrink）: 失敗ケースを最小化して表示
- 実行回数: デフォルト 100 試行（`--proptest-runs <n>` で変更）
- 型別ジェネレータ: `Int`, `Float`, `String`, `List<T>`, `Record`

**完了条件**: Rust テスト 2 件（3507 + 2 = **3509**）

```rust
// driver.rs mod v67600_tests
fn proptest_stage_invariant()       // "proptest" / "forall" / "shrink" キーワードを含む
fn proptest_counterexample_shrink() // 反例縮小の説明 / "--proptest-runs" を含む
```

---

## v67.7.0 — Interactive Profiling（`fav profile --interactive`）

**概要**: プロファイリング結果をインタラクティブに探索する。
ホットスポットをドリルダウンし、コード行レベルでボトルネックを特定する。

```bash
$ fav profile --interactive pipeline.fav

[hotspot] Transform: 847ms (72% of total)  ← カーソルを当てて選択
> drill
  [line 12] collect { yield ... } — 723ms (85% of Transform)
  → W041: AOT 最適化非対応パターン
  → Suggestion: List.map に変換で 3× 高速化
  → Apply fix? [y/N]: y
  → Applying... Done. 再プロファイル中...
  [line 12] List.map { ... }        — 241ms (33ms 削減)

[hotspot] EmbedText: 1240ms (次のボトルネック)
> drill
  [API calls] Rune.openai.embed: 1000 回 sequential
  → Suggestion: batch_embed(texts, batch_size: 50) で 20× 高速化
```

**実装内容**:

- `--interactive` フラグ — REPL 風のプロファイル探索
- `drill` コマンド — ホットスポットをコード行レベルにドリルダウン
- lint 統合: W041 等の警告を自動表示
- `fav suggest` 連携: ドリルダウン中に最適化提案を表示
- インクリメンタル再プロファイル: 修正後の diff を即時確認

**完了条件**: Rust テスト 2 件（3509 + 2 = **3511**）

```rust
// driver.rs mod v67700_tests
fn profile_interactive_hotspot() // cmd_profile に "--interactive" / "hotspot" キーワードを含む
fn profile_interactive_drill()   // "drill" / "Suggestion" キーワードを含む
```

---

## v67.8.0 — Math-Aware Doc Generation（`fav doc --math`）

**概要**: 数学 Rune の関数ドキュメントに LaTeX 数式を埋め込む。
`///` コメントで数式を記述し、`fav doc` が Markdown + MathJax 形式で出力する。

```favnir
/// Computes the gradient of scalar function `f` at point `x`.
///
/// Formula: ∇f(x) = (∂f/∂x₁, ∂f/∂x₂, ..., ∂f/∂xₙ)
///
/// Uses reverse-mode automatic differentiation (backpropagation).
///
/// ```favnir
/// bind g = Rune.autodiff.grad(|x| { x * x }, 3.0)
/// // g == 6.0  (derivative of x² at x=3 is 2x = 6)
/// ```
///
/// $$ \nabla f(x) = \frac{\partial f}{\partial x} $$
public fn grad(f: Float -> Float, x: Float) -> Float { ... }
```

**実装内容**:

- `fav doc --math` フラグ — LaTeX 数式を MathJax 記法で出力
- `$$...$$` ブロック数式、`$...$` インライン数式のパース
- 出力フォーマット: Markdown（`--format md`）、HTML + MathJax（`--format html`）
- コードブロックのコンパイル確認: `/// ``` favnir` 内のコードが型チェックを通ることを確認
- サイト統合: `fav doc --format mdx` で site/ に直接出力

**完了条件**: Rust テスト 2 件（3511 + 2 = **3513**）

```rust
// driver.rs mod v67800_tests
fn doc_math_latex_rendered()    // cmd_doc_math が "$$" / "MathJax" / "∇" を含む出力を返す
fn doc_math_example_compiles()  // "--math" / "--format" キーワードを含む
```

---

## v67.9.0 — 安定化・コードフリーズ（Developer Intelligence 前調整）

**概要**: v67.1〜v67.8 の全機能が正常動作することを確認する安定化バージョン。
デバッガ・可視化・AI 提案・テストツール群の統合確認。

**確認内容**:

- `fav debug` / `fav viz` / `fav suggest` / `fav simulate` の全コマンドが正常起動
- `Rune.proptest` 構文が型チェックを通ること
- `site/content/docs/tools/developer-intelligence.mdx` の作成

**完了条件**: Rust テスト 2 件（3513 + 2 = **3515**）

```rust
// driver.rs mod v67900_tests
fn dev_intelligence_all_stable()           // debug/viz/suggest/simulate の各コマンドが存在
fn debug_viz_suggest_docs_complete()       // developer-intelligence.mdx が存在し "fav debug" を含む
```

---

## v68.0.0 — Developer Intelligence 宣言 ★クリーンアップ

**宣言文**:

> 「ステップ実行デバッガが、AI パイプラインの内部を露わにする。
>  時間を遡って本番障害を再現し、DAG 可視化が依存関係を一目で示す。
>  AI アドバイザーがプロファイリングデータを読み、次の最適化を提案する。
>
>  これが Favnir v68.0 — Developer Intelligence の姿である。」

**タスク**:

- [ ] `fav/Cargo.toml` version を `"68.0.0"` に更新
- [ ] `MILESTONE.md` 先頭に v68.0.0「Developer Intelligence」エントリを追加
- [ ] `README.md` に v68.0.0 宣言文を追加
- [ ] `CHANGELOG.md` 先頭に v68.0.0 エントリを追加
- [ ] `v68000_tests` 4 件を `driver.rs` に追加
- [ ] `cargo clean` 実行（★クリーンアップ）
- [ ] `cargo test -j 8 -- --test-threads=8` で 3519 tests passed を確認

**完了条件**: `v68000_tests` 4 件（3515 + 4 = **3519**）

```rust
// driver.rs mod v68000_tests
fn cargo_toml_version_is_68_0_0()      // Cargo.toml に "version = \"68.0.0\"" を含む
fn changelog_has_v68_0_0()             // CHANGELOG.md に "v68.0.0" を含む
fn milestone_has_dev_intelligence()    // MILESTONE.md に "Developer Intelligence" を含む
fn readme_mentions_dev_intelligence()  // README.md に "Developer Intelligence" または "v68.0" を含む
```

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v67.0.0（ベース） | 3497 | — |
| v67.1.0 | 3499 | +2 |
| v67.2.0 | 3501 | +2 |
| v67.3.0 | 3503 | +2 |
| v67.4.0 | 3505 | +2 |
| v67.5.0 | 3507 | +2 |
| v67.6.0 | 3509 | +2 |
| v67.7.0 | 3511 | +2 |
| v67.8.0 | 3513 | +2 |
| v67.9.0 | 3515 | +2 |
| v68.0.0 | 3519 | +4 |
