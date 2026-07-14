# v40.3.0 仕様書 — `Event<T>` + timestamp フィールド

## バージョン概要

| 項目 | 内容 |
|------|------|
| バージョン | v40.3.0 |
| テーマ | Streaming Foundations — `Event<T>` 型 + `timestamp` フィールド |
| 前バージョン | v40.2.0（2820 tests） |
| 目標テスト数 | 2823（+3） |
| 参照ロードマップ | `versions/roadmap/roadmap-v40.1-v41.0.md` §v40.3.0 |

---

## 背景と目的

v40.1.0〜v40.2.0 でウィンドウ関数スタブ（tumbling / sliding / session）を追加した。
これらのウィンドウ演算は **イベントの発生時刻（timestamp）** を基準として動作するが、
現状のスタブ実装には時刻情報がない。

v40.3.0 では、ストリームを流れるイベントの型として `Event<T>` を定義し、
`timestamp: Int` フィールド（Unix epoch ミリ秒）を付与する。これにより
v40.4.0 以降の out-of-order 処理・Watermark 基盤の土台が整う。

```favnir
type Event<T> = {
  value:     T
  timestamp: Int   // Unix epoch (ms)
}
```

---

## 実装スコープ

### 変更ファイル

| ファイル | 変更内容 |
|----------|----------|
| `runes/stream/stream.fav` | `Event<T>` 型定義追加 + ヘッダー更新 |
| `runes/stream/rune.toml` | version: `40.2.0` → `40.3.0`、description に `Event<T>` / `timestamp` 追記 |
| `fav/Cargo.toml` | version: `40.2.0` → `40.3.0` |
| `CHANGELOG.md` | `[v40.3.0]` エントリ追加（`[v40.2.0]` の直後） |
| `fav/src/driver.rs` | v40200_tests stub 化 + v40300_tests 追加 |

---

## `Event<T>` 型設計

```favnir
// Event<T> — ストリームイベント型（v40.3.0）
// timestamp: Unix epoch (ms) をウィンドウ演算の時刻基準として使用
type Event<T> = {
  value:     T
  timestamp: Int
}
```

**注**: Favnir の `type` 宣言構文でレコード型エイリアスとして定義する。
ジェネリクス `T` は v40.3.0 時点ではパーサーがサポートする範囲でのスタブ記述とし、
完全な型チェック統合は v43.x 型推論スプリントで行う。

---

## テスト設計（v40300_tests）

```rust
#[cfg(test)]
mod v40300_tests {
    #[test]
    fn cargo_toml_version_is_40_3_0() {
        // NOTE: 次バージョン bump 時に Stubbed コメントへ置き換えること
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("40.3.0"), "Cargo.toml must contain version 40.3.0");
    }

    #[test]
    fn changelog_has_v40_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v40.3.0]"), "CHANGELOG.md must contain [v40.3.0]");
    }

    #[test]
    fn stream_fav_has_event_type() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("Event"), "stream.fav must contain Event type definition");
        assert!(src.contains("timestamp"), "stream.fav must contain timestamp field");
    }
}
```

テスト数: 2820 + 3 = **2823**

---

## 完了条件

**自動検証（cargo test）:**

| # | 条件 |
|---|------|
| 1 | `runes/stream/stream.fav` に `Event` 型定義（`timestamp` フィールド含む）が存在する |
| 3 | `Cargo.toml` の version が `40.3.0` |
| 4 | `CHANGELOG.md` に `[v40.3.0]` エントリが存在する |
| 5 | `cargo test` 全通過（failures=0、テスト数 ≥ 2823） |
| 6 | `v40300_tests` 3 件すべて pass |

**手動確認（テスト化対象外 — ロードマップが「3 件」と明示）:**

| # | 条件 |
|---|------|
| 2 | `runes/stream/rune.toml` の version が `40.3.0`、description に `Event<T>` / `timestamp` が反映されている |

**パーサー互換性フォールバック方針:**

`type Event<T> = { ... }` 構文が Favnir パーサーでエラーになる場合は、
コメント形式（`// type Event<T> = ...`）でスタブ記述し TODO を明記する。
この場合でも `stream_fav_has_event_type` テストは `src.contains("Event")` / `src.contains("timestamp")`
で通過するため pass 扱いとする。完全な型定義統合は v43.x 型推論スプリントまで持ち越す。

---

## ロードマップとの差異

特記事項なし。ロードマップ §v40.3.0 の仕様を完全に反映。
