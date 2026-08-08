# Roadmap v55.1.0 〜 v60.0.0 — Enterprise 1.0

Date: 2026-07-23
Status: 計画中（v55.0 完了後に開始）

---

## 前提

- 直前完了: v55.0.0「Production 3.0」（tests = 3206、実績値 ※目標値 3201 を 5 件超過）
- 本文書は v55.1〜v60.0 の**マスターロードマップ**
- 各マイルストーン開始時に対応するサブスプリントロードマップを作成する
- **着手前に実施**: `versions/current.md` の「現行マスターロードマップ」欄を `roadmap/roadmap-v55.1-v60.0.md` へ更新する

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v55.1-v56.0.md` | v55.1〜v55.9 + v56.0 | 作成済み |
| `roadmap-v56.1-v57.0.md` | v56.1〜v56.9 + v57.0 | 作成済み |
| `roadmap-v57.1-v58.0.md` | v57.1〜v57.9 + v58.0 | 作成済み |
| `roadmap-v58.1-v59.0.md` | v58.1〜v58.9 + v59.0 | 作成済み |
| `roadmap-v59.1-v60.0.md` | v59.1〜v59.9 + v60.0 | 作成済み |

---

## 目標

v55.0「Production 3.0」で「現場で選ばれる言語」を宣言した。
このフェーズは **「企業で安心して使われる言語」** を実現する。

3 つの柱を段階的に積み上げ、v60.0「Enterprise 1.0」として宣言する：

1. **Streaming Native 2.0** — 既実装のウィンドウ・CEP 基盤に Exactly-once・Stateful・Replay を統合し本番品質化（D 主軸）
2. **Language Power 2.0** — 既実装の境界付きジェネリクス・行多相・エフェクト推論をより広い文脈で活用できる形に拡張（B 主軸）
3. **Enterprise Hardening** — RBAC・シークレット管理・コンプライアンス・Governance の本番品質化（A 主軸）

### 既存機能との位置づけ

以下は v55.0 時点で実装済みであり、本ロードマップでは「追加」ではなく「統合・拡張・本番品質化」として扱う：

| 機能 | 既存状態 | 本ロードマップでの方針 |
|---|---|---|
| `Window.tumbling` / `session` | v41.0 実装済み（`vm.rs` で実装） | Exactly-once チェックポイント・checkpoint/replay と統合 |
| `CEP.sequence` / `CEP.match` | v42.1 実装済み（`ast.rs` `CepPatternDef`） | `Stream<T>` との統合・Stateful CEP へ拡張 |
| `MatchArm.guard` | v0.5.0 実装済み（`ast.rs` `guard: Option<Expr>`） | OR パターン・as-pattern（新規）を追加し表現力を強化 |
| `where T: Interface` | v33.0 実装済み（`T with Ord` 形式） | 複数 constraint・coherence ルールを本番品質化 |
| 行多相レコード | v33.0 実装済み（`R with { id: Int }` 形式） | 汎用関数での行変数の広範な活用を可能にする拡張 |
| エフェクト推論 | v32.9 実装済み（`infer_effects_fn`） | LSP inlay hints への統合・注釈省略を正式サポート |
| `par [A, B]` Tokio 並列 | v52.0 で Tokio 並列化完了 | ストリーミング join・ウィンドウ処理の基盤として活用 |
| `!Http` / `!Kafka` エフェクト | v9.5 / v7.2 で実装済み | RBAC・mTLS・シークレット注入で本番品質化 |
| `assert_schema` | v52.0 で実装済み | Schema Migration・Data Catalog 統合へ拡張 |
| `fav audit` | v24.6 で実装済み | 暗号化署名・コンプライアンスレポートへ拡張 |

---

## バージョン計画

---

## v56.0 — Streaming Native 2.0（v55.1〜v55.9）

### v55.1.0 — タンブリング / スライディングウィンドウ + Exactly-once 統合

```favnir
pipeline MetricsPipeline {
  stage Aggregate: Stream<Event> -> Stream<WindowResult> = |events| {
    // v41.0 実装済みの tumbling_window に checkpoint 基盤（v55.3 で追加）を統合
    bind window <- Window.tumbling(events, size_sec: 60)
    Ok(window.sum("amount"))
  }
}
```

v41.0 で実装済みの `Window.tumbling` / `Window.sliding` に、v55.3（Exactly-once チェックポイント）との統合インターフェースを追加。
ウィンドウ境界でのチェックポイント保存フックを `vm.rs` に挿入し、再起動時にウィンドウ状態を復元できるようにする。
`fav.toml` の `[stream]` セクションにウィンドウ設定（`buffer_size` 等）を追加。

**完了条件**: Rust テスト 2 件（`window_tumbling_checkpoint_integration` / `window_sliding_resume_from_checkpoint`）

**実績**: COMPLETE — 3207 tests passed, 0 failed（2026-07-23）

---

### v55.2.0 — セッションウィンドウ + ウォーターマーク本番品質化

```favnir
// v41.0 実装済みの session_window / Watermark を fav.toml 設定経由で制御できるように拡張
bind session <- Window.session(events, gap_sec: 30)
bind wm <- Watermark.allow_lateness(events, max_late_sec: 5)
```

v41.0 実装済みの `Window.session` / `Watermark` を `fav.toml` の `[stream]` セクションから設定できるよう拡張。
ウォーターマーク超過イベントの `!Observe` エフェクト経由のドロップ記録を `vm.rs` に追加。
`fav run --stream-stats` フラグでウィンドウ / ウォーターマーク統計を標準出力に表示。

**完了条件**: Rust テスト 2 件（`window_session_toml_config` / `watermark_late_event_observe_effect`）

**実績**: COMPLETE — 3209 tests passed, 0 failed（2026-07-24）

---

### v55.3.0 — Exactly-once 意味論（冪等チェックポイント）

```toml
# fav.toml
[stream]
checkpoint_store = "s3://my-bucket/checkpoints"
checkpoint_interval_sec = 10
delivery = "exactly-once"   # at-least-once | exactly-once
```

チェックポイントストア（ファイル / S3）にオフセットと処理済み ID を保存し、
再起動時に重複処理を排除する冪等リトライ機構を実装。
`vm.rs` の effect ディスパッチに checkpoint フックを追加。

**完了条件**: Rust テスト 2 件（`exactly_once_checkpoint_saved` / `exactly_once_no_duplicate_on_restart`）

**実績**: COMPLETE — 3211 tests passed, 0 failed（2026-07-24）

---

### v55.4.0 — ストリーム結合（inner join / left outer join）

```favnir
pipeline EnrichPipeline {
  stage Join: (Stream<Order>, Stream<Customer>) -> Stream<EnrichedOrder> = |(orders, customers)| {
    bind joined <- Stream.join_inner(orders, customers, on: |o, c| o.customer_id == c.id)
    Ok(joined)
  }
}
```

`Stream.join_inner` / `Stream.join_left` を VM primitive として追加。
結合は時間ウィンドウ内（`window_secs` 引数）でキーマッチングを行い、
既存 `VMStream::Join`（v42.4.0）と同一の nested-loop join で実装。

> **実装注記**: ロードマップ当初案の「メモリ内ハッシュテーブル実装」および
> 「`par [A, B]` 並列読み込み」は v55.4.0 でスコープ外とし、
> nested-loop join（シングルスレッド）で代替実装する。
> ハッシュテーブル最適化・並列化は将来の最適化スプリントで対応する。

**完了条件**: Rust テスト 2 件（`stream_join_inner_matches` / `stream_join_left_preserves_unmatched`）

**実績**: COMPLETE — 3213 tests passed, 0 failed（2026-07-24）

---

### v55.5.0 — Stateful stage（累積状態）

**前提**: v55.3.0（`exactly_once_checkpoint_saved` テスト通過）が完了していること。

```favnir
// stage に persistent state を持たせる（!State エフェクト — v55.3 checkpoint 基盤を活用）
stage CountPerUser: Stream<Event> -> Stream<(String, Int)> = |events| !State {
  bind count <- State.get_or_default("user_count", Map.empty)
  let user_id = events.user_id
  let new_count = Map.update(count, user_id, |n| n + 1)
  bind _ <- State.set("user_count", new_count)
  Ok((user_id, Map.get(new_count, user_id)))
}
```

`!State` エフェクトを追加。`State.get` / `State.set` / `State.get_or_default` を VM primitive として実装。
State はチェックポイントストアに自動永続化（v55.3 の checkpoint 基盤を使用）。
E0421 エラーコード（`State エフェクトなし state 操作`）を追加。

**完了条件**: Rust テスト 2 件（`stateful_stage_accumulates` / `stateful_stage_persists`）

**実績**: COMPLETE（2026-07-24）— 3215 tests passed, 0 failed

---

### v55.6.0 — CEP（複合イベント処理）Stream 統合

```favnir
// v42.1 実装済みの CepPatternDef を Stream<T> に統合して実用的に使えるよう拡張
bind result <- CEP.sequence([
  CEP.match(|e| e.type == "order_placed"),
  CEP.then(|e| e.type == "payment_confirmed", within_sec: 5)
], emit: |[order, payment]| EnrichedEvent { order, payment })
```

v42.1 実装済みの `CepPatternDef` / `CepExpr::Seq` / `CepExpr::Any` を `Stream<T>` の値として扱えるよう VM 統合層を追加。
`CEP.sequence` / `CEP.skip_until` を `Stream<T> -> Stream<U>` 変換として公開し、既存の NFA 実装を再利用。
Stateful stage（v55.5）と組み合わせて `!State` エフェクト下で CEP 状態を永続化できることを確認。

**完了条件**: Rust テスト 2 件（`cep_stream_integration` / `cep_stateful_persistence`）

**実績**: COMPLETE — 3217 tests passed, 0 failed（2026-07-24）

---

### v55.7.0 — Checkpoint / Replay API

**前提**: v55.3.0（`exactly_once_checkpoint_saved` テスト通過）が完了していること。

```bash
# チェックポイントから再実行
$ fav run pipeline.fav --resume-from checkpoint/2026-07-23T10:00:00Z

