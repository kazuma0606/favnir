# Roadmap v40.1.0 〜 v45.0.0 — Precision & Flow

Date: 2026-07-11
Status: 骨格確定（v40.0 完了時点）、詳細は各マイルストーン完了後に確定

---

## 目標

v40.0「Enterprise Governance」で「チームで安全に運用できる」を実現した。
このフェーズは **「型安全なリアルタイムパイプラインを、最小限の型注釈で記述できる」** を実現する。

```
言語の型推論を強化し、ジェネリクスや戻り値型を手で書かなくても
コンパイラが補完してくれる。
同時に、サブ秒レイテンシのストリーム処理・CEP を型安全に記述できる。
```

---

## バージョン計画

### v41.0.0 — Streaming Foundations ★クリーンアップ

ウィンドウ操作・Watermark・out-of-order イベント処理の基盤を整備する。

#### v40.1.0 — `tumbling_window` / `sliding_window`

```favnir
stage Aggregate {
  bind windowed <- Stream.tumbling_window(events, 60)   // 60秒ウィンドウ
  bind sums     <- List.map(windowed, |w| List.sum(w))
}
```

**完了条件**: Rust テスト 3 件

---

#### v40.2.0 — `session_window`

```favnir
bind sessions <- Stream.session_window(events, gap: 30)
// 30秒アイドルでウィンドウを閉じる
```

**完了条件**: Rust テスト 3 件

---

#### v40.3.0 — `Event<T>` + timestamp フィールド

`Event<T>` 型に `timestamp: Int` フィールドを追加。
ウィンドウ演算の時刻基準として使用。

**完了条件**: Rust テスト 3 件

---

#### v40.4.0 — Out-of-order イベント処理

遅延イベントの許容（late_tolerance）と drop ポリシー（`drop` / `reprocess`）。

**完了条件**: Rust テスト 3 件

---

#### v40.5.0 — `fav.toml [stream]` セクション

```toml
[stream]
watermark_delay = 5     # 秒
late_policy = "drop"    # drop | reprocess
```

**完了条件**: Rust テスト 3 件

---

#### v40.6.0 — Kafka / Redis Streams window 対応

既存 Kafka・Redis Rune にウィンドウ集計メソッドを追加。

**完了条件**: Rust テスト 2 件

---

#### v40.7.0 — `fav bench --stream`

ストリームパイプラインのスループット / レイテンシ計測コマンド。

**完了条件**: Rust テスト 2 件

---

#### v40.8.0 — Streaming cookbook

`site/content/cookbook/window-aggregation.mdx` /
`site/content/cookbook/kafka-streaming.mdx`

**完了条件**: Rust テスト 1 件

---

#### v40.9.0 — 安定化

コードフリーズ。v41.0 前調整。

**完了条件**: meta テスト 2 件

---

#### v41.0.0 — Streaming Foundations 宣言 ★クリーンアップ

**完了条件**:
- v40.1〜v40.9 の全機能が動作する
- `★クリーンアップ` 完了
- `v41000_tests` 4 件 pass

---

### v42.0.0 — Type Precision ★クリーンアップ

Refinement type・タプルパターン・ガード付き match・Row polymorphism 強化。

#### v41.1.0 — Refinement type 基盤

```favnir
type Age = Int where (>= 0 && <= 150)
type Name = String where (len > 0 && len < 256)

fn greet(name: Name) -> String { "Hello, " ++ name }
```

**完了条件**: Rust テスト 3 件

---

#### v41.2.0 — Refinement type `fav check` 統合・E0400 系

refinement 条件違反を静的検出。E0400〜E0404 エラーコード追加。

**完了条件**: Rust テスト 3 件

---

#### v41.3.0 — タプルパターン match

```favnir
match (status, count) {
  ("ok", 0) -> "empty ok"
  ("ok", n) -> "ok: " ++ Int.to_string(n)
  (err, _)  -> "error: " ++ err
}
```

**完了条件**: Rust テスト 3 件

---

#### v41.4.0 — ガード付き match

```favnir
match score {
  n if n >= 90 -> "A"
  n if n >= 70 -> "B"
  _            -> "C"
}
```

**完了条件**: Rust テスト 3 件

---

#### v41.5.0 — Row polymorphism 強化

record spread / diff の型精度向上。部分的な record 型の統一。

**完了条件**: Rust テスト 3 件

---

#### v41.6.0 — Newtype 自動 impl

```favnir
type Kg(Float)   // + / * / - を Float から自動継承
type Meter(Float)
```

**完了条件**: Rust テスト 3 件

---

#### v41.7.0 — W030 lint

refinement 条件の冗長ガード検出（例: `Int where (> 0)` の変数に `if x > 0` は不要）。

**完了条件**: Rust テスト 2 件

---

#### v41.8.0 — Type Precision cookbook

`site/content/cookbook/refinement-types.mdx` /
`site/content/docs/language/refinement-types.mdx`

**完了条件**: Rust テスト 1 件

---

#### v41.9.0 — 安定化

コードフリーズ。v42.0 前調整。

**完了条件**: meta テスト 2 件

