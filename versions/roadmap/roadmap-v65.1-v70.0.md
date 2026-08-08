# Roadmap v65.1.0 〜 v70.0.0 — Intelligent ETL 1.0

Date: 2026-08-04
Status: 進行中（v65.0.0 完了、v65.1.0 から開始）

---

## 前提

- 直前完了: v65.0.0「Performance 1.0」（tests = 3453）
- 本文書は v65.1〜v70.0 の**マスターロードマップ**
- 各マイルストーン開始時に対応するサブスプリントロードマップを作成する

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v65.1-v66.0.md` | v65.1〜v65.9 + v66.0 | 作成済み |
| `roadmap-v66.1-v67.0.md` | v66.1〜v66.9 + v67.0 | 作成済み |
| `roadmap-v67.1-v68.0.md` | v67.1〜v67.9 + v68.0 | 作成済み |
| `roadmap-v68.1-v69.0.md` | v68.1〜v68.9 + v69.0 | 作成済み |
| `roadmap-v69.1-v70.0.md` | v69.1〜v69.9 + v70.0 | 作成済み |

---

## ビジョン

> **「型チェックが、AI の出力を安全にする。」**

v65.0「Performance 1.0」で Favnir は「型安全 × 高速」を実現した。
次の問いは——型安全と高速の上に、**数学的正確性と知性**を乗せられるか、だ。

2026年のデータエンジニアリングの最前線は **AI データパイプライン** である。
LLM・ベクトル検索・リアルタイム推論・自動微分を組み込んだパイプラインが急増している。
しかしこれらのパイプラインは**型なし・壊れやすく・デバッグ不能**という現実がある。

Favnir が「型安全な AI パイプライン言語」になれば、誰も持っていない差別化が生まれる。

その道筋は 4 段階だ：

```
Math & Science Foundation   ─ 数学の土台なき AI は砂上の楼閣
AI-Native Stage Layer       ─ 型システムで LLM・ベクトルを制御する
Developer Intelligence      ─ パイプラインをデバッグし、AI が次の手を示す
Distributed Favnir          ─ 型安全な AI パイプラインを大規模に動かす
                            ↓
            v70.0 — Intelligent ETL 1.0
```

---

## 宣言文（v70.0 目標）

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

---

## Phase 1: v65.1〜v65.9 + v66.0 — Math & Science Foundation

**テーマ**: 「数学の土台なき AI は砂上の楼閣」

AI パイプラインの信頼性は数学的正確性から始まる。
線形代数・統計・自動微分・最適化を型安全な Rune として Favnir に組み込む。
この Phase が完了してはじめて、AI ステージが「型で守られた」ものになる。

### v65.1.0 — Linear Algebra Rune（`Rune.linalg`）

行列・ベクトルを型付きで扱う線形代数 Rune。
**型パラメータで次元数を保証**することが最大の差別化点。

```favnir
// 次元違いはコンパイルエラー
public stage PCA: Matrix<Float>[1000, 128] -> Matrix<Float>[1000, 32] = |m| {
    Rune.linalg.svd(m, components: 32)
}
```

実装内容:
- `Matrix<T>[rows, cols]` 型 + `Vec<T>[n]` 型（次元数を型パラメータに）
- `dot`, `matmul`, `transpose`, `inverse`, `svd`, `eig`, `norm`, `diag`
- `LU`, `QR`, `Cholesky` 分解

**完了条件**: Rust テスト 2 件（ベース 3453 + 2 = 3455）
- `linalg_rune_matrix_ops`
- `linalg_rune_svd_decomposition`

---

### v65.2.0 — Statistics Rune（`Rune.stats`）

記述統計・確率分布・仮説検定・回帰分析を型安全に扱う統計 Rune。

```favnir
public stage SummaryStats: List<Float> -> StatsReport = |data| {
    Rune.stats.describe(data)
    // → { mean: 4.2, std: 1.1, median: 4.0, p95: 6.3, skewness: 0.3 }
}