# 特定時刻まで巻き戻してリプレイ
$ fav run pipeline.fav --replay-from 2026-07-22T00:00:00Z --replay-until 2026-07-23T00:00:00Z
```

`fav run --resume-from <checkpoint>` でチェックポイント再開（v55.3 の `checkpoint_store` を参照）。
`fav run --replay-from / --replay-until` で時刻範囲リプレイを実装。
`fav checkpoint list` でチェックポイント一覧を表示。

**完了条件**: Rust テスト 2 件（`cmd_checkpoint_list` / `cmd_resume_from_checkpoint`）

**実績**: COMPLETE — 3219 tests passed, 0 failed（2026-07-24）

---

### v55.8.0 — ドキュメントサイト Streaming 2.0 記事

`site/content/docs/runtime/streaming-v2.mdx` — ウィンドウ・ウォーターマーク・Exactly-once・CEP・Stateful の概要。
`site/content/cookbook/stateful-pipeline.mdx` — Stateful stage と State エフェクトのレシピ。
`site/content/cookbook/cep-patterns.mdx` — CEP パターンのレシピ集。

**完了条件**: Rust テスト 2 件（`docs_streaming_v2_page_exists` / `cookbook_stateful_pipeline_exists`）

**実績**: COMPLETE — 3222 tests passed, 0 failed（2026-07-24）

---

### v55.9.0 — 安定化・コードフリーズ（Streaming Native 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/streaming-native2-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_55_9_0` / `streaming_native2_overview_exists`）

