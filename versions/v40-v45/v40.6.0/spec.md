# v40.6.0 仕様書 — Kafka Streams `consume_windowed` 追加

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.6.0 |
| テーマ | Streaming Foundations — Kafka / Redis Streams window 対応 |
| 前バージョン | v40.5.0（2829 tests） |
| 目標テスト数 | 2832（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.6.0 |

---

## 背景と目的

v40.5.0 で `fav.toml [stream]` セクション（`watermark_delay` / `late_policy`）を実装した。
本バージョンでは既存の Kafka Rune（`runes/kafka/kafka.fav`）に
ウィンドウ集計メソッド `consume_windowed` を追加する。

`consume_windowed` は指定した秒数のウィンドウ内でメッセージを収集し、
集計パイプラインに渡すためのスタブとして機能する。
実際の集計ロジックは v40.7 以降で実装。

また、kafka rune が `rune.toml` を持っていないため、本バージョンで新規作成する。
Redis Rune にも同様の `consume_windowed` スタブを追加する（ロードマップタイトル「Kafka / Redis Streams window 対応」の要件を満たすため）。

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `runes/kafka/kafka.fav` | `consume_windowed` スタブ追加・ヘッダー更新 |
| `runes/kafka/rune.toml` | 新規作成（kafka rune のメタ情報） |
| `runes/redis/redis.fav` | `consume_windowed` スタブ追加・ヘッダー更新 |
| `fav/Cargo.toml` | version: `40.5.0` → `40.6.0` |
| `CHANGELOG.md` | `[v40.6.0]` エントリ追加（`[v40.5.0]` の直後） |
| `fav/src/driver.rs` | v40500_tests stub 化 + v40600_tests 追加 |

---

## `consume_windowed` 設計

既存の `consume_batch` と同パターンのスタブとして追加する。

```
// ウィンドウ単位でメッセージを収集する（v40.6.0 スタブ）
// window_secs 秒分のメッセージを JSON 配列文字列で返す。
// 実際のウィンドウ集計ロジックは v40.7 以降で実装。
public fn consume_windowed(conn: KafkaConn, topic: String, group_id: String, window_secs: Int) -> Result<String, String> {
    Kafka.consume_batch_raw(conn, topic, window_secs)
}
```

ヘッダーコメントを `// runes/kafka/kafka.fav — Kafka Rune (v40.6.0)` に更新。

---

## `rune.toml` 設計

```toml
[rune]
name        = "kafka"
version     = "40.6.0"
entry       = "kafka.fav"
description = "Kafka Rune — connect / produce / consume_one / consume_batch / consume_windowed / create_topic / KafkaConn"
```

---

## テスト設計（v40600_tests）

```rust
#[cfg(test)]
mod v40600_tests {
    #[test]
    fn cargo_toml_version_is_40_6_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.6.0"), "Cargo.toml must contain version 40.6.0");
    }

    #[test]
    fn changelog_has_v40_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.6.0]"), "CHANGELOG.md must contain [v40.6.0]");
    }

    #[test]
    fn kafka_fav_has_consume_windowed() {
        let src = include_str!("../../runes/kafka/kafka.fav");
        assert!(src.contains("consume_windowed"), "kafka.fav must contain consume_windowed");
    }
}
```

テスト数: 2829 + 3 = **2832**

**注**: ロードマップは「推定 2831（2 件）」と記載しているが、CHANGELOG 検証テストを追加して 3 件（2832）とする（v40.1〜v40.5 の確立パターンに合わせる）。

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 |
|---|------|
| 1 | `runes/kafka/kafka.fav` に `consume_windowed` 関数が存在する |
| 2 | `runes/kafka/rune.toml` が存在し `name = "kafka"` を含む |
| 3 | `Cargo.toml` の version が `40.6.0` |
| 4 | `CHANGELOG.md` に `[v40.6.0]` エントリが存在する |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2832） |
| 6 | `v40600_tests` 3 件すべて pass |

---

## ロードマップとの差異

- ロードマップは「推定 2831（2 件）」と記載しているが、CHANGELOG 検証テストを追加して 3 件（2832）とする。
- kafka rune.toml が欠落していたため本バージョンで補完する（ロードマップには未記載だが軽微な整備）。
- Redis Rune にも `consume_windowed` スタブを追加し、ロードマップタイトルの「Kafka / Redis」両対応を満たす。
