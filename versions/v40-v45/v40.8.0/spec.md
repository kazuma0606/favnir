# v40.8.0 仕様書 — Streaming cookbook

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.8.0 |
| テーマ | Streaming Foundations — Streaming cookbook |
| 前バージョン | v40.7.0（2835 tests） |
| 目標テスト数 | 2838（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.8.0 |

---

## 背景と目的

v40.7.0 で `fav bench --stream` を追加した。
本バージョンではストリーミング関連の cookbook 記事を 2 件追加し、
ユーザーがストリームパイプラインの構築方法を参照できるようにする。

- `window-aggregation.mdx` — ウィンドウ集計パイプラインの使い方
- `kafka-streaming.mdx` — Kafka Streams ウィンドウ消費パイプラインの使い方

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `site/content/cookbook/window-aggregation.mdx` | 新規作成 |
| `site/content/cookbook/kafka-streaming.mdx` | 新規作成 |
| `fav/Cargo.toml` | version: `40.7.0` → `40.8.0` |
| `CHANGELOG.md` | `[v40.8.0]` エントリ追加（`[v40.7.0]` の直後） |
| `fav/src/driver.rs` | v40700_tests stub 化 + v40800_tests 追加 |

---

## `window-aggregation.mdx` 設計

フロントマター + コード例 + 関連 Rune の構成。既存 cookbook（`llm-streaming.mdx`）のスタイルに合わせる。

```markdown
---
title: "ウィンドウ集計パイプライン"
description: "`stream.tumbling_window` でイベントをウィンドウ単位に集計するパイプラインを構築する"
---

# ウィンドウ集計パイプライン

`stream.tumbling_window` を使ってイベントストリームをタンブリングウィンドウで集計します。

## コード例

...（タンブリングウィンドウの fav コード例）...

## 関連 Rune

- [`stream`](/docs/runes/stream) — ウィンドウ関数（tumbling_window / sliding_window / session_window）
```

---

## `kafka-streaming.mdx` 設計

`consume_windowed` を使った Kafka Streams ウィンドウ消費パイプラインのサンプル。

```markdown
---
title: "Kafka Streams ウィンドウ消費"
description: "`kafka.consume_windowed` でウィンドウ単位に Kafka メッセージを消費するパイプラインを構築する"
---

# Kafka Streams ウィンドウ消費

`kafka.consume_windowed` を使って Kafka トピックからウィンドウ単位でメッセージを消費します。

## コード例

...（consume_windowed の fav コード例）...

## 関連 Rune

- [`kafka`](/docs/runes/kafka) — Kafka 接続・消費・produce
```

---

## テスト設計（v40800_tests）

```rust
#[cfg(test)]
mod v40800_tests {
    #[test]
    fn cargo_toml_version_is_40_8_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.8.0"), "Cargo.toml must contain version 40.8.0");
    }

    #[test]
    fn changelog_has_v40_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.8.0]"), "CHANGELOG.md must contain [v40.8.0]");
    }

    #[test]
    fn cookbook_window_aggregation_exists() {
        let src = include_str!("../../site/content/cookbook/window-aggregation.mdx");
        assert!(src.contains("tumbling_window"), "window-aggregation.mdx must mention tumbling_window");
    }
}
```

`v40800_tests` は `include_str!` のみ使用のため `use super::*` 不要。

テスト数: 2835 + 3 = **2838**

**注**: ロードマップは「推定 2834（1 件）」と記載しているが、v40.7.0 の実績（2835）を起点に 3 テスト構成（2838）とする（確立パターンに合わせる）。

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 | 検証方法 |
|---|------|----------|
| 1 | `window-aggregation.mdx` が存在し `tumbling_window` を含む | `cookbook_window_aggregation_exists` テスト |
| 2 | `kafka-streaming.mdx` が存在する | 手動確認 |
| 3 | `Cargo.toml` の version が `40.8.0` | `cargo_toml_version_is_40_8_0` テスト |
| 4 | `CHANGELOG.md` に `[v40.8.0]` エントリが存在する | `changelog_has_v40_8_0` テスト |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2838） | cargo test |
| 6 | `v40800_tests` 3 件すべて pass | cargo test |

---

## ロードマップとの差異

- ロードマップは「推定 2834（1 件）」と記載しているが、実績 2835 を起点に 3 テスト構成（2838）とする。
- `kafka-streaming.mdx` は直接テストしないが（テスト数を最小限に抑えるため）、`window-aggregation.mdx` のテストで cookbook 追加の完了を代表的に確認する。