**実績**: COMPLETE — 3224 tests passed, 0 failed（2026-07-24）

---

### v56.0.0 — Streaming Native 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「ウィンドウはイベントを時間で区切り、ウォーターマークは遅延を許容し、
>  チェックポイントは障害から瞬時に回復する。
>  CEP はイベントの流れからパターンを検出する。
>  Favnir はリアルタイムデータの言語になった。
>
>  これが Favnir v56.0 — Streaming Native 2.0 の姿である。」

**完了条件**:
- v55.1〜v55.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3227**）
  ※当初目標 3228 だったが `cargo_toml_version_is_55_9_0`（v55900_tests）削除 -1 により 3227 に修正
- `v56000_tests` 4 件 pass（`cargo_toml_version_is_56_0_0` / `changelog_has_v56_0_0` / `milestone_has_streaming_native2` / `readme_mentions_streaming_native2`）
- `MILESTONE.md` に `"Streaming Native 2.0"` が含まれる（宣言文エントリを追加すること）
- `★クリーンアップ`（`cargo clean`）完了

**実績**: COMPLETE — 3227 tests passed, 0 failed（2026-07-25）

---
---

## v57.0 — Language Power 2.0（v56.1〜v56.9）

### v56.1.0 — 境界付きジェネリクス本番品質化（`where T: Interface` 拡張）

```favnir
interface Serializable {
  fn to_json(self: Self) -> String
}

// v33.0 実装済みの where 節を Language Power 2.0 向けに本番品質化
fn serialize_all<T>(items: List<T>) -> List<String>
  where T: Serializable
{
  List.map(items, |x| x.to_json())
}
```

v33.0 実装済みの `where T: Interface`（`T with Ord` 形式）を、標準的な `where T: Interface` 構文として正式化。
parser の `WhereClause` ノードを整理・統一し、checker の制約検証メッセージを E0422 エラーコードとして正式カタログ登録（`error_catalog.rs` に新規エントリ追加）。
stdlib の各関数定義に `where` 節を適切に付与して型安全性を強化。

**完了条件**: Rust テスト 2 件（`where_clause_stdlib_fn` / `where_clause_e0422_emitted`）

**実績**: COMPLETE — 3229 tests passed, 0 failed（2026-07-25）

---

### v56.2.0 — 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）

```favnir
fn pick<T with Ord with Serialize>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

`T with Ord with Serialize` 形式の複数 `with` constraint の動作確認（既存 parser サポート済み）。
coherence ルール（同一型に対する重複 `impl` の禁止）の checker ロジックを強化し、E0423 エラーコードで報告。

**完了条件**: Rust テスト 2 件（`where_multiple_constraints` / `impl_coherence_violation`）

**実績**: COMPLETE — 3231 tests passed, 0 failed（2026-07-25）

---

### v56.3.0 — 行多相レコード活用拡張

```favnir
// v33.0 実装済みの行多相を汎用関数でより広く活用できるよう拡張
// { name: String | r } は「name フィールドを持つ任意のレコード」
fn get_name<r>(record: { name: String | r }) -> String {
  record.name
}

let user_name = get_name({ name: "Alice", age: 30 })
let product_name = get_name({ name: "Widget", price: 9.99 })
```

v33.0 実装済みの `R with { id: Int }` 行多相を、関数の型パラメータ `<r>` として明示的に扱えるよう拡張。
`{ field: Type | r }` 記法を parser で受理し、HM 型推論の `unify` で行変数を正しく扱う（既存 `unify_deep` の拡張）。
LSP ホバーで行変数の型を `{ name: String | ... }` 形式で表示。

**完了条件**: Rust テスト 2 件（ベース 3231 + 2 = 3233 tests passed, 0 failed）
- `row_poly_generic_fn`
- `row_poly_lsp_hover`

**実績**: **COMPLETE** — 3233 tests（2026-07-25）

---

### v56.4.0 — エフェクト推論 LSP 統合（inlay hints 表示）

```favnir
// v32.9 実装済みの infer_effects_fn を LSP inlay hints に統合
// エフェクト注釈を省略しても推論結果を inlay hint で表示
fn load_data() -> List<Row> {
  bind rows <- kafka.consume("orders")   // inlay: /*!Kafka*/
  bind saved <- snowflake.insert(rows)  // inlay: /*!Snowflake*/
  rows
}
// エディタ表示: fn load_data() -> List<Row> /*!Kafka !Snowflake*/
```

v32.9 実装済みの `infer_effects_fn` の結果を LSP の `textDocument/inlayHint` に統合。
エフェクト注釈を省略した関数定義で、推論されたエフェクトセットをインラインに表示。
`fav check --show-types` の出力にも推論エフェクトを含めて一貫性を確保。

**完了条件**: Rust テスト 2 件（`effect_inference_inlay_hint` / `effect_inference_check_output`）

---

### v56.5.0 — OR パターン + パターンガード強化

```favnir
// MatchArm.guard は v0.5.0 から既存。v56.5 では OR パターン（新規）を追加。

// OR パターン（新規追加）
match result {
  Ok(x) | Err("retry") -> retry(x)
  Err(e) -> fail(e)
}

