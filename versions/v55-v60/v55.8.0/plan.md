# Plan — v55.8.0 — ドキュメントサイト Streaming 2.0 記事

## ステップ

### Step 0: 事前作業 — ロードマップのテスト数訂正（実装開始前に実施）

`versions/roadmap/roadmap-v55.1-v56.0.md` の v55.8.0 完了条件テスト数（3221）は変更しない。
コードレビュー対応で `cookbook_cep_patterns_exists` を追加するため、実績テスト数は 3222 となる。
ロードマップの実績欄には 3222 を記録する（完了条件欄の 3221 はそのまま）。

---

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `55.8.0` に更新。

```toml
[package]
version = "55.8.0"
```

---

### Step 2: MDX ファイル作成（3 件）

#### 2a: `site/content/docs/runtime/streaming-v2.mdx`

以下セクションを含む:
- ウィンドウ（Tumbling/Sliding/Session）+ fav.toml 設定
- ウォーターマーク（遅延許容）
- Exactly-once チェックポイント（`fav checkpoint list` / `--resume-from`）
- Stateful Stage（State.get_or_default / State.set）
- CEP（CEP.sequence / CEP.skip_until、cep-patterns レシピへのリンク）
- Stream Join（Stream.join_inner / Stream.join_left）

型注釈の正確性: `Window.tumbling` を使う stage の戻り型は `Stream<List<Event>>`。

#### 2b: `site/content/cookbook/stateful-pipeline.mdx`

以下レシピを含む:
- ユーザーごとのイベントカウント（`State.get_or_default` + `State.set`）
- 累積合計（running sum）
- セッション状態（最後のアクティブ時刻）
- fav.toml `[stream]` 設定例
- 注意事項（State.get → Option<T>、State.set → Unit 等）

#### 2c: `site/content/cookbook/cep-patterns.mdx`

以下レシピを含む:
- 注文→決済の連続検出（`CEP.sequence` 2 述語）
- エラーイベント以降を抽出（`CEP.skip_until`、inclusive セマンティクス）
- 3 ステップシーケンス検出
- Stateful CEP（状態永続化）
- 境界ケース一覧表

---

### Step 3: `driver.rs` — `v55800_tests` モジュール追加

`v55700_tests` の直前（`// -- v55700_tests` コメント行の前）に挿入する。

```rust
// -- v55800_tests (v55.8.0) -- ドキュメントサイト Streaming 2.0 記事 --
#[cfg(test)]
mod v55800_tests {
    #[test]
    fn docs_streaming_v2_page_exists() {
        let src = include_str!("../../site/content/docs/runtime/streaming-v2.mdx");
        assert!(src.contains("Streaming Native 2.0"), "streaming-v2.mdx must mention Streaming Native 2.0");
        assert!(src.contains("CEP"), "streaming-v2.mdx must mention CEP");
        assert!(src.contains("checkpoint"), "streaming-v2.mdx must mention checkpoint");
        assert!(src.contains("Stateful"), "streaming-v2.mdx must mention Stateful");
    }

    #[test]
    fn cookbook_stateful_pipeline_exists() {
        let src = include_str!("../../site/content/cookbook/stateful-pipeline.mdx");
        assert!(src.contains("State.get"), "stateful-pipeline.mdx must mention State.get");
        assert!(src.contains("State.set"), "stateful-pipeline.mdx must mention State.set");
        assert!(src.contains("State.get_or_default"), "stateful-pipeline.mdx must mention State.get_or_default");
    }

    #[test]
    fn cookbook_cep_patterns_exists() {
        let src = include_str!("../../site/content/cookbook/cep-patterns.mdx");
        assert!(src.contains("CEP.sequence"), "cep-patterns.mdx must mention CEP.sequence");
        assert!(src.contains("CEP.skip_until"), "cep-patterns.mdx must mention CEP.skip_until");
    }
}
```

---

### Step 4: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo build 2>&1 | tail -5
```

期待結果: `Finished`

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待結果: `3222 tests passed, 0 failed`

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

期待結果: クリーン

---

### Step 5: ポスト処理

- `CHANGELOG.md` に v55.8.0 エントリ追加
- `versions/current.md` を v55.8.0 / 3222 tests に更新
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.8.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.8.0 実績欄も COMPLETE に更新

---

## 注意事項

- `include_str!` パスは `fav/src/driver.rs` 起点で `../../site/content/...` となる（`fav/` の 2 階層上が `favnir/`）。
- MDX の Favnir コードサンプルは言語仕様に準拠:
  - リストリテラル `[a, b, c]` は非対応 → `collect { yield a; yield b; }` を使う
  - `let` は非対応 → `bind val <- expr` を使う
  - `Window.tumbling` の戻り型は `List<Event>`（ウィンドウ内イベント集合）
- `streaming-v2.mdx` に `Window.tumbling` のサンプルを載せる場合、
  戻り型を `Stream<Int>` と書くのは誤り（コードレビュー [MED] 指摘事項）。
  `Stream<List<Event>>` を使うこと。
- `cep-patterns.mdx` の Stateful CEP サンプルで `bind matches <- Ok(CEP.sequence(...))` は
  Favnir の do-notation 内で List を Result に昇格させる有効パターン。