public stage AnomalyDetect: List<Float> -> List<Anomaly> = |data| {
    bind dist = Rune.stats.fit(NormalDist, data)
    Rune.stats.zscore_filter(data, dist, threshold: 3.0)
}
```

実装内容:
- 記述統計: `mean`, `std`, `median`, `percentile`, `skewness`, `kurtosis`
- 分布: `NormalDist`, `PoissonDist`, `BinomialDist`, `ExponentialDist` + `fit` / `sample` / `pdf` / `cdf`
- 仮説検定: `t_test`, `chi_square`, `ks_test`, `mannwhitney`
- 回帰: `linear_regression`, `logistic_regression` → 係数・p 値・R²

**完了条件**: Rust テスト 2 件（ベース 3455 + 2 = 3457）
- `stats_rune_describe`
- `stats_rune_hypothesis_test`

---

### v65.3.0 — Autodiff Rune（`Rune.autodiff`）

自動微分（reverse-mode AD）を Favnir の型システムと統合。
勾配計算・バックプロパゲーションを型安全に表現する。

```favnir
// 勾配を型で保証 — 入力と勾配は同じ型
public stage GradientStep: Tensor<Float> -> Tensor<Float> = |params| {
    bind loss_fn = |p: Tensor<Float>| -> Float { model_loss(p) }
    bind grad = Rune.autodiff.grad(loss_fn, params)
    params - 0.01 * grad
}
```

実装内容:
- `Tape` — 計算グラフの記録（動的 AD）
- `grad(f, x)` — スカラー値関数の勾配
- `jacobian(f, x)` — ベクトル値関数のヤコビアン
- `hessian(f, x)` — 二階微分
- チェーンルール自動適用（`+`, `*`, `exp`, `log`, `sin`, `cos`, `tanh`）

**完了条件**: Rust テスト 2 件（ベース 3457 + 2 = 3459）
- `autodiff_rune_grad_scalar`
- `autodiff_rune_chain_rule`

---

### v65.4.0 — Optimization Rune（`Rune.optim`）

勾配ベースの最適化アルゴリズム群。Autodiff Rune と組み合わせて ML 訓練ループを型安全に。

```favnir
public stage TrainModel: Dataset -> ModelParams = |data| {
    bind optimizer = Rune.optim.adam(lr: 0.001, beta1: 0.9, beta2: 0.999)
    Rune.optim.minimize(
        loss_fn: |params| { cross_entropy(model(params, data), data.labels) },
        initial: ModelParams.random(),
        optimizer: optimizer,
        max_iter: 1000,
        tol: 1e-6
    )
}
```

実装内容:
- `SGD`, `Adam`, `AdaGrad`, `RMSProp` オプティマイザ
- `minimize(loss_fn, initial, optimizer, max_iter, tol)` — 汎用最適化ループ
- `line_search`, `conjugate_gradient`, `l_bfgs`
- 収束判定・早期終了・学習率スケジューラ

**完了条件**: Rust テスト 2 件（ベース 3459 + 2 = 3461）
- `optim_rune_adam_converges`
- `optim_rune_minimize_quadratic`

---

### v65.5.0 — Numerical Methods Rune（`Rune.numeric`）

数値積分・微分方程式・補間・フーリエ変換などの数値解析ツール群。

```favnir
// 数値積分（シンプソン則）
public stage IntegrateSignal: List<Float> -> Float = |samples| {
    Rune.numeric.integrate(samples, method: Simpson, dx: 0.01)
}

// フーリエ変換（信号処理パイプライン）
public stage FrequencyAnalysis: List<Float> -> Spectrum = |signal| {
    Rune.numeric.fft(signal)
}
```

実装内容:
- 数値積分: `trapezoid`, `simpson`, `gauss_quadrature`
- ODE ソルバー: `euler`, `runge_kutta4`, `dormand_prince`（適応刻み幅）
- 補間: `linear`, `cubic_spline`, `polynomial`
- フーリエ: `fft`, `ifft`, `power_spectrum`
- 根探索: `bisection`, `newton_raphson`, `brent`

**完了条件**: Rust テスト 2 件（ベース 3461 + 2 = 3463）
- `numeric_rune_integrate`
- `numeric_rune_fft`

---

### v65.6.0 — Time Series Rune（`Rune.timeseries`）

時系列データの解析・予測・異常検知を型安全に扱う Rune。

```favnir
public stage ForecastDemand: TimeSeries<Float> -> Forecast = |sales| {
    bind model = Rune.timeseries.fit(SARIMA(p:1, d:1, q:1, P:1, D:1, Q:1, s:7), sales)
    Rune.timeseries.predict(model, horizon: 30)
}

