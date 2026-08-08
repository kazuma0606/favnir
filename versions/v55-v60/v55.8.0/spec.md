# Spec — v55.8.0 — ドキュメントサイト Streaming 2.0 記事

## 概要

v55.8.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の第 8 弾。
v55.1〜v55.7 で実装したウィンドウ・ウォーターマーク・Exactly-once・CEP・Stateful・Checkpoint を
サイトドキュメント（MDX）として公開し、Rust テストで存在確認を行う。

具体的には以下を実装する：
1. `site/content/docs/runtime/streaming-v2.mdx` — Streaming Native 2.0 概要記事
2. `site/content/cookbook/stateful-pipeline.mdx` — Stateful stage レシピ
3. `site/content/cookbook/cep-patterns.mdx` — CEP パターンレシピ集
4. `driver.rs` に `v55800_tests` を追加（3 件: `docs_streaming_v2_page_exists` / `cookbook_stateful_pipeline_exists` / `cookbook_cep_patterns_exists`）

> **注記**: ロードマップ完了条件の「テスト 2 件」はコードレビュー対応で 3 件に増加した（`cookbook_cep_patterns_exists` 追加）。
> ベーステスト数はロードマップ記載 3221（3219 + 2）ではなく 3222（3219 + 3）が実績値。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.8.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v55.8.0 行
- ベーステスト数: **3219**（v55.7.0 完了時点の実績値）
- 目標テスト数: **3222**（+3、コードレビュー対応で 2 → 3 件）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `Window.tumbling` / `Window.sliding` + Exactly-once 統合 | v55.1.0 | 実装済み |
| `Window.session` / `Watermark` 本番品質化 | v55.2.0 | 実装済み |
| Exactly-once チェックポイント（`checkpoint_store`） | v55.3.0 | 実装済み |
| `Stream.join_inner` / `Stream.join_left` | v55.4.0 | 実装済み |
| `State.get` / `State.set` / `State.get_or_default` | v55.5.0 | 実装済み |
| `CEP.sequence` / `CEP.skip_until` | v55.6.0 | 実装済み |
| `RESUME_FROM_CHECKPOINT` thread-local + API | v55.7.0 | 実装済み |
| `streaming-v2.mdx` / `stateful-pipeline.mdx` / `cep-patterns.mdx` | — | **本バージョンで追加** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.8.0"
```

---

### 2. `site/content/docs/runtime/streaming-v2.mdx` — 概要記事

ウィンドウ（Tumbling/Sliding/Session）・ウォーターマーク・Exactly-once チェックポイント・
Stateful Stage・CEP・Stream Join の概要をカバー。
以下キーワードを含む:
- `"Streaming Native 2.0"` — タイトル・概要に記載
- `"CEP"` — CEP.sequence / CEP.skip_until の説明
- `"checkpoint"` — Exactly-once セクションに記載
- `"Stateful"` — Stateful Stage セクションに記載

型注釈の正確性:
- `Window.tumbling` を使う stage の戻り型は `Stream<List<Event>>`（`Stream<Int>` は不正確なため使用禁止）

---

### 3. `site/content/cookbook/stateful-pipeline.mdx` — Stateful レシピ

`State.get` / `State.set` / `State.get_or_default` を使った 3 パターンのレシピ:
- ユーザーごとのイベントカウント
- 累積合計（running sum）
- セッション状態（最後のアクティブ時刻）

以下キーワードを含む: `"State.get"` / `"State.set"` / `"State.get_or_default"`

---

### 4. `site/content/cookbook/cep-patterns.mdx` — CEP レシピ集

`CEP.sequence` / `CEP.skip_until` を使った 4 パターンのレシピ:
- 注文→決済の連続検出（CEP.sequence 2 述語）
- エラーイベント以降を抽出（CEP.skip_until、inclusive セマンティクス）
- 3 ステップシーケンス検出
- Stateful CEP（状態永続化）

以下キーワードを含む: `"CEP.sequence"` / `"CEP.skip_until"`

---

### 5. `fav/src/driver.rs` — `v55800_tests` 追加

`v55700_tests` の直前に挿入する（逆順挿入の慣行に従う）。

```rust
// -- v55800_tests (v55.8.0) -- ドキュメントサイト Streaming 2.0 記事 --
#[cfg(test)]
mod v55800_tests {
    #[test]
    fn docs_streaming_v2_page_exists() {
        let src = include_str!("../../site/content/docs/runtime/streaming-v2.mdx");
        assert!(src.contains("Streaming Native 2.0"), ...);
        assert!(src.contains("CEP"), ...);
        assert!(src.contains("checkpoint"), ...);
        assert!(src.contains("Stateful"), ...);
    }

    #[test]
    fn cookbook_stateful_pipeline_exists() {
        let src = include_str!("../../site/content/cookbook/stateful-pipeline.mdx");
        assert!(src.contains("State.get"), ...);
        assert!(src.contains("State.set"), ...);
        assert!(src.contains("State.get_or_default"), ...);
    }

    #[test]
    fn cookbook_cep_patterns_exists() {
        let src = include_str!("../../site/content/cookbook/cep-patterns.mdx");
        assert!(src.contains("CEP.sequence"), ...);
        assert!(src.contains("CEP.skip_until"), ...);
    }
}
```

> **注記**: `include_str!` パスは `fav/src/driver.rs` から見て `../../site/content/...` となる。

---

## テスト仕様

### `docs_streaming_v2_page_exists`

`streaming-v2.mdx` に以下が含まれることを検証:
- `"Streaming Native 2.0"`
- `"CEP"`
- `"checkpoint"`
- `"Stateful"`

### `cookbook_stateful_pipeline_exists`

`stateful-pipeline.mdx` に以下が含まれることを検証:
- `"State.get"`（`State.get_or_default` も包含）
- `"State.set"`
- `"State.get_or_default"`

### `cookbook_cep_patterns_exists`

`cep-patterns.mdx` に以下が含まれることを検証:
- `"CEP.sequence"`
- `"CEP.skip_until"`

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（3222 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `docs_streaming_v2_page_exists` pass
- `cookbook_stateful_pipeline_exists` pass
- `cookbook_cep_patterns_exists` pass（コードレビュー対応で追加）
- `streaming-v2.mdx` の型注釈が正確（`Window.tumbling` → `Stream<List<Event>>`）
- `CHANGELOG.md` に v55.8.0 エントリが追加されている
- `versions/current.md` が v55.8.0 / 3222 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.8.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.8.0 実績欄も COMPLETE に更新

---

## 備考

- `include_str!` はコンパイル時にファイルを埋め込むため、ファイルが存在しない場合はコンパイルエラーになる。
  テストを先に書いてから MDX を作成する順序でも可。
- `site/content/docs/runtime/` ディレクトリには `parallel.mdx`（v51.8.0 で追加済み）が存在する。
  `streaming-v2.mdx` は同ディレクトリへの追加。
- ロードマップの完了条件テスト数（3221）はコードレビュー対応（+1）により実績が 3222 となった。
  ロードマップは実績欄に 3222 を記録する（完了条件欄の 3221 はそのまま残す）。
- Favnir 言語コードサンプルの注意点:
  - `[...]` はリスト内包記法（リストリテラルではない）
  - リスト生成には `collect { yield x; }` または `List.range` を使う
  - `let` キーワードは存在しない（`bind val <- expr` を使う）