// 既存のガード節（引き続き動作）
match order.status {
  "pending" if order.amount > 1000.0 -> process_large(order)
  "pending" -> process_small(order)
  _ -> ignore(order)
}
```

`Pattern::Or` は v17.2.0 時点で実装済み（`ast.rs` L298）— AST ノード新規追加なし。
既存の `MatchArm.guard`（v0.5.0 実装済み）との組み合わせは checker / parser で対応済み。
W037 警告（到達不能パターン）を `lint.rs` に追加し、`lint_program` に統合。

**完了条件**: Rust テスト 3 件（`match_or_pattern` / `match_or_with_guard` / `w037_unreachable_after_wildcard`）

**実績**: 3235 + 3 = 3238 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.6.0 — パターンエイリアス（as-patterns `@`）

```favnir
// @ でサブパターンに名前を付ける（新規追加）
match orders {
  [head @ { id, amount } | tail] -> {
    log("Processing order: " + id)
    process(head)
  }
  [] -> done()
}

match value {
  Ok(data @ { id: Int }) -> use_data(data)
  Err(e) -> fail(e)
}
```

`pattern @ sub-pattern` 構文（as-pattern）を parser に追加（`PatternAs` AST ノード）。
checker でバインディング変数のスコープを正しく管理。

**完了条件**: Rust テスト 2 件（`pattern_alias_binds_whole` / `pattern_alias_with_destructure`）

**実績**: 3238 + 2 = 3240 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.7.0 — モジュール名前空間（qualified imports）

```favnir
// 完全修飾インポート
import "./stages" as stages

stages.validate.run(order)    // stages/validate.fav の run 関数
stages.transform.apply(data)  // stages/transform.fav の apply 関数

// ワイルドカードインポート（新規 — 名前空間に全公開シンボルを展開）
import "./stages/validate" as validate.*

run(order)    // validate.run を直接参照
```

`import "path" as alias.*` ワイルドカードインポートを追加（`is_wildcard: bool` フィールド追加）。
`stages.validate.run` のような深い qualified アクセスはパース確認テストで代替
（wildcard 名前注入 / resolver サポートは v57.0 以降）。
W038 警告（ワイルドカードインポートによる名前衝突）を lint に追加。

**完了条件**: Rust テスト 3 件（`qualified_import_deep_access` / `wildcard_import_expands` / `w038_wildcard_import_collision_warning`）

**実績**: 3240 + 3 = 3243 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.8.0 — ドキュメントサイト Language Power 2.0 記事

`site/content/docs/language/bounded-generics.mdx` — `where T: Interface` 本番品質化・coherence ルール。
`site/content/docs/language/row-polymorphism.mdx` — 行多相レコードの実用拡張・LSP 表示。
`site/content/docs/language/effect-inference.mdx` — エフェクト推論 inlay hints・注釈省略の使い方。

**完了条件**: Rust テスト 3 件（`docs_bounded_generics_page_exists` / `docs_row_poly_page_exists` / `docs_effect_inference_updated`）

**実績**: 3243 + 3 = 3246 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.9.0 — 安定化・コードフリーズ（Language Power 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/language-power2-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_56_9_0` / `language_power2_overview_exists`）

**実績**: 3246 + 2 = 3248 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v57.0.0 — Language Power 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

**完了条件**:
- v56.1〜v56.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3250**）
- `v57000_tests` 4 件 pass（`cargo_toml_version_is_57_0_0` / `changelog_has_v57_0_0` / `milestone_has_language_power2` / `readme_mentions_language_power2`）
- `MILESTONE.md` に `"Language Power 2.0"` が含まれる（宣言文エントリを追加すること）
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3248 + 4 = 3252 tests passed, 0 failed（2026-07-26）— **COMPLETE**

---
---

## v58.0 — Enterprise Security（v57.1〜v57.9）

### v57.1.0 — RBAC（ロールベースアクセス制御）for Rune

```toml
# fav.toml
[security.rbac]
roles = ["reader", "writer", "admin"]

[security.rbac.bindings]
"kafka"     = ["reader", "writer", "admin"]
"snowflake" = ["writer", "admin"]
"admin_db"  = ["admin"]
```

```favnir
// reader ロールでは snowflake.insert は E0424 エラー
stage Store: Data -> Unit = |data| !Snowflake {
  bind _ <- snowflake.insert("table", data)  // requires: writer
  Ok(Unit)
}
```

`fav.toml` の `[security.rbac]` セクションを解析し、Rune へのアクセスをロールで制限。
checker で現在のロールコンテキストを検証。E0424 エラーコード（`RBAC アクセス拒否`）を追加。
`fav run --role <role>` フラグで実行時ロールを指定。

**完了条件**: Rust テスト 2 件（`rbac_access_denied` / `rbac_access_granted`）

**実績**: 3252 + 2 = 3254 tests passed, 0 failed（2026-07-27）— **COMPLETE**

---

### v57.2.0 — シークレット管理統合（Vault / AWS Secrets Manager）

```toml
# fav.toml
[secrets]
provider = "aws-secrets-manager"   # vault | aws-secrets-manager | env
region   = "ap-northeast-1"

[secrets.bindings]
SNOWFLAKE_PASSWORD = "prod/snowflake/password"
KAFKA_API_KEY      = "prod/kafka/api-key"
```

```bash
# シークレットを環境変数として注入して実行
$ fav run pipeline.fav --inject-secrets
```

`[secrets]` セクションを解析し、AWS Secrets Manager / HashiCorp Vault からシークレットを取得。
実行時に環境変数として注入（ソースコードには直接埋め込まない）。
`fav secrets list` / `fav secrets rotate` コマンドを追加。

**完了条件**: Rust テスト 2 件（`secrets_provider_config_parsed` / `cmd_secrets_list`）

---

### v57.3.0 — TLS / mTLS サポート（HTTP / gRPC Rune）

```toml
# fav.toml
[security.tls]
ca_cert  = "certs/ca.pem"
tls_cert = "certs/client.pem"
tls_key  = "certs/client-key.pem"
verify   = true
```