---

#### v42.0.0 — Type Precision 宣言 ★クリーンアップ

**完了条件**:
- v41.1〜v41.9 の全機能が動作する
- `★クリーンアップ` 完了
- `v42000_tests` 4 件 pass

---

### v43.0.0 — Real-Time Power ★クリーンアップ

CEP・Stream join・Back-pressure でリアルタイム処理能力を完成させる。

#### v42.1.0 — CEP DSL 基盤

`fav cep` — Complex Event Processing の構文・型・VM サポート基盤。

**完了条件**: Rust テスト 3 件

---

#### v42.2.0 — CEP パターン: `seq` / `any` / `not`

```favnir
cep pattern LoginThenPurchase {
  seq(Login, Purchase) within 300   // 300秒以内
}
```

**完了条件**: Rust テスト 3 件

---

#### v42.3.0 — CEP checker.fav 統合

CEP パターン型の型チェック対応。

**完了条件**: Rust テスト 3 件

---

#### v42.4.0 — Stream join（time-window）

```favnir
bind joined <- Stream.join(orders, payments,
  on: |o, p| o.id == p.order_id,
  window: 60)
```

**完了条件**: Rust テスト 3 件

---

#### v42.5.0 — Back-pressure `@max_inflight`

```favnir
@max_inflight(100)
stage SlowSink {
  bind _ <- Db.batch_insert(ctx, rows)
}
```

**完了条件**: Rust テスト 2 件

---

#### v42.6.0 — WebSocket Rune

リアルタイム push sink。`runes/websocket/` 追加。

**完了条件**: Rust テスト 2 件

---

#### v42.7.0 — `fav monitor`

実行中パイプラインのスループット / イベント数 / レイテンシをターミナルに表示。

**完了条件**: Rust テスト 2 件

---

#### v42.8.0 — Real-Time Power cookbook

`site/content/cookbook/cep-login-purchase.mdx` /
`site/content/cookbook/stream-join.mdx`

**完了条件**: Rust テスト 1 件

---

#### v42.9.0 — 安定化

コードフリーズ。v43.0 前調整。

**完了条件**: meta テスト 2 件

---

#### v43.0.0 — Real-Time Power 宣言 ★クリーンアップ

**完了条件**:
- v42.1〜v42.9 の全機能が動作する
- `★クリーンアップ` 完了
- `v43000_tests` 4 件 pass

---

### v44.0.0 — Language Expressiveness ★クリーンアップ

型推論 6 カテゴリを段階的に実装。v43.13 まで拡張スプリント。

#### v43.1.0 — 戻り値型推論（Return type omission）

```favnir
// 推論前（必須）
fn double(x: Int) -> Int { x * 2 }

// 推論後（省略可）
fn double(x: Int) { x * 2 }   // -> Int をブロック末尾式から推論
```

checker.fav・compiler.fav 両方に対応。

**完了条件**: Rust テスト 3 件

---

#### v43.2.0 — 戻り値型推論: `fav check` 統合・E0401 系

推論失敗時のエラー E0401（ambiguous return type）。
`fav check --show-types` で推論された戻り値型を表示。

**完了条件**: Rust テスト 3 件

---

#### v43.3.0 — ジェネリック型引数推論（Call-site inference）

```favnir
fn identity<A>(x: A) -> A { x }

bind v <- identity(42)      // A = Int を引数から確定
bind s <- identity("hello") // A = String を引数から確定
```

**完了条件**: Rust テスト 3 件

---

#### v43.4.0 — ジェネリック推論: 曖昧ケース検出（E0402）

複数の型変数が競合する場合に E0402 ambiguous type variable を報告。

**完了条件**: Rust テスト 3 件

---

#### v43.5.0 — ラムダ引数型推論（Contextual lambda inference）

```favnir
// 推論前（明示）
[1, 2, 3] |> List.map(|x: Int| x * 2)

// 推論後（List<Int> から x: Int が伝播）
[1, 2, 3] |> List.map(|x| x * 2)
```

**完了条件**: Rust テスト 3 件

---

#### v43.6.0 — パイプライン型伝播（Pipeline stage typing）

```favnir
stage Transform {
  bind rows  <- Csv.read("data.csv")           // Stream<Row>  — 推論
  bind nums  <- List.map(rows, |r| r.value)    // List<Float>  — 推論
  bind valid <- List.filter(nums, |v| v > 0.0) // List<Float>  — 推論
}
// 中間型の明示が不要になる
```

**完了条件**: Rust テスト 3 件

---

#### v43.7.0 — 構造体リテラル推論（Structural inference）

```favnir
// 渡す先の関数シグネチャから要素型が確定
process({ name: "Alice", age: 30 })
// process の引数型から { name: String, age: Int } を推論
```

リスト・タプル・レコードリテラルの型を呼び出しコンテキストから決定。

**完了条件**: Rust テスト 3 件

---

#### v43.8.0 — 双方向型推論（Bidirectional / top-down）

期待型の下向き伝播。関数が `Int -> Bool` を期待していれば `|x| x > 0` の `x: Int` が確定。