public stage DetectSeasonality: TimeSeries<Float> -> SeasonalComponents = |ts| {
    Rune.timeseries.decompose(ts, method: STL, period: 7)
}
```

実装内容:
- `ARIMA`, `SARIMA`, `Exponential Smoothing`（Holt-Winters）
- `STL` 季節分解、トレンド抽出
- `ChangePointDetection`（PELT / BOCPD）
- `autocorrelation`, `partial_autocorrelation`, `adf_test`（単位根検定）
- `resample`, `rolling_mean`, `ewm`（指数加重平均）

**完了条件**: Rust テスト 2 件（ベース 3463 + 2 = 3465）
- `timeseries_rune_arima_fit`
- `timeseries_rune_stl_decompose`

---

### v65.7.0 — ML Primitives Rune（`Rune.ml`）

古典的 ML アルゴリズムを型安全なステージとして提供する Rune。
LLM に依存しない「型で守られた ML」の基盤。

```favnir
public stage ClassifyCustomers: CustomerFeatures -> Segment = |features| {
    bind model = Rune.ml.knn(k: 5, metric: Cosine)
    Rune.ml.predict(model, features)
}

public stage FeatureImportance: Dataset -> FeatureRanking = |data| {
    bind tree = Rune.ml.random_forest(n_estimators: 100, max_depth: 10)
    Rune.ml.feature_importance(tree, data)
}
```

実装内容:
- 分類: `KNN`, `NaiveBayes`, `RandomForest`, `GradientBoosting`
- 回帰: `Ridge`, `Lasso`, `ElasticNet`, `SVR`
- クラスタリング: `KMeans`, `DBSCAN`, `HierarchicalClustering`
- 次元削減: `PCA`（linalg Rune 上）, `UMAP`, `t-SNE`
- 評価: `accuracy`, `f1_score`, `roc_auc`, `confusion_matrix`
- `cross_validate`, `grid_search`（ハイパーパラメータ探索）

**完了条件**: Rust テスト 2 件（ベース 3465 + 2 = 3467）
- `ml_rune_random_forest_classify`
- `ml_rune_cross_validate`

---

### v65.8.0 — Math Lint Rules（W050〜W054）

数学 Rune 特有のアンチパターンを静的解析で検出する lint ルール。

```
W050: 行列次元の不一致が型推論で検出できない動的パス（警告）
W051: 数値不安定な演算（ゼロ除算・log(0) の可能性）
W052: 統計的有意性なしの比較（サンプルサイズ < 30 の t 検定）
W053: 自動微分ループでの in-place 変更（テープ破壊の危険）
W054: 最適化ループの収束条件未設定（無限ループのリスク）
```

**完了条件**: Rust テスト 2 件（ベース 3467 + 2 = 3469）
- `lint_w051_detects_div_zero_risk`
- `lint_w053_detects_inplace_in_autodiff`

---

### v65.9.0 — 安定化・コードフリーズ（Math & Science 前調整）

v65.1〜v65.8 の全機能が正常動作することを確認する安定化バージョン。

**完了条件**: Rust テスト 2 件（ベース 3469 + 2 = 3471）
- `math_foundation_all_runes_stable`
- `math_docs_complete`

---

### v66.0.0 — Math & Science Foundation 宣言 ★クリーンアップ

**宣言文**:

> 「行列の次元は型で保証され、勾配は自動的に伝播する。
>  統計的検定は型安全に呼び出せ、時系列の周期は型パラメータに刻まれる。
>  数学的正確性が、AI パイプラインの信頼性を支える土台になった。
>
>  これが Favnir v66.0 — Math & Science Foundation の姿である。」

**完了条件**: `v66000_tests` 4 件（ベース 3471 + 4 = 3475）
- `cargo_toml_version_is_66_0_0`
- `changelog_has_v66_0_0`
- `milestone_has_math_science`
- `readme_mentions_math_science`

---

## Phase 2: v66.1〜v66.9 + v67.0 — AI-Native Stage Layer

**テーマ**: 「型システムで LLM・ベクトルを制御する」

数学基盤の上に、AI パイプラインの中核機能を構築する。
ベクトルの次元・LLM の出力スキーマ・埋め込みモデルの型を Favnir が保証する。

### v66.1.0 — Vector Stage Primitives

```favnir
// 1536 次元ベクトル — 次元違いはコンパイルエラー
public stage EmbedText: String -> Vec<Float>[1536] = |text| {
    Rune.openai.embed(model: "text-embedding-3-small", text: text)
}
public stage CosineSim: (Vec<Float>[1536], Vec<Float>[1536]) -> Float = |(a, b)| {
    Rune.linalg.cosine_similarity(a, b)
}
```

**完了条件**: `vec_stage_dim_type_check` / `vec_stage_cosine_sim`（3475 + 2 = 3477）

---

### v66.2.0 — LLM Extraction Stage（型安全 JSON 抽出）

LLM の出力を型安全なスキーマに変換。非構造データ → 型付きレコードを保証。

```favnir
schema InvoiceData {
    vendor: String,
    amount: Float,
    date: DateTime,
    line_items: List<LineItem>
}