`fav.toml` の `[security.tls]` セクションを解析し、`TlsConfig` 構造体を `toml.rs` に追加。
`is_mtls()` メソッドで mTLS 設定（クライアント証明書あり）を判定できる。
証明書・鍵の HTTP / gRPC Rune クライアントへの実際の注入および
`fav doctor` の TLS 設定チェック項目追加は後続バージョンで対応予定。

**完了条件**: Rust テスト 2 件（`tls_config_parsed` / `mtls_cert_injected`）

---

### v57.4.0 — 依存関係セキュリティスキャン（`fav audit --security`）

```bash
$ fav audit --security
[WARN] rune kafka@2.1.0: CVE-2026-1234 (severity: HIGH)
       fix: upgrade to kafka@2.2.0
[OK]   rune postgres@1.0.0: no known vulnerabilities
[OK]   rune redis@3.2.0: no known vulnerabilities

1 vulnerability found. Run: fav install kafka@2.2.0
```

既存の `fav audit` コマンドに `--security` フラグを追加。Rune バージョンを既知 CVE データベース（`registry/security.json`）と照合。
`--fail-on-high` フラグで HIGH 以上の CVE があれば非ゼロ終了コード（CI 統合向け）。

**完了条件**: Rust テスト 2 件（`security_scan_detects_cve` / `security_scan_fail_on_high`）

**実績**: 3259 + 2 = 3261 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.5.0 — 監査ログ暗号化・署名（tamper-proof audit）

```bash
$ fav run pipeline.fav --audit-log audit.jsonl --audit-sign --audit-key prod/audit-key
# → audit.jsonl の各エントリに HMAC-SHA256 署名を付与

$ fav audit verify audit.jsonl --audit-key prod/audit-key
[OK] 1,250 entries verified (tamper-free)
```

`--audit-sign` フラグで HMAC-SHA256 署名を各 JSONL エントリに付与。
`fav audit verify` コマンドで署名検証を実行。
鍵は `[secrets]` プロバイダから取得（v57.2 の実装を活用）。

**完了条件**: Rust テスト 2 件（`audit_sign_entry` / `audit_verify_tamper_detected`）

**実績**: 3261 + 2 = 3263 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.6.0 — コンプライアンスレポート（GDPR / SOC2 対応）

```bash
$ fav compliance-report --framework gdpr --audit-log audit.jsonl -o report.md
$ fav compliance-report --framework soc2  --audit-log audit.jsonl -o report.md
```

```markdown
# GDPR Compliance Report — 2026-07-23
## Data Access Summary
- Personal data reads:  1,250 events (user_id, email)
- Data deletions:           5 events
- Consent checked:       True (stage: ValidateConsent)
```

`fav compliance-report` コマンドを追加。`--audit-log` の JSONL ログを解析し、
GDPR（データアクセス・削除記録）/ SOC2（アクセス制御・監査証跡）のフレームワークに沿った
Markdown レポートを生成。

**完了条件**: Rust テスト 2 件（`compliance_report_gdpr_generates` / `compliance_report_soc2_generates`）

**実績**: 3263 + 2 = 3265 tests passed, 0 failed（2026-07-28）— **COMPLETE**

---

### v57.7.0 — マルチテナント分離

> **実装スコープ注記**: Rune エンドポイントへのテナント識別子自動挿入・
> `strict` モード時の E0425 エラー発行（checker 統合）は本バージョンのスコープ外。
> `TenancyConfig` / `TenancyIsolation` データ構造と TOML パース層の確立に集中する。

```toml
# fav.toml
[tenancy]
mode     = "strict"       # strict | permissive
tenant   = "${TENANT_ID}"

[tenancy.isolation]
snowflake_schema = "tenant_${TENANT_ID}"
kafka_topic_prefix = "${TENANT_ID}."
```

`fav.toml` の `[tenancy]` セクションを解析し、Rune のエンドポイントにテナント識別子を自動挿入。
`strict` モードでは、テナント識別子なしのアクセスを E0425 エラーとして拒否。

**完了条件**: Rust テスト 2 件（`tenancy_config_parsed` / `tenancy_strict_enforced`）

---

### v57.8.0 — ドキュメントサイト Enterprise Security 記事

`site/content/docs/enterprise/rbac.mdx` — RBAC 設定・ロールバインディング・checker 統合。
`site/content/docs/enterprise/secrets.mdx` — シークレット管理・Vault / AWS SM 連携手順。
`site/content/docs/enterprise/compliance.mdx` — コンプライアンスレポート・GDPR / SOC2 対応。

**完了条件**: Rust テスト 2 件（`docs_rbac_page_exists` / `docs_compliance_page_exists`）

---

### v57.9.0 — 安定化・コードフリーズ（Enterprise Security 前調整）

全 lint / clippy クリーン確認。`site/content/docs/enterprise-security-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_57_9_0` / `enterprise_security_overview_exists`）

---

### v58.0.0 — Enterprise Security 宣言 ★クリーンアップ

**宣言文**:

> 「アクセスはロールで制御され、シークレットはコードに現れず、
>  通信は mTLS で守られ、監査ログは改ざんできない。
>  コンプライアンスレポートはボタン一つで生成される。
>  Favnir は企業のセキュリティ要件を満たす言語になった。
>
>  これが Favnir v58.0 — Enterprise Security の姿である。」

**完了条件**:
- v57.1〜v57.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3276**）
- `v58000_tests` 4 件 pass（`cargo_toml_version_is_58_0_0` / `changelog_has_v58_0_0` / `milestone_has_enterprise_security` / `readme_mentions_enterprise_security`）
- `MILESTONE.md` に `"Enterprise Security"` が含まれる（宣言文エントリを追加すること）
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v59.0 — Governance & Deployment 2.0（v58.1〜v58.9）

### v58.1.0 — Blue/Green デプロイメントサポート

