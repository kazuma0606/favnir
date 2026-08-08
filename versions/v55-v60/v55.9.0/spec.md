# Spec — v55.9.0 — 安定化・コードフリーズ（Streaming Native 2.0 前調整）

## 概要

v55.9.0 は Streaming Native 2.0 スプリント（v55.1〜v55.9）の最終バージョン。
全 lint / clippy クリーン確認と `streaming-native2-overview.mdx` 骨子作成を行い、
v55.1〜v55.8 の全テストが通過していることを確認して v56.0 に備える。

具体的には以下を実装する：
1. `site/content/docs/streaming-native2-overview.mdx` — Streaming Native 2.0 宣言と概要（骨子）
2. `driver.rs` に `v55900_tests` を追加（2 件: `cargo_toml_version_is_55_9_0` / `streaming_native2_overview_exists`）

---

## ロードマップ参照

- `versions/roadmap/roadmap-v55.1-v56.0.md` — v55.9.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v55.9.0 行
- ベーステスト数: **3222**（v55.8.0 完了時点の実績値）
- 目標テスト数: **3224**（+2）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| ウィンドウ + Exactly-once 統合 | v55.1.0 | 実装済み |
| セッションウィンドウ + ウォーターマーク | v55.2.0 | 実装済み |
| Exactly-once チェックポイント | v55.3.0 | 実装済み |
| ストリーム結合（join_inner / join_left） | v55.4.0 | 実装済み |
| Stateful stage（State API） | v55.5.0 | 実装済み |
| CEP 統合（sequence / skip_until） | v55.6.0 | 実装済み |
| Checkpoint / Replay API | v55.7.0 | 実装済み |
| Streaming 2.0 ドキュメント（MDX 3 件） | v55.8.0 | 実装済み |
| `streaming-native2-overview.mdx` | — | **本バージョンで追加** |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "55.9.0"
```

---

### 2. `site/content/docs/streaming-native2-overview.mdx` — 骨子作成

以下を含む:
- ロードマップ記載の宣言文（引用ブロック形式）
- v55.1〜v55.8 の機能一覧テーブル
- fav.toml クイックスタート設定例
- Stateful stage コード例
- CEP stage コード例（述語関数を事前定義すること）
- チェックポイント CLI 操作例
- 詳細ドキュメントへのリンク（streaming-v2 / stateful-pipeline / cep-patterns）
- 次のステップ（v56.0 予定機能）

以下キーワードを含む:
- `"Streaming Native 2.0"` — タイトル・宣言文に記載
- `"Exactly-once"` — 機能一覧・設定例に記載
- `"CEP"` — CEP 統合行・コード例に記載
- `"Stateful"` — Stateful stage 行・コード例に記載

CEP コード例の注意点:
- `yield is_start; yield is_end;` のように述語関数を参照する場合、
  直前に `fn is_start(e: Event) -> Bool { ... }` / `fn is_end(e: Event) -> Bool { ... }` を定義すること

---

### 3. `fav/src/driver.rs` — `v55900_tests` モジュール追加

`v55800_tests` の直前（`// -- v55800_tests` コメント行の前）に挿入する。

```rust
// -- v55900_tests (v55.9.0) -- 安定化・コードフリーズ（Streaming Native 2.0 前調整）--
#[cfg(test)]
mod v55900_tests {
    #[test]
    fn cargo_toml_version_is_55_9_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"55.9.0\""),
            "Cargo.toml version should be 55.9.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn streaming_native2_overview_exists() {
        let src = include_str!("../../site/content/docs/streaming-native2-overview.mdx");
        assert!(src.contains("Streaming Native 2.0"), "streaming-native2-overview.mdx must mention Streaming Native 2.0");
        assert!(src.contains("Exactly-once"), "streaming-native2-overview.mdx must mention Exactly-once");
        assert!(src.contains("CEP"), "streaming-native2-overview.mdx must mention CEP");
        assert!(src.contains("Stateful"), "streaming-native2-overview.mdx must mention Stateful");
    }
}
```

> **注記**: `include_str!` パスは `fav/src/driver.rs` から見て
> - `../Cargo.toml` → `fav/Cargo.toml`
> - `../../site/content/docs/streaming-native2-overview.mdx` → `favnir/site/content/docs/...`

---

## テスト仕様

### `cargo_toml_version_is_55_9_0`

`fav/Cargo.toml` に `version = "55.9.0"` が含まれることを検証。
エラーメッセージには最初に `version` を含む行を表示する。

### `streaming_native2_overview_exists`

`streaming-native2-overview.mdx` に以下が含まれることを検証:
- `"Streaming Native 2.0"`
- `"Exactly-once"`
- `"CEP"`
- `"Stateful"`

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（3224 tests passed, 0 failed）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_55_9_0` pass
- `streaming_native2_overview_exists` pass
- `streaming-native2-overview.mdx` に宣言文・機能一覧・CEP 述語定義付きコード例を含む
- `CHANGELOG.md` に v55.9.0 エントリが追加されている
- `versions/current.md` が v55.9.0 / 3224 tests を反映
- `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.9.0 実績を COMPLETE に更新
- `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.9.0 実績欄も COMPLETE に更新

---

## 備考

- 本バージョンは「コードフリーズ」のため新機能追加は行わない。
  ドキュメント骨子作成・バージョンアップ・テストのみ実施する。
- `streaming-native2-overview.mdx` は骨子（骨格ドキュメント）であり、
  v56.0 公式宣言時に詳細化する。宣言文の最終行
  「これが Favnir v56.0 — Streaming Native 2.0 の姿である。」は v56.0 で追加する。
- コードレビュー対応: CEP コード例で `is_start` / `is_end` を未定義のまま参照しないこと
  （`fn is_start(e: Event) -> Bool { ... }` を先に定義すること）。
- Favnir 言語注意点:
  - `[...]` はリスト内包記法（リストリテラルではない）
  - リスト生成には `collect { yield x; }` または `List.range` を使う
  - `let` キーワードは非対応 → `bind val <- expr` を使う