public stage ExtractInvoice: String -> InvoiceData = |raw_text| {
    Rune.llm.extract(raw_text, schema: InvoiceData, model: "claude-sonnet-4-6")
    // スキーマ違反は実行時エラーではなく型エラー
}
```

**完了条件**: `llm_extract_typed_schema` / `llm_extract_schema_mismatch_error`（3477 + 2 = 3479）

---

### v66.3.0 — Embedding Pipeline Rune

ローカルモデル・OpenAI・Cohere 等を統一インターフェースで。

**完了条件**: `embed_rune_openai` / `embed_rune_local_model`（3479 + 2 = 3481）

---

### v66.4.0 — Vector DB Runes（Pinecone / pgvector / Weaviate）

```favnir
public stage StoreEmbeddings: List<(String, Vec<Float>[1536])> -> Unit = |pairs| {
    Rune.pinecone.upsert(pairs, namespace: "docs")
}
public stage SemanticSearch: Vec<Float>[1536] -> List<Document> = |query_vec| {
    Rune.pinecone.query(query_vec, top_k: 10, include_metadata: true)
}
```

**完了条件**: `vector_db_upsert_query` / `vector_db_type_safe_dim`（3481 + 2 = 3483）

---

### v66.5.0 — Streaming Inference Stage

リアルタイムスコアリングパイプライン。Kafka ストリーム + ML モデル推論を型安全に組み合わせる。

```favnir
pipeline RealtimeScoring {
    step "ingest"  = stream KafkaIngest
    step "embed"   = seq EmbedText after "ingest"
    step "score"   = seq MLScore after "embed"
    step "publish" = stream KafkaPublish after "score"
}
```

**完了条件**: `streaming_inference_pipeline` / `streaming_backpressure_ai`（3483 + 2 = 3485）

---

### v66.6.0 — Model Serving Rune（`Rune.serve`）

Favnir ステージをモデルサービングエンドポイントとして公開。

```favnir
// fav serve pipeline.fav --port 8080
// POST /score → InvoiceData
```

**完了条件**: `model_serve_endpoint_type` / `model_serve_schema_validation`（3485 + 2 = 3487）

---

### v66.7.0 — Feature Store Rune（`Rune.featurestore`）

型安全なフィーチャーエンジニアリング。フィーチャーの定義・バージョン管理・取得を型で保証。

**完了条件**: `feature_store_define_feature` / `feature_store_versioned_retrieval`（3487 + 2 = 3489）

---

### v66.8.0 — AI Pipeline Lint Rules（W055〜W059）

```
W055: 型なし LLM 出力をそのまま下流に流す（スキーマ抽出なし）
W056: 埋め込み次元の暗黙的キャスト（Vec<Float>[768] → Vec<Float>[1536]）
W057: ベクトル DB upsert なしの query（空 namespace への問い合わせ）
W058: ストリーミング推論ステージでのバッファなし直接処理
W059: LLM 呼び出しのリトライなし（外部 API の一時障害無対策）
```

**完了条件**: `lint_w055_untyped_llm_output` / `lint_w056_dim_implicit_cast`（3489 + 2 = 3491）

---

### v66.9.0 — 安定化・コードフリーズ（AI Stage Layer 前調整）

**完了条件**: `ai_stage_layer_all_stable` / `ai_rune_docs_complete`（3491 + 2 = 3493）

---

### v67.0.0 — AI-Native Stage Layer 宣言 ★クリーンアップ

**宣言文**:

> 「LLM の出力にスキーマが付き、ベクトルの次元が型で保証される。
>  埋め込みモデルの選択が型エラーを生まず、
>  リアルタイム推論パイプラインがバックプレッシャー制御下で動く。
>
>  これが Favnir v67.0 — AI-Native Stage Layer の姿である。」

**完了条件**: `v67000_tests` 4 件（3493 + 4 = 3497）

---

## Phase 3: v67.1〜v67.9 + v68.0 — Developer Intelligence

**テーマ**: 「パイプラインをデバッグし、AI が次の手を示す」

### v67.1.0 — `fav debug`（ステップ実行デバッガ）

```bash
$ fav debug pipeline.fav
[fav debug] v67.1.0 — ステップ実行モード
> run
[step 1/4] LoadCsv      → 1000 rows  (2ms)   ← 自動停止
> inspect row[0]        # レコード内容確認
> continue
[step 2/4] Transform    → 998 rows   (45ms)
> breakpoint "Validate" # 次のステップで停止
> continue
[step 3/4] Validate     → 998 rows   (8ms)   ← ブレークポイント停止
```

**完了条件**: `debug_step_execution` / `debug_breakpoint_stage`（3497 + 2 = 3499）

---

### v67.2.0 — Time-Travel Debugging（記録 & リプレイ）

パイプライン実行を記録して任意のステップに巻き戻す。
本番障害の再現に威力を発揮。

```bash
$ fav run pipeline.fav --record session.fav-trace
$ fav debug --replay session.fav-trace
[replay] Rewinding to step 2...
```

**完了条件**: `debug_record_replay` / `debug_rewind_to_step`（3499 + 2 = 3501）

---

### v67.3.0 — `fav viz`（パイプライン DAG 可視化）

```bash
$ fav viz pipeline.fav --ascii
LoadCsv ──► Transform ──► Validate ──┬──► InsertDB
                                      └──► SendSlack