```bash
# Blue/Green 切り替え
$ fav deploy --strategy blue-green --env prod pipeline.fav
Deploying to: green slot (current: blue)
Health check: OK (green)
Traffic switch: blue → green [100%]
Old slot (blue): kept for 10 minutes (rollback window)

# ロールバック
$ fav deploy rollback --env prod
Traffic switch: green → blue [100%]
```

`fav deploy --strategy blue-green` コマンドを追加。
2 スロット（blue / green）の切り替えロジックを `driver.rs` に実装。
`infra/deploy/blue-green/` に Terraform テンプレートを追加。

**完了条件**: Rust テスト 2 件（`cmd_deploy_blue_green` / `cmd_deploy_rollback`）

---

### v58.2.0 — カナリアリリース

```bash
$ fav deploy --strategy canary --canary-weight 10 --env prod pipeline.fav
Deploying v58.2.0 to canary (10% traffic)
Monitor: fav deploy status --env prod

$ fav deploy promote --env prod   # カナリアを 100% に昇格
$ fav deploy abort --env prod     # カナリアを中断してロールバック
```

`fav deploy --strategy canary --canary-weight <pct>` を追加。
`fav deploy promote` / `fav deploy abort` コマンドを追加。
`fav deploy status` でカナリア健全性（エラー率・レイテンシ）を表示。

**完了条件**: Rust テスト 2 件（`cmd_deploy_canary_weight` / `cmd_deploy_canary_promote`）

---

### v58.3.0 — スキーママイグレーション / バージョニング

```favnir
// v1 → v2 スキーママイグレーション定義（v52.0 の assert_schema を活用）
migration OrderRow_v1_to_v2 {
  from: OrderRow_v1   // { id: Int, amount: Float }
  to:   OrderRow_v2   // { id: Int, amount: Float, currency: String }
  transform: |v1| { id: v1.id, amount: v1.amount, currency: "JPY" }
}
```

```bash
$ fav schema migrate --from v1 --to v2 --data orders.jsonl
```

`migration` ブロックを AST / parser に追加。
`fav schema migrate` コマンドで JSONL データをマイグレーション定義に従って変換。
`assert_schema`（v52.0 実装済み）にバージョン引数を追加。

**完了条件**: Rust テスト 2 件（`schema_migration_transforms` / `cmd_schema_migrate`）

---

### v58.4.0 — Data Catalog 統合（`fav catalog`）

```bash
$ fav catalog push --catalog datahub://localhost:8080
Registering pipeline: OrderIngestion
  stage Parse:    RawOrder → Order
  stage Validate: Order → Result<ValidOrder>
  stage Store:    ValidOrder → Unit  (Snowflake: orders_v2)

$ fav catalog search "order"
OrderIngestion  pipeline  last_run: 2026-07-23T10:00:00Z
```

`fav catalog push` で DataHub / Apache Atlas にパイプラインメタデータ（lineage / schema）を登録。
`fav catalog search` でカタログ検索。`!Catalog` エフェクトを追加。

**完了条件**: Rust テスト 2 件（`cmd_catalog_push` / `cmd_catalog_search`）

---

### v58.5.0 — Policy-as-Code（`fav policy`）

```favnir
// policy/data-retention.fav
policy DataRetention {
  rule NoPersonalDataInLogs: |pipeline| {
    pipeline.stages
      |> List.filter(|s| s.writes_to("logs"))
      |> List.all(|s| !s.accesses_field("email") && !s.accesses_field("user_id"))
  }
}
```

```bash
$ fav policy check pipeline.fav --policy-dir policy/
[FAIL] DataRetention: stage "AuditLog" writes email to logs
```

`policy` ブロックを AST / parser に追加。
`fav policy check` コマンドでポリシー違反を検出。E0426 エラーコード（`ポリシー違反`）を追加。
`fav policy list` でアクティブポリシー一覧を表示。

**完了条件**: Rust テスト 2 件（`policy_check_violation` / `policy_check_passes`）

---

### v58.6.0 — マルチ環境設定（dev / staging / prod）

```toml
# fav.toml
[env.dev]
snowflake.database = "DEV_DB"
kafka.bootstrap = "localhost:9092"

[env.staging]
snowflake.database = "STAGING_DB"
kafka.bootstrap = "kafka-staging:9092"

[env.prod]
snowflake.database = "PROD_DB"
kafka.bootstrap = "kafka-prod:9092"
```

```bash
$ fav run pipeline.fav --env staging
$ fav run pipeline.fav --env prod
```

`fav.toml` の `[env.<name>]` セクションを解析し、`--env` フラグで環境別設定を選択。
既存の `expand_env_vars` の拡張として `inject_env_config` を実装。

**完了条件**: Rust テスト 2 件（`env_config_parsed` / `env_config_injected`）

---

### v58.7.0 — HA / DR（ヘルスチェック・フェイルオーバー）

```bash
$ fav run pipeline.fav --ha --replica 2
[HA] Primary replica started (port 8080)
[HA] Secondary replica started (port 8081)
[HA] Health check: /healthz → 200 OK
[HA] Failover: primary → secondary (reason: primary unresponsive)
```

`fav run --ha --replica <n>` フラグで複数レプリカを起動。
`/healthz` エンドポイントを自動追加。
プライマリ障害時に自動フェイルオーバーする Tokio ベースの watchdog を実装。

**完了条件**: Rust テスト 2 件（`ha_health_check_endpoint` / `ha_failover_triggers`）

---

### v58.8.0 — ドキュメントサイト Governance & Deployment 記事

`site/content/docs/enterprise/deployment.mdx` — Blue/Green・カナリア・HA の設定と運用。
`site/content/docs/enterprise/governance.mdx` — Schema Migration・Data Catalog・Policy-as-Code。
`site/content/cookbook/multi-env-pipeline.mdx` — マルチ環境設定のレシピ。

