# v26.4.0 実装計画 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

## 実装方針

- `StreamingAnnotation` への `backpressure` フィールド追加は **最小変更**（ast.rs + parser.rs の 2 ファイルのみ）
- 新規 `Stream.*` 4 primitive は既存の `VMStream` 列挙体に追加（既存アームの直後に配置）
- `runes/stream/stream.fav` は 6 関数を vm.rs primitive に薄くラップするだけ（Cargo 依存追加なし）
- `Stream.*` primitive に `#[cfg]` ガードは付けない（既存の `Stream.map` 等と同じ方針）

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                        # "26.3.0" であること
cat benchmarks/v26.3.0.json                             # "test_count":2062 であること
cargo test --bin fav 2>&1 | tail -3                     # 2062 件 PASS であること
ls runes/stream/ 2>/dev/null || echo "not found"        # 未存在であること
grep -n 'backpressure' fav/src/ast.rs | head -5         # 存在しないこと
grep -n '"Stream.flat_map"' fav/src/backend/vm.rs       # 存在しないこと
```

### Step 1: `fav/Cargo.toml` bump（26.3.0 → 26.4.0）

```toml
version = "26.4.0"
```

### Step 2: `fav/src/ast.rs` — `StreamingAnnotation` に `backpressure` 追加

`StreamingAnnotation` 構造体の `chunk_size` フィールドの直後に追加:

```rust
pub struct StreamingAnnotation {
    pub chunk_size: Option<i64>,
    pub backpressure: Option<bool>,  // v26.4.0 追加
    pub span: Span,
}
```

`StreamingAnnotation` のデフォルト/初期化箇所をすべて確認し、`backpressure: None` を追加。

### Step 3: `fav/src/frontend/parser.rs` — `backpressure` パース対応

`parse_streaming_annotation`（または `#[streaming(...)]` を処理する相当関数）内で
`chunk_size` のパース処理の後（`else if` ないし match の追加アーム）に以下を追加:

```rust
} else if key == "backpressure" {
    // 値: true / false トークン
    // ann.backpressure = Some(true/false);
}
```

パース後の `StreamingAnnotation` 初期化に `backpressure` フィールドを追加する。

### Step 4: `fav/src/backend/vm.rs` — 新規 `Stream.*` 4 primitive 追加

#### Step 4a: `VMStream` 列挙体に 4 バリアント追加

既存の `VMStream` enum の最後に追加:

```rust
FlatMap {
    stream: Box<VMStream>,
    func: VMValue,
},
Window {
    stream: Box<VMStream>,
    size_secs: i64,
    func: VMValue,
},
Merge {
    streams: Vec<VMStream>,
},
Split {
    stream: Box<VMStream>,
    predicate: VMValue,
},
```

#### Step 4b: `"Stream.flat_map"` primitive 追加

挿入位置: 既存 `"Stream.to_list"` または `"Stream.take"` primitive の後。

```rust
"Stream.flat_map" => {
    // args: [stream, fn]
    // スタブ: VMValue::Stream(Box::new(VMStream::FlatMap { stream, func })) を返す
}
```

#### Step 4c: `"Stream.window"` primitive 追加

```rust
"Stream.window" => {
    // args: [stream, size_secs, fn]
    // スタブ: VMValue::Stream(Box::new(VMStream::Window { stream, size_secs, func })) を返す
}
```

#### Step 4d: `"Stream.merge"` primitive 追加

```rust
"Stream.merge" => {
    // args: [streams_list]  — VMValue::List of VMValue::Stream
    // スタブ: VMValue::Stream(Box::new(VMStream::Merge { streams })) を返す
}
```

#### Step 4e: `"Stream.split"` primitive 追加

```rust
"Stream.split" => {
    // args: [stream, predicate]
    // スタブ: 2 要素の VMValue::List を返す
    // [VMValue::List(trues), VMValue::List(falses)] — 内側要素も VMValue::List（VMValue::Stream ではない）
}
```

#### Step 4f: `materialize_stream` 関数の VMStream match に 4 バリアントのアーム追加

`vm.rs` の `materialize_stream` 関数（`Stream.to_list` はこの関数を呼び出す）は
`VMStream` の全バリアントを `match` している。新バリアント追加後に `non-exhaustive patterns` エラーが出るため、
以下の各バリアントに対するアームを追加する:
- `VMStream::FlatMap { stream, func }` → ストリームを評価してリスト化し、各要素に func を適用後フラット化
- `VMStream::Window { stream, size_secs, func }` → `size_secs` 要素ずつバッチ化して func を適用（スタブ: 秒数ではなく要素数として扱う）
- `VMStream::Merge { streams }` → 各ストリームを順次連結した `Vec<VMValue>` を返す
- `VMStream::Split { stream, predicate }` → predicate で仕分けして `VMValue::List([VMValue::List(trues), VMValue::List(falses)])` を返す

### Step 4.5: `cargo build` — コンパイルエラーなし確認

```bash
cargo build --bin fav 2>&1 | grep -E "^error" | head -10
```

### Step 5: `runes/stream/stream.fav` 新規作成

spec.md §4 の内容を実装:

```favnir
public fn map(stream, f) { Stream.map(stream, f) }
public fn filter(stream, pred) { Stream.filter(stream, pred) }
public fn flat_map(stream, f) { Stream.flat_map(stream, f) }
public fn window(stream, size_secs, f) { Stream.window(stream, size_secs, f) }
public fn merge(streams) { Stream.merge(streams) }
public fn split(stream, pred) { Stream.split(stream, pred) }
```

### Step 6: `site/content/docs/runes/stream.mdx` 新規作成

- `#[streaming]` アノテーション全オプション（chunk_size / backpressure）
- `Stream.*` 6 関数 API リファレンス
- タンブリングウィンドウの使用例
- スコープ外（スライディングウィンドウ等）

### Step 7: `CHANGELOG.md` 更新

```markdown
## [v26.4.0] — 2026-06-26 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

### Added
- `StreamingAnnotation.backpressure: Option<bool>` フィールド追加（ast.rs + parser.rs）
- `Stream.flat_map` / `Stream.window` / `Stream.merge` / `Stream.split` — VM primitive 4 件追加
- `runes/stream/stream.fav` — Stream Rune 新規作成（map / filter / flat_map / window / merge / split）
- `site/content/docs/runes/stream.mdx` — Stream Rune ドキュメント新規作成
```

### Step 8: `benchmarks/v26.4.0.json` 新規作成

```json
{"version":"26.4.0","test_count":2070,"timestamp":"2026-06-26"}
```

### Step 9: `fav/src/driver.rs` に `v264000_tests` 追加

`v263000_tests` の直後に追加:

```rust
// ── v264000_tests (v26.4.0) — #[streaming] backpressure + Stream.* 操作 ──────
#[cfg(test)]
mod v264000_tests {
    #[test]
    fn stream_rune_has_map_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn map"), "stream map fn not found");
    }
    #[test]
    fn stream_rune_has_filter_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn filter"), "stream filter fn not found");
    }
    #[test]
    fn stream_rune_has_flat_map_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn flat_map"), "stream flat_map fn not found");
    }
    #[test]
    fn stream_rune_has_window_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn window"), "stream window fn not found");
    }
    #[test]
    fn stream_rune_has_merge_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn merge"), "stream merge fn not found");
    }
    #[test]
    fn stream_rune_has_split_fn() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("public fn split"), "stream split fn not found");
    }
    #[test]
    fn streaming_annotation_supports_backpressure() {
        let src = include_str!("ast.rs");  // driver.rs と ast.rs は同じ src/ ディレクトリ
        assert!(src.contains("backpressure"), "StreamingAnnotation must have backpressure field");
    }
    #[test]
    fn changelog_has_v26_4_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.4.0]"), "CHANGELOG.md must contain '[v26.4.0]'");
    }
}
```

### Step 10: テスト確認

```bash
cd fav && cargo test v264000 --bin fav          # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2070 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.3.0 → 26.4.0 |
| `fav/src/ast.rs` | `StreamingAnnotation` に `backpressure: Option<bool>` 追加 |
| `fav/src/frontend/parser.rs` | `backpressure` キーのパース追加 |
| `fav/src/backend/vm.rs` | `VMStream` に 4 バリアント追加 + 4 primitive 追加 + `materialize_stream` 関数の match 拡張 |
| `runes/stream/stream.fav` | **新規作成**（6 関数） |
| `site/content/docs/runes/stream.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.4.0]` エントリ先頭に追加 |
| `benchmarks/v26.4.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v264000_tests`（8 件）追加 |

---

## 注意事項

- `VMStream` 列挙体の変更後、`VMStream` に対する全ての `match` 式が網羅的になっているか確認すること（`Stream.to_list` の評価ループなど）。`cargo build` で `non-exhaustive patterns` エラーが出た場合は全アームに対応する。
- `runes/stream/` ディレクトリは未存在。Write ツールが自動作成する。
- `streaming_annotation_supports_backpressure` テストは `include_str!("../ast.rs")` を使う（`driver.rs` から見た相対パス）。
- `stream.fav` の `fn map` / `fn filter` は vm.rs の既存 `Stream.map` / `Stream.filter` primitive を呼ぶ。新規 primitive は追加しない。
- `Stream.split` の返却型は `VMValue::List([true_stream, false_stream])` — タプル型は不要。
- `backpressure: true` の VM 実行時セマンティクスは v27.x 以降。v26.4.0 ではアノテーションとして格納するのみでよい。

## リスクと対応

| リスク | 対応 |
|---|---|
| `VMStream` match 式の網羅性エラー | `cargo build` 後に `non-exhaustive patterns` を確認し、全バリアントにアームを追加 |
| `StreamingAnnotation` 初期化箇所の漏れ | `grep -n 'StreamingAnnotation {' fav/src/` で全箇所を確認し `backpressure: None` を追加 |
| `Stream.split` の返却型が不明瞭 | `VMValue::List` の 2 要素 `[trues_stream, falses_stream]` として実装 |
| `Stream.window` の `size_secs` 型（`i64` vs `f64`） | `i64` で統一（既存の `VMValue::Int` と一致） |