$ fav viz pipeline.fav --format svg -o pipeline.svg
# ブラウザで開ける SVG を生成（ステージ別実行時間付き）
```

**完了条件**: `viz_ascii_dag` / `viz_svg_with_timing`（3501 + 2 = 3503）

---

### v67.4.0 — `fav suggest`（AI 最適化アドバイザー）

プロファイリング結果 + LLM でボトルネックの自動提案。

```bash
$ fav suggest pipeline.fav --from-profile fav-profile.json

Suggestion 1 [HIGH IMPACT]:
  Transform stage: 847ms
  collect { yield } パターン → List.map 変換で AOT 3× 高速化
  → fav fix --apply suggestion-1

Suggestion 2 [MED]:
  EmbedText を par で並列化 → スループット 4× 向上
```

**完了条件**: `suggest_from_profile` / `suggest_applies_fix`（3503 + 2 = 3505）

---

### v67.5.0 — `fav simulate`（合成データパイプラインテスト）

```favnir
// pipeline.test.fav
simulate SemanticSearch {
    input: Rune.gen.text(count: 100, seed: 42),
    assert: |result| { result.len() <= 10 && result[0].score > 0.8 }
}
```

**完了条件**: `simulate_pipeline_with_synthetic` / `simulate_assertion_failure`（3505 + 2 = 3507）

---

### v67.6.0 — Pipeline Property Testing（`Rune.proptest`）

```favnir
proptest stage Transform {
    forall x: Int where x > 0 { Transform(x) > 0 }
    forall x: Int where x == 0 { Transform(x) == 0 }
}
```

**完了条件**: `proptest_stage_invariant` / `proptest_counterexample_shrink`（3507 + 2 = 3509）

---

### v67.7.0 — Interactive Profiling（`fav profile --interactive`）

```bash
$ fav profile --interactive pipeline.fav
[hotspot] Transform: 847ms (72% of total)
  Drill down? [y/N]: y
  [line 12] collect { yield ... } — 承認? W041 を --allow で抑制しますか？