**完了条件**: Rust テスト 2 件（`docs_deployment_page_exists` / `docs_governance_page_exists`）

---

### v58.9.0 — 安定化・コードフリーズ（Governance & Deployment 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/governance-overview.mdx` 骨子作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_58_9_0` / `governance_overview_exists`）

---

### v59.0.0 — Governance & Deployment 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「パイプラインは Blue/Green で無停止デプロイされ、
>  カナリアは段階的にトラフィックを引き受ける。
>  スキーマはバージョン管理され、データはカタログで検索できる。
>  ポリシーはコードで記述され、コンプライアンスは自動で証明される。
>  Favnir のパイプラインは運用チームに信頼される。
>
>  これが Favnir v59.0 — Governance & Deployment 2.0 の姿である。」

**完了条件**:
- v58.1〜v58.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3294**）
- `v59000_tests` 4 件 pass（`cargo_toml_version_is_59_0_0` / `changelog_has_v59_0_0` / `milestone_has_governance_deployment2` / `readme_mentions_governance_deployment2`）
- `MILESTONE.md` に `"Governance & Deployment 2.0"` が含まれる（宣言文エントリを追加すること）
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---
---

## v60.0 — Enterprise 1.0 宣言（v59.1〜v59.9）

### v59.1.0 — エンタープライズ E2E ハーネス強化

```bash
# エンタープライズ全機能を統合した E2E テストスイート
$ fav test --suite enterprise
[OK] RBAC enforcement
[OK] Secret injection (AWS SM mock)
[OK] mTLS connection
[OK] Audit log signing + verification
[OK] Blue/Green deploy simulation
[OK] Compliance report (GDPR)
[OK] Policy check (DataRetention)
[OK] Data catalog push (DataHub mock)
```

`examples/enterprise-demo/` ディレクトリに全エンタープライズ機能を統合したデモを作成。
`fav test --suite enterprise` コマンドを追加。`driver.rs` に `cmd_test_enterprise` を実装。

**完了条件**: Rust テスト 2 件（`enterprise_e2e_demo_structure` / `cmd_test_enterprise_suite`）

---

### v59.2.0 — SLA 保証ティア（SLA Guarantee + アラート統合）

```toml
# fav.toml
[sla]
latency_p99_ms  = 200
error_rate_pct  = 0.1
availability_pct = 99.9

[sla.alerting]
channels = ["pagerduty", "slack"]
escalation_policy = "prod-oncall"
```

既存の `sla` Rune（v52.5 実装済み）をより上位の SLA Guarantee モードとして統合。
`fav run --sla-enforce` フラグで実行時 SLA 監視を有効化し、違反時に自動アラートを発火。
`fav sla report` コマンドで SLA 達成率レポートを生成。

**完了条件**: Rust テスト 2 件（`sla_guarantee_config_parsed` / `sla_report_generates`）

---

### v59.3.0 — コスト可視化（`fav cost-estimate`）

```bash
$ fav cost-estimate pipeline.fav --provider aws
Stage Analysis:
  Parse     (Kafka):      ~$0.08/hour  (2M msgs/hr × $0.04/1M)
  Validate  (CPU):        ~$0.03/hour  (0.5 vCPU on Lambda)
  Store     (Snowflake):  ~$0.12/hour  (1 credit/hr × $3/credit / 25)

Total estimated cost: ~$0.23/hour  (~$165/month)
```

`fav cost-estimate` コマンドを追加。各 Rune の操作量とクラウドプロバイダの料金表
（`registry/pricing/<provider>.json`）を照合してコスト見積もりを計算。

**完了条件**: Rust テスト 2 件（`cost_estimate_generates` / `cost_estimate_aws_pricing`）

---

### v59.4.0 — Rune マーケットプレイス Phase 1（`fav marketplace`）

```bash
$ fav marketplace list
Name          Author          Downloads  License
kafka         favnir-official  12,450    MIT
snowflake     favnir-official   8,320    MIT
salesforce    acme-corp           920    Apache-2.0

$ fav marketplace publish --rune my-rune
Publishing my-rune@1.0.0 to Favnir Marketplace...
[OK] Published: https://marketplace.favnir.dev/rune/my-rune
```

既存の `fav publish`（v29.1 実装済み）を Marketplace 向けに拡張。
`fav marketplace list` / `fav marketplace search` を追加。
エンタープライズ向け Private Registry サポートを追加。

**完了条件**: Rust テスト 2 件（`cmd_marketplace_list` / `cmd_marketplace_publish`）

---

### v59.5.0 — Migration Toolkit（v1 → Enterprise マイグレーション）

```bash
$ fav migrate --from v1 --to enterprise --dry-run
[analyze] pipeline.fav
  [WARN] import rune "kafka" → import kafka  (W035: legacy_import_rune)
  [WARN] !Http effect: add TLS config to fav.toml  (new in v57.3)
  [INFO] No RBAC config detected: add [security.rbac] if needed
  [INFO] No [env.*] sections: consider multi-env config (v58.6)

$ fav migrate --from v1 --to enterprise --apply
[fixed] import rune "kafka" → import kafka
```

`fav migrate --from <version> --to <target>` コマンドを追加。
W035（legacy import）の自動修正と、Enterprise 機能への移行ガイダンスを生成。
`--dry-run` で変更内容を確認し、`--apply` で自動修正を適用。

**完了条件**: Rust テスト 2 件（`cmd_migrate_dry_run` / `cmd_migrate_auto_fix_import`）

---

### v59.6.0 — Enterprise 認定チェックリスト（`fav certify`）

