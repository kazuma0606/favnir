# v40.9.0 仕様書 — v41.0 前調整・安定化

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.9.0 |
| テーマ | Streaming Foundations — v41.0 前調整・安定化（コードフリーズ） |
| 前バージョン | v40.8.0（2838 tests） |
| 目標テスト数 | 2840（+2） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.9.0 |

---

## 背景と目的

v40.8.0 でストリーミング関連 cookbook を 2 件追加した。
本バージョンはコードフリーズ版であり、新規機能追加は行わない。
`site/content/docs/streaming-foundations.mdx` を新規作成し、
v40.1〜v40.8 で追加したストリーミング機能の全体概観ドキュメントを整備する。

次バージョン v41.0.0 は「Streaming Foundations」マイルストーン宣言（`cargo clean` 含む）を行う。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `site/content/docs/streaming-foundations.mdx` | 新規作成 |
| `fav/Cargo.toml` | version: `40.8.0` → `40.9.0` |
| `CHANGELOG.md` | `[v40.9.0]` エントリ追加（`[v40.8.0]` の直後） |
| `fav/src/driver.rs` | `cargo_toml_version_is_40_8_0` stub 化 + v40900_tests 追加 |

---

## `streaming-foundations.mdx` 設計

v40.1〜v40.8 の成果物をまとめる概観ドキュメント。既存 docs ページ（`site/content/docs/`）のスタイルに合わせる。

```markdown
---
title: "Streaming Foundations"
description: "Favnir のストリーミング基盤 — ウィンドウ集計・Watermark・out-of-order イベント処理"
---

# Streaming Foundations

Favnir v40.x で整備されたストリーミング機能の概観。

## ウィンドウ関数

- `stream.tumbling_window` — 固定幅タンブリングウィンドウ（v40.1）
- `stream.sliding_window` — スライドウィンドウ（v40.1）
- `stream.session_window` — セッションウィンドウ（v40.2）

## イベント型と Watermark

...（Event<T> / timestamp / late_policy の説明）...

## 関連 cookbook

- [ウィンドウ集計パイプライン](/cookbook/window-aggregation)
- [Kafka Streams ウィンドウ消費](/cookbook/kafka-streaming)
```

`streaming-foundations` という文字列をドキュメント内に含めること（v41000_tests の `streaming_foundations_doc_exists` テストで参照される可能性を考慮）。

---

## テスト設計（v40900_tests）

```rust
#[cfg(test)]
mod v40900_tests {
    #[test]
    fn cargo_toml_version_is_40_9_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.9.0"), "Cargo.toml must contain version 40.9.0");
    }

    #[test]
    fn changelog_has_v40_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.9.0]"), "CHANGELOG.md must contain [v40.9.0]");
    }
}
```

`v40900_tests` は `include_str!` のみ使用のため `use super::*` 不要。

テスト数: 2838 + 2 = **2840**

**注**: ロードマップは「推定 2836（+2 件）」と記載しているが、v40.8.0 の実績（2838）を起点に 2 テスト構成（2840）とする（確立パターンに合わせる）。

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 | 検証方法 |
|---|------|----------|
| 1 | `Cargo.toml` の version が `40.9.0` | `cargo_toml_version_is_40_9_0` テスト |
| 2 | `CHANGELOG.md` に `[v40.9.0]` エントリが存在する | `changelog_has_v40_9_0` テスト |
| 3 | `streaming-foundations.mdx` が存在する | 手動確認 |
| 4 | `cargo test` 全通過（failures=0、テスト数 ≥ 2840） | cargo test |
| 5 | `v40900_tests` 2 件すべて pass | cargo test |

---

## ロードマップとの差異

- ロードマップは「推定 2836（2 件）」と記載しているが、実績 2838 を起点に 2 テスト構成（2840）とする。
- `streaming-foundations.mdx` は直接テストしない（次バージョン v41.0.0 の `streaming_foundations_doc_exists` テストで間接的に検証される）。