```

**完了条件**: `profile_interactive_hotspot` / `profile_interactive_drill`（3509 + 2 = 3511）

---

### v67.8.0 — Math-Aware Doc Generation（`fav doc --math`）

数学 Rune の関数ドキュメントに LaTeX 数式を埋め込む。

```favnir
/// Computes the gradient of `f` at `x`.
/// Formula: ∇f(x) = ∂f/∂x₁, ∂f/∂x₂, ..., ∂f/∂xₙ
/// ```
/// bind g = Rune.autodiff.grad(|x| { x * x }, 3.0)
/// // g == 6.0
/// ```
```

**完了条件**: `doc_math_latex_rendered` / `doc_math_example_compiles`（3511 + 2 = 3513）

---

### v67.9.0 — 安定化・コードフリーズ（Developer Intelligence 前調整）

**完了条件**: `dev_intelligence_all_stable` / `debug_viz_suggest_docs_complete`（3513 + 2 = 3515）

---

### v68.0.0 — Developer Intelligence 宣言 ★クリーンアップ

**宣言文**:

> 「ステップ実行デバッガが、AI パイプラインの内部を露わにする。
>  時間を遡って本番障害を再現し、DAG 可視化が依存関係を一目で示す。
>  AI アドバイザーがプロファイリングデータを読み、次の最適化を提案する。
>
>  これが Favnir v68.0 — Developer Intelligence の姿である。」

**完了条件**: `v68000_tests` 4 件（3515 + 4 = 3519）

---

## Phase 4: v68.1〜v68.9 + v69.0 — Distributed Favnir

**テーマ**: 「型安全な AI パイプラインを、クラスタ規模で動かす」

### v68.1.0 — Multi-Node `par`（分散並列実行）

```favnir
// par が単一マシン → 複数マシンに拡張
pipeline DistributedEmbedding {
    step "load"   = seq LoadDocs
    step "embed"  = par [EmbedNode1, EmbedNode2, EmbedNode3] after "load"
    step "store"  = seq VectorStore after "embed"
}
// fav run pipeline.fav --cluster workers.yaml
```

**完了条件**: `distributed_par_multi_node` / `distributed_work_rebalance`（3519 + 2 = 3521）

---

### v68.2.0 — Pipeline Checkpointing（耐障害性・再開）

```bash
$ fav run pipeline.fav --checkpoint ./checkpoints/
# 途中で失敗
$ fav run pipeline.fav --resume ./checkpoints/step-3.ckpt
# step 3 から再開
```

**完了条件**: `checkpoint_save_restore` / `checkpoint_resume_mid_pipeline`（3521 + 2 = 3523）

---

### v68.3.0 — Kubernetes-Native Orchestration

```yaml
# fav deploy --target kubernetes で生成
apiVersion: favnir.dev/v1
kind: Pipeline
metadata: { name: semantic-search }
spec:
  stages:
    - name: embed
      replicas: 4
      resources: { memory: "2Gi", gpu: "1" }
```

**完了条件**: `k8s_pipeline_manifest_gen` / `k8s_stage_replicas`（3523 + 2 = 3525）

---

### v68.4.0 — Stage Retry Policies（型安全エラー回復）

```favnir
pipeline ResilientPipeline {
    step "call-llm" = seq CallLLM with {
        retry: ExponentialBackoff(max: 3, base_ms: 500),
        on_failure: Fallback(CachedResponse),
        timeout_ms: 5000
    }
}
```

**完了条件**: `retry_exponential_backoff` / `retry_fallback_stage`（3525 + 2 = 3527）

---

### v68.5.0 — Distributed Incremental Cache

複数ワーカー間でコンパイルキャッシュを共有。同一ステージの重複計算を排除。

**完了条件**: `distributed_cache_hit_across_workers` / `distributed_cache_invalidation`（3527 + 2 = 3529）

---

### v68.6.0 — Cost-Aware Scheduling（AI パイプラインコスト最適化）

```bash
$ fav cost-estimate pipeline.fav --provider aws --scale 1M-rows
Estimated cost: $2.34
  LLM calls (GPT-4o): $1.80 (77%)
  Vector DB queries:  $0.42 (18%)
  Compute (ECS):      $0.12 (5%)