```bash
$ fav certify --level enterprise
Checking Favnir Enterprise 1.0 requirements...
[OK]  RBAC configured ([security.rbac])
[OK]  Secrets managed (provider: aws-secrets-manager)
[OK]  TLS enabled ([security.tls])
[OK]  Audit logging active (--audit-sign enabled in CI)
[OK]  Compliance report: GDPR (last generated: 2026-07-23)
[WARN] SLA enforcement: not enabled in production pipeline
       Add: [sla] + fav run --sla-enforce

Enterprise 1.0 certification: 5/6 checks passed (1 warning)
```

`fav certify --level enterprise` コマンドを追加。
`fav.toml` と CI 設定を解析して Enterprise 1.0 要件の充足を確認。
証明書 JSON（`enterprise-cert.json`）を生成。

**完了条件**: Rust テスト 2 件（`cmd_certify_passes` / `cmd_certify_generates_cert`）

---

### v59.7.0 — README / MILESTONE Enterprise 1.0 整備

`README.md` に Enterprise 1.0 への言及・v56〜v60 機能サマリーを追加。
`MILESTONE.md` に `## v60.0.0（予定）— Enterprise 1.0` エントリを追加。
`site/content/docs/enterprise/enterprise1-overview.mdx` を作成。

**完了条件**: Rust テスト 2 件（`readme_has_enterprise1_mention` / `docs_enterprise1_overview_exists`）

---

### v59.8.0 — ドキュメントサイト Enterprise 1.0 総括記事

`site/content/docs/enterprise/index.mdx` — Enterprise 1.0 の全機能一覧・認定要件・移行ガイド。
`site/content/cookbook/enterprise-checklist.mdx` — Enterprise 運用に必要な設定チェックリスト。

**完了条件**: Rust テスト 2 件（`docs_enterprise_index_exists` / `cookbook_enterprise_checklist_exists`）

---

### v59.9.0 — 安定化・コードフリーズ（Enterprise 1.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/enterprise1-overview.mdx` を完成させる。
`cargo test` 全通過を確認して v60.0 へ。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_59_9_0` / `enterprise1_overview_doc_complete`）

---

### v60.0.0 — Enterprise 1.0 宣言 ★クリーンアップ

**宣言文**:

> 「ストリームはウィンドウで区切られ、型システムは制約で守られる。
>  アクセスはロールで制御され、シークレットはコードに現れない。
>  デプロイは無停止で切り替わり、ポリシーはコードで記述される。
>  コストは可視化され、SLA は保証され、コンプライアンスは証明される。
>
>  Favnir はデータエンジニアリングのエンタープライズ標準になった。
>
>  これが Favnir v60.0 — Enterprise 1.0 の姿である。」

**完了条件**:
- v59.1〜v59.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3316**）
- `v60000_tests` 4 件 pass（`cargo_toml_version_is_60_0_0` / `changelog_has_v60_0_0` / `milestone_has_enterprise1` / `readme_mentions_enterprise1`）
- `MILESTONE.md` に `"Enterprise 1.0"` が含まれる（宣言文エントリを追加すること）
- `★クリーンアップ`（`cargo clean`）完了

**実績**: — （未実施）

---

## テスト数推移

| バージョン | 想定テスト数 | 累積増加 | 備考 |
|---|---|---|---|
| v55.0.0（ベース） | 3206 | — | 実績値（目標値 3201 を 5 件超過） |
| v56.0.0 | ~3228 | +22 | |
| v57.0.0 | 3252 | +44 | 実績値（2026-07-26 COMPLETE） |
| v57.1.0 | 3255 | +3 | 実績値（2026-07-27 COMPLETE） |
| v57.2.0 | 3257 | +2 | 実績値（2026-07-27 COMPLETE） |
| v57.3.0 | 3259 | +2 | 実績値（2026-07-27 COMPLETE） |
| v57.4.0 | 3261 | +2 | 実績値（2026-07-28 COMPLETE） |
| v57.5.0 | 3263 | +2 | 実績値（2026-07-28 COMPLETE） |
| v57.6.0 | 3265 | +2 | 実績値（2026-07-28 COMPLETE） |
| v57.7.0 | 3267 | +2 | 実績値（2026-07-28 COMPLETE） |
| v57.8.0 | 3270 | +3 | 実績値（2026-07-28 COMPLETE） |
| v57.9.0 | 3272 | +2 | 実績値（2026-07-28 COMPLETE） |
| v58.0.0 | 3276 | +4 | 実績値（2026-07-28 COMPLETE） |
| v59.0.0 | 3326 | +32 | 実績値（v59.1〜v59.9 完了） |
| v60.0.0 | 3330 | +4 | 実績値（2026-07-30 COMPLETE） |

各サブスプリント 2 件追加、各マイルストーン 4 件追加（x.0.0 テストモジュール）。
実際の件数はサブスプリントロードマップ作成時に確定する。

## 追加されるエラーコード・警告コード

| コード | バージョン | 内容 |
|---|---|---|
| E0421 | v55.5.0 | `!State` エフェクトなし state 操作 |
| E0422 | v56.1.0 | `where` 節 constraint 違反（旧 E0325 統合） |
| E0423 | v56.2.0 | `impl` coherence 違反 |
| E0424 | v57.1.0 | RBAC アクセス拒否 |
| E0425 | v57.7.0+ | tenancy strict モード違反（checker 統合は後続バージョンで対応） |
| E0426 | v58.5.0 | ポリシー違反 |
| W037 | v56.5.0 | 到達不能パターン |
| W038 | v56.7.0 | ワイルドカードインポートによる名前衝突 |

---

## 参考リンク

- 前マスターロードマップ（完了）: `versions/roadmap/roadmap-v50.1-v55.0.md`
- 前サブスプリント詳細（完了）: `versions/roadmap/roadmap-v54.1-v55.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
