# v26.4.0 仕様書 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.4.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | `#[streaming]` に `backpressure` フィールド追加 + `Stream.*` 6 操作の実質化 |
| 依存関係 | v26.1〜v26.3 完了後（実 Rune とのインテグレーション検証のため） |
| 目標テスト数 | 2070 件（+8 件）|

---

## 背景と目的

v26.1〜v26.3 で kinesis / nats / rabbitmq Rune が「動く Rune の 5 条件」を満たした。
しかし現在の `#[streaming]` アノテーションは `chunk_size` のみ対応しており、
本物のバックプレッシャー制御セマンティクスが欠けている。

また `Stream.*` 操作のうち `map` / `filter` / `take` / `to_list` / `from` / `of` / `gen` は
vm.rs に実装済みだが、`flat_map` / `window` / `merge` / `split` の 4 操作が未実装。

v26.4.0 では以下を実装する:

1. `StreamingAnnotation` に `backpressure: Option<bool>` フィールドを追加（ast.rs + parser.rs）
2. `Stream.flat_map` / `Stream.window` / `Stream.merge` / `Stream.split` を vm.rs に追加
3. `runes/stream/stream.fav` を新規作成（6 関数を公開）
4. `site/content/docs/runes/stream.mdx` を新規作成

### 既存 `Stream.*` の現状（実装済み）

`fav/src/backend/vm.rs` ~4491〜4601 行に以下が既に実装されている:

| primitive 名 | VMStream バリアント | 状態 |
|---|---|---|
| `"Stream.from"` | `VMStream::FromList` | 実装済み |
| `"Stream.of"` | `VMStream::Single` | 実装済み |
| `"Stream.gen"` | `VMStream::Gen` | 実装済み |
| `"Stream.map"` | `VMStream::Map` | 実装済み |
| `"Stream.filter"` | `VMStream::Filter` | 実装済み |
| `"Stream.take"` | `VMStream::Take` | 実装済み |
| `"Stream.to_list"` | （即時評価） | 実装済み |

> これらは `VMValue::Stream(Box<VMStream>)` による遅延評価を使っており、`VMValue::List` ではない。
> v26.4.0 では **既存の実装を変更しない**。

---

## 機能仕様

### 1. `StreamingAnnotation` の拡張（ast.rs）

現在:

```rust
pub struct StreamingAnnotation {
    pub chunk_size: Option<i64>,
    pub span: Span,
}
```

v26.4.0 後:

```rust
pub struct StreamingAnnotation {
    pub chunk_size: Option<i64>,
    pub backpressure: Option<bool>,  // NEW
    pub span: Span,
}
```

Favnir 構文:

```favnir
// chunk_size のみ（従来どおり）
#[streaming(chunk_size: 1000)]

// backpressure 追加（v26.4.0 新機能）
#[streaming(chunk_size: 1000, backpressure: true)]

// backpressure のみも可
#[streaming(backpressure: true)]
```

### 2. parser.rs — `#[streaming]` パース対応

`parse_streaming_annotation`（または相当関数）に `backpressure` キーのパースを追加:

- キー `"backpressure"` → `true` / `false` を `StreamingAnnotation.backpressure` に格納
- 既存の `chunk_size` パース処理と共存させる（`else if` ないし `match key` で分岐）
- 未知キーは無視して継続（後方互換性のため）

### 3. 新規 `Stream.*` primitive 4 件（vm.rs）

既存の `VMStream` 列挙体に以下の 4 バリアントを追加し、対応する primitive を実装する。
実装はスタブ（リスト変換ベース）で構わない。

| primitive 名 | VMStream バリアント | スタブ実装 |
|---|---|---|
| `"Stream.flat_map"` | `VMStream::FlatMap { stream, func }` | ストリームを即時評価してリスト化し、各要素に fn を適用後フラット化 |
| `"Stream.window"` | `VMStream::Window { stream, size_secs, func }` | ストリームを `size_secs` 要素ずつバッチ化して fn を適用（タンブリング）。スタブでは秒数ではなく要素数として扱う。実際の時刻ウィンドウは v27.x 以降。 |
| `"Stream.merge"` | `VMStream::Merge { streams }` | 複数ストリームを順次連結 |
| `"Stream.split"` | `VMStream::Split { stream, predicate }` | predicate でストリームを 2 つのリストに分岐し、`VMValue::List` で返す |

> スタブ実装で `VMValue::Stream(Box<VMStream::FlatMap {...}))` を返してもよいが、
> `Stream.to_list` で評価できる形にすること（遅延評価チェーンに組み込む）。
>
> `Stream.split` は 2 ストリームのタプルではなく `[true_list, false_list]` の `VMValue::List` で返す。
> 実際のストリーミング分岐は v27.x 以降で実装。

#### wasm32 対応

新規 primitive 4 件は `#[cfg(not(target_arch = "wasm32"))]` ガード + wasm32 フォールバック不要。
理由: `Stream.*` primitive は WASM でも使用するため、`#[cfg]` ガードを付けない。