```favnir
fn filter_positive(xs: List<Int>) -> List<Int> {
  List.filter(xs, |x| x > 0)   // x: Int は xs の要素型から伝播
}
```

**完了条件**: Rust テスト 3 件

---

#### v43.9.0 — `fav check --show-inference`

全式に推論された型を注釈表示。型推論のデバッグ支援。

**完了条件**: Rust テスト 2 件

---

#### v43.10.0 — `fav check --explain` — 推論失敗時 AI 解説統合

v39 の Llm Rune を活用し、推論失敗エラーの自然言語解説を出力。

**完了条件**: Rust テスト 2 件

---

#### v43.11.0 — Opaque type 完全化

```favnir
opaque type Token = String   // 外部からの String への暗黙 coerce を禁止
```

**完了条件**: Rust テスト 3 件

---

#### v43.12.0 — W031〜W033 lint（冗長型注釈の警告）

- W031: 推論可能な戻り値型の明示的注釈
- W032: 推論可能なジェネリック型引数の明示
- W033: 推論可能なラムダ引数型の明示

**完了条件**: Rust テスト 3 件

---

#### v43.13.0 — Language Expressiveness cookbook + 安定化

`site/content/cookbook/type-inference-guide.mdx` /
`site/content/docs/language/type-inference.mdx`

コードフリーズ。v44.0 前調整。

**完了条件**: meta テスト 2 件

---

#### v44.0.0 — Language Expressiveness 宣言 ★クリーンアップ

**完了条件**:
- v43.1〜v43.13 の全機能が動作する
- `★クリーンアップ` 完了
- `v44000_tests` 4 件 pass

---

### v45.0.0 — Precision & Flow 宣言 ★クリーンアップ

Streaming × Type Precision × Real-Time × Language Expressiveness の統合完成。

#### v44.1.0 — Refinement type × Streaming 統合

```favnir
type PositiveFloat = Float where (> 0.0)
stage Validate {
  bind valid <- List.filter(events, |e| e.value > 0.0)
  // valid: Stream<Event<PositiveFloat>> — 推論
}
```

**完了条件**: Rust テスト 3 件

---

#### v44.2.0 — CEP × Refinement type

CEP パターン条件に refinement type を使用可能にする。

**完了条件**: Rust テスト 3 件

---

#### v44.3.0 — Stream join × Opaque type

join キーを opaque type で型安全に管理。型が異なるキーでの誤 join を静的に防ぐ。

**完了条件**: Rust テスト 3 件

---

#### v44.4.0 — 型推論 × パイプライン lineage

`fav explain --lineage` の出力に推論された型を表示。
ウィンドウ・join の lineage も追跡対象に含める。

**完了条件**: Rust テスト 2 件

---

#### v44.5.0 — Back-pressure × `fav policy` 統合

ポリシー宣言に `max_inflight` 上限を追加可能にする。

```favnir
policy {
  max_inflight: 100
}
```

**完了条件**: Rust テスト 2 件

---

#### v44.6.0 — Precision & Flow E2E デモ

`infra/e2e-demo/precision-flow/` — CEP + refinement type + governance 統合デモ。
実データを使った Kafka → CEP → Opaque join → Policy gate の完全パイプライン。

**完了条件**: Rust テスト 1 件

---

#### v44.7.0 — ドキュメントサイト — Precision & Flow 概要ページ

`site/content/docs/precision-and-flow.mdx` — 全機能の統合解説ページ。

**完了条件**: Rust テスト 1 件

---

#### v44.8.0 — パフォーマンス最終調整

ストリーム処理 + 型推論の速度最適化。
`fav bench --stream` での計測値を v41.0 比で改善。

**完了条件**: Rust テスト 2 件

---

#### v44.9.0 — 安定化

コードフリーズ。v45.0 前調整。

**完了条件**: meta テスト 2 件

---

#### v45.0.0 — Precision & Flow 宣言 ★クリーンアップ

**宣言文（暫定）**:

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

**完了条件**:
- v41.0〜v44.9 の全機能が動作する / テスト数（宣言時実績 + 4）
- `★クリーンアップ` 完了
- `v45000_tests` 4 件 pass（version / changelog / milestone / readme）

---

## スプリント構成まとめ

| スプリント | バージョン範囲 | 版数 | テーマ |
|---|---|---|---|
| Streaming Foundations | v40.1〜v40.9 → v41.0 | 9+1 | ウィンドウ・Watermark・OOO |
| Type Precision | v41.1〜v41.9 → v42.0 | 9+1 | Refinement・パターン強化 |
| Real-Time Power | v42.1〜v42.9 → v43.0 | 9+1 | CEP・Stream join・Back-pressure |
| Language Expressiveness | v43.1〜v43.13 → v44.0 | 13+1 | 型推論 6 カテゴリ + Opaque |
| Precision & Flow 宣言 | v44.1〜v44.9 → v45.0 | 9+1 | 全機能統合・E2E デモ |

---

## 参考リンク

- マスタースケジュール前版: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v39.1-v40.0.md`
- 達成宣言: `MILESTONE.md`
