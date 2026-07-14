# v41.0.0 仕様書 — Streaming Foundations 宣言

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v41.0.0 |
| テーマ | Streaming Foundations 宣言（★クリーンアップ） |
| 前バージョン | v40.9.0（2841 tests） |
| 目標テスト数 | 2845（+4） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v41.0.0 |

---

## 背景と目的

v40.1〜v40.9 で整備したストリーミング基盤を「Streaming Foundations」として正式宣言するバージョン。

**宣言文:**

> 「`tumbling_window` / `sliding_window` / `session_window` でウィンドウ集計を型安全に書ける。
>  `Event<T>` の timestamp と Watermark で out-of-order イベントを制御できる。
>
>  これが Favnir v41.0 — Streaming Foundations の姿である。」

本バージョンの変更は以下の通り。Rust コードの機能追加はなし。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `MILESTONE.md` | `v41.0.0 — Streaming Foundations` エントリ追加 |
| `README.md` | `Streaming Foundations`（v41.0）の記述追加 |
| `fav/Cargo.toml` | version: `40.9.0` → `41.0.0` |
| `CHANGELOG.md` | `[v41.0.0]` エントリ追加（`[v40.9.0]` の直後） |
| `fav/src/driver.rs` | `cargo_toml_version_is_40_9_0` stub 化 + v41000_tests 追加 |

---

## `MILESTONE.md` 追記設計

v40.0.0 のエントリの直前に v41.0.0 エントリを追加する（最新が先頭）。

```markdown
## v41.0.0 — Streaming Foundations（2026-07-11）

> 「`tumbling_window` / `sliding_window` / `session_window` でウィンドウ集計を型安全に書ける。
>  `Event<T>` の timestamp と Watermark で out-of-order イベントを制御できる。
>
>  これが Favnir v41.0 — Streaming Foundations の姿である。」

v41.0.0 をもって、Favnir の **Streaming Foundations** を正式に宣言する。

### 達成コンポーネント（v40.1〜v40.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| tumbling_window / sliding_window | v40.1 | 固定幅・スライドウィンドウ |
| session_window | v40.2 | セッションウィンドウ |
| Event<T> + timestamp | v40.3 | イベント型に時刻基準フィールド追加 |
| Out-of-order 処理 | v40.4 | late_tolerance / drop / reprocess |
| fav.toml [stream] | v40.5 | プロジェクト設定でストリーム設定管理 |
| Kafka / Redis Streams 対応 | v40.6 | consume_windowed 追加 |
| fav bench --stream | v40.7 | ストリームパイプライン計測スタブ |
| Streaming cookbook | v40.8 | window-aggregation / kafka-streaming MDX |
| 安定化 | v40.9 | streaming-foundations.mdx ドキュメント整備 |

**宣言日**: 2026-07-11
```

---

## `README.md` 追記設計

既存の `v40.0` 記述の後に `v41.0` の一行を追加する（README 内の milestone 記述箇所）。
`"Streaming Foundations"` という文字列が README に含まれるようにする。

---

## テスト設計（v41000_tests）

```rust
#[cfg(test)]
mod v41000_tests {
    #[test]
    fn cargo_toml_version_is_41_0_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("41.0.0"), "Cargo.toml must contain version 41.0.0");
    }

    #[test]
    fn changelog_has_v41_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v41.0.0]"), "CHANGELOG.md must contain [v41.0.0]");
    }

    #[test]
    fn milestone_has_streaming_foundations() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Streaming Foundations"), "MILESTONE.md must contain Streaming Foundations");
    }

    #[test]
    fn readme_mentions_streaming_foundations() {
        let src = include_str!("../../README.md");
        assert!(src.contains("Streaming Foundations"), "README.md must mention Streaming Foundations");
    }
}
```

`v41000_tests` は `include_str!` のみ使用のため `use super::*` 不要。

テスト数: 2841 + 4 = **2845**

**注**: ロードマップは「≥ 2836 + 4 = 2840」と記載していたが、v40.9.0 では code-reviewer 指摘（`streaming_foundations_doc_exists` テスト追加）により 2840 → 2841 に増加した経緯がある。その実績（2841）を起点に 4 テスト構成（2845）とする（確立パターンに合わせる）。ロードマップの完了条件も 2845 に更新済み。

---

## `cargo clean` 手順

v41.0.0 は ★クリーンアップ版。`cargo test` 全通過後に以下を実施する。

1. `cargo clean` 実行
2. `fav/tmp/hello.fav` を復元（`cargo clean` で削除される）
   - 正しい内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`
3. `cargo test` を再実行し 2845 passed / 0 failed を確認

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 | 検証方法 |
|---|------|----------|
| 1 | `Cargo.toml` の version が `41.0.0` | `cargo_toml_version_is_41_0_0` テスト |
| 2 | `CHANGELOG.md` に `[v41.0.0]` エントリが存在する | `changelog_has_v41_0_0` テスト |
| 3 | `MILESTONE.md` に `Streaming Foundations` が存在する | `milestone_has_streaming_foundations` テスト |
| 4 | `README.md` に `Streaming Foundations` が存在する | `readme_mentions_streaming_foundations` テスト |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2845） | cargo test |
| 6 | `v41000_tests` 4 件すべて pass | cargo test |
| 7 | `cargo clean` 完了・`hello.fav` 復元・`cargo test` 再通過 | 手動確認 |

---

## ロードマップとの差異

- ロードマップは「≥ 2836 + 4 = 2840」と記載していたが、v40.9.0 の code-reviewer 指摘により実績が 2841 に増加したため、2841 + 4 = 2845 を採用する。ロードマップ §v41.0.0 の完了条件も 2845 に更新済み。