Optimization: バッチサイズを 10 → 50 に増やすと $0.90 削減可能
```

**完了条件**: `cost_estimate_ai_pipeline` / `cost_optimize_batch_size`（3529 + 2 = 3531）

---

### v68.7.0 — Multi-Cloud AI Routing（LLM/VectorDB プロバイダー切り替え）

```favnir
// fav.toml で本番・開発環境を切り替え
[ai]
llm_provider = "anthropic"       # prod
embed_provider = "openai"        # prod
vector_db = "pinecone"           # prod

[ai.dev]
llm_provider = "ollama-local"    # dev（コスト無料）
embed_provider = "ollama-local"
vector_db = "qdrant-local"
```

**完了条件**: `multi_cloud_ai_routing` / `ai_provider_local_fallback`（3531 + 2 = 3533）

---

### v68.8.0 — Distributed Observability（AI パイプライン分散トレーシング）

分散実行中のパイプラインを OpenTelemetry でエンドツーエンドトレース。
LLM 呼び出し・ベクトル DB クエリのレイテンシを統合ダッシュボードで可視化。

**完了条件**: `distributed_otel_trace` / `distributed_latency_breakdown`（3533 + 2 = 3535）

---

### v68.9.0 — 安定化・コードフリーズ（Distributed Favnir 前調整）

**完了条件**: `distributed_all_stable` / `distributed_docs_complete`（3535 + 2 = 3537）

---

### v69.0.0 — Distributed Favnir 宣言 ★クリーンアップ

**宣言文**:

> 「`par` がクラスタを越え、チェックポイントが失敗を無効にする。
>  Kubernetes が AI ステージのスケールを決め、
>  コスト見積もりが LLM 呼び出しの予算を守る。
>  型安全な AI パイプラインが、大規模でも壊れない。
>
>  これが Favnir v69.0 — Distributed Favnir の姿である。」

**完了条件**: `v69000_tests` 4 件（3537 + 4 = 3541）

---

## Phase 5: v69.1〜v69.9 + v70.0 — Intelligent ETL 1.0 宣言

**テーマ**: 統合・最終調整・宣言

### v69.1〜v69.9 — Integration Sprint（詳細は着手時に確定）

候補:
- `v69.1`: E2E デモ（CSV → Embed → VectorDB → Semantic Search）
- `v69.2`: Playground WASM 版 AI パイプライン（ブラウザで動作）
- `v69.3`: ドキュメントサイト「Intelligent ETL ガイド」
- `v69.4`: `fav migrate` AI パイプライン版（旧 ETL → AI ETL 自動変換）
- `v69.5〜v69.9`: 安定化・細部調整

### v70.0.0 — Intelligent ETL 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

**完了条件**: `v70000_tests` 4 件（3545 + 4 = 3549）
- `cargo_toml_version_is_70_0_0`
- `changelog_has_v70_0_0`
- `milestone_has_intelligent_etl`
- `readme_mentions_intelligent_etl`

---

## テスト数推移（計画値）

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v65.0.0（ベース） | 3453 | — | Performance 1.0 宣言 |
| v65.1〜v65.9 | 3453 + 18 = 3471 | +18 | Math Rune 群（各 +2） |
| v66.0.0 | 3471 + 4 = 3475 | +4 | Math & Science 宣言 |
| v66.1〜v66.9 | 3475 + 18 = 3493 | +18 | AI Stage Layer（各 +2） |
| v67.0.0 | 3493 + 4 = 3497 | +4 | AI-Native Stage Layer 宣言 |
| v67.1〜v67.9 | 3497 + 18 = 3515 | +18 | Developer Intelligence（各 +2） |
| v68.0.0 | 3515 + 4 = 3519 | +4 | Developer Intelligence 宣言 |
| v68.1〜v68.9 | 3519 + 18 = 3537 | +18 | Distributed Favnir（各 +2） |
| v69.0.0 | 3537 + 4 = 3541 | +4 | Distributed Favnir 宣言 |
| v69.1〜v69.9 | 3541 + 4 以上 | 未定 | Integration Sprint（v69.1: +2, v69.2: +2 確定） |
| v70.0.0 | ≥ 3549 | +4 | Intelligent ETL 1.0 宣言 |

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 現行マスター: `versions/roadmap/roadmap-v65.1-v70.0.md`（本文書）
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