> 既存の `Stream.map` 等にも `#[cfg]` ガードがないことを確認して踏襲すること。

### 4. `runes/stream/stream.fav` — 新規作成

```favnir
// runes/stream/stream.fav — Stream Rune (v26.4.0)
//
// 使い方:
//   import rune "stream"
//
// Stream.* primitive を薄くラップして公開する。
// 既存の Stream.map / Stream.filter は vm.rs に実装済みのため再宣言不要。

public fn map(stream, f) {
    Stream.map(stream, f)
}

public fn filter(stream, pred) {
    Stream.filter(stream, pred)
}

public fn flat_map(stream, f) {
    Stream.flat_map(stream, f)
}

public fn window(stream, size_secs, f) {
    Stream.window(stream, size_secs, f)
}

public fn merge(streams) {
    Stream.merge(streams)
}

public fn split(stream, pred) {
    Stream.split(stream, pred)
}
```

### 5. `site/content/docs/runes/stream.mdx` — 新規作成

- `#[streaming]` アノテーションの全オプション（`chunk_size` / `backpressure`）
- `Stream.*` 6 関数の API リファレンス
- タンブリングウィンドウのユースケース例
- `Stream.merge` / `Stream.split` の利用パターン
- スコープ外（スライディングウィンドウ・セッションウィンドウは v27.x）

---

## エラー処理

- `#[streaming(backpressure: true)]` の VM 実行時: `backpressure` は VM レベルでは現時点でアノテーションとして格納されるのみ（bounded channel 実装は v27.x）。エラーにはならない。
- `Stream.flat_map` / `Stream.window` / `Stream.merge` / `Stream.split` の引数型不一致 → `Err("Stream.X: invalid argument")` を返す

---

## スコープ外（v27.x 以降）

- `backpressure: true` による実際の `tokio::sync::mpsc` bounded channel 制御（ロードマップ v26.4 節は「VM で実際に機能させる」と記述しているが、スタブ段階ではアノテーション格納のみとし bounded channel 実装は v27.x に延期する）
- スライディングウィンドウ / セッションウィンドウ（`Stream.window` の `type` パラメータ）
- `Stream.split` の 2 出力ストリーム同時消費（タプル返却）
- `Stream.zip` / `Stream.throttle` / `Stream.debounce`

---

## Rust テスト（v264000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `stream_rune_has_map_fn` | `runes/stream/stream.fav` に `fn map` が含まれる | assert |
| `stream_rune_has_filter_fn` | `runes/stream/stream.fav` に `fn filter` が含まれる | assert |
| `stream_rune_has_flat_map_fn` | `runes/stream/stream.fav` に `fn flat_map` が含まれる | assert |
| `stream_rune_has_window_fn` | `runes/stream/stream.fav` に `fn window` が含まれる | assert |
| `stream_rune_has_merge_fn` | `runes/stream/stream.fav` に `fn merge` が含まれる | assert |
| `stream_rune_has_split_fn` | `runes/stream/stream.fav` に `fn split` が含まれる | assert |
| `streaming_annotation_supports_backpressure` | `fav/src/ast.rs` に `backpressure` が含まれる（`include_str!("ast.rs")`） | assert |
| `changelog_has_v26_4_0` | `CHANGELOG.md` に `[v26.4.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.4.0"` であること
- [ ] `fav/src/ast.rs` の `StreamingAnnotation` に `backpressure: Option<bool>` フィールドが存在すること
- [ ] `fav/src/frontend/parser.rs` が `backpressure` キーをパースしてエラーにならないこと
- [ ] `fav/src/backend/vm.rs` に `"Stream.flat_map"` primitive が存在すること
- [ ] `fav/src/backend/vm.rs` に `"Stream.window"` primitive が存在すること
- [ ] `fav/src/backend/vm.rs` に `"Stream.merge"` primitive が存在すること
- [ ] `fav/src/backend/vm.rs` に `"Stream.split"` primitive が存在すること
- [ ] `runes/stream/stream.fav` が存在すること
- [ ] `runes/stream/stream.fav` に 6 関数（map / filter / flat_map / window / merge / split）が定義されていること
- [ ] `site/content/docs/runes/stream.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.4.0]` エントリが存在すること
- [ ] `benchmarks/v26.4.0.json` が存在すること（test_count: 2070）
- [ ] `v264000_tests` 8 件すべて PASS
- [ ] 総テスト数 ≥ 2070 件

---

## テスト件数

- v26.3.0 完了時: 2062 件
- v26.4.0 追加: 8 件（v264000_tests）
- **目標**: 2062 + 8 = **2070 件**

> `benchmarks/v26.3.0.json` で `test_count: 2062` を確認済み（実装前に Step 0 で再確認すること）。
> ロードマップの v27.0 完了条件は「`cargo test streaming` で 6 件以上 PASS」。本バージョンでは 8 件を実装する。
