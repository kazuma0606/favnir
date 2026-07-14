# Roadmap v40.1.0 〜 v41.0.0 — Streaming Foundations

Date: 2026-07-11
Status: 計画中（v40.0 完了時点）、詳細は実装時に確定

---

## 目標

v40.0「Enterprise Governance」で「チームで安全に運用できる」を実現した。
このフェーズは **「ウィンドウ操作・Watermark・out-of-order イベント処理の基盤を整備し、型安全なストリームパイプラインを書けるようにする」** を実現する。

---

## バージョン計画

### v40.1.0 — `tumbling_window` / `sliding_window` ✅ 完了（2026-07-11、2817 tests）

```favnir
stage Aggregate {
  bind windowed <- Stream.tumbling_window(events, 60)
  bind sums     <- List.map(windowed, |w| List.sum(w))
}
```

`Stream.tumbling_window(stream, seconds)` — 固定幅ウィンドウ
`Stream.sliding_window(stream, size, step)` — スライドウィンドウ

**完了条件**: Rust テスト 3 件（推定 2817 tests passed, 0 failed）

---

### v40.2.0 — `session_window` ✅ 完了（2026-07-11、2820 tests）

```favnir
bind sessions <- Stream.session_window(events, gap: 30)
// 30秒アイドルでウィンドウを閉じる
```

`Stream.session_window(stream, gap: seconds)` — セッションウィンドウ

**完了条件**: Rust テスト 3 件（推定 2820 tests passed, 0 failed）

---

### v40.3.0 — `Event<T>` + timestamp フィールド ✅ 完了（2026-07-11、2823 tests）

`Event<T>` 型に `timestamp: Int` フィールドを追加。ウィンドウ演算の時刻基準として使用。

```favnir
type Event<T> = {
  value: T
  timestamp: Int   // Unix epoch (ms)
}
```

**完了条件**: Rust テスト 3 件（推定 2823 tests passed, 0 failed）

---

### v40.4.0 — Out-of-order イベント処理 ✅ 完了（2026-07-11、2826 tests）

遅延イベントの許容（`late_tolerance`）と drop ポリシー（`drop` / `reprocess`）。

```favnir
bind valid <- Stream.with_late_policy(events,
  tolerance: 5,
  policy: "drop")
```

**完了条件**: Rust テスト 3 件（推定 2826 tests passed, 0 failed）

---

### v40.5.0 — `fav.toml [stream]` セクション ✅ COMPLETE（2026-07-11）

```toml
[stream]
watermark_delay = 5     # 秒
late_policy = "drop"    # drop | reprocess
```

`fav.toml` パーサーに `[stream]` セクション追加。`inject_stream_config` で pipeline に伝播。

**完了条件**: Rust テスト 3 件（推定 2829 tests passed, 0 failed）

---

### v40.6.0 — Kafka / Redis Streams window 対応 ✅ COMPLETE（2026-07-11）

既存 Kafka・Redis Rune にウィンドウ集計メソッドを追加。
`runes/kafka/kafka.fav` に `Kafka.consume_windowed` 追加。

**完了条件**: Rust テスト 2 件（推定 2831 tests passed, 0 failed）

---

### v40.7.0 — `fav bench --stream` ✅ COMPLETE（2026-07-11）

ストリームパイプラインのスループット / レイテンシ計測コマンド。
`cmd_bench` に `--stream` フラグ追加。

**完了条件**: Rust テスト 2 件（推定 2833 tests passed, 0 failed）

---

### v40.8.0 — Streaming cookbook ✅ COMPLETE（2026-07-11）

- `site/content/cookbook/window-aggregation.mdx`
- `site/content/cookbook/kafka-streaming.mdx`

**完了条件**: Rust テスト 1 件（推定 2834 tests passed, 0 failed）
**実績**: 2838 tests passed, 0 failed（v40800_tests 3/3 pass）

---

### v40.9.0 — v41.0 前調整・安定化 ✅ COMPLETE（2026-07-11）

コードフリーズ（新規機能追加なし）。`site/content/docs/streaming-foundations.mdx` 新規作成。

**完了条件**: meta テスト 2 件（推定 2836 tests passed, 0 failed）
**実績**: 2840 tests passed, 0 failed（v40900_tests 2/2 pass）

---

### v41.0.0 — Streaming Foundations 宣言 ★クリーンアップ ✅ COMPLETE（2026-07-11）

**宣言文（暫定）**:

> 「`tumbling_window` / `sliding_window` / `session_window` でウィンドウ集計を型安全に書ける。
>  `Event<T>` の timestamp と Watermark で out-of-order イベントを制御できる。
>
>  これが Favnir v41.0 — Streaming Foundations の姿である。」

**完了条件**:
- v40.1〜v40.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ 2841 + 4 = **2845**）（v40.9.0 実績 2841 起点）
- `v41000_tests` 4 件 pass（内訳: `cargo_toml_version_is_41_0_0` / `changelog_has_v41_0_0` / `milestone_has_streaming_foundations` / `readme_mentions_streaming_foundations`）
- `MILESTONE.md` に `"Streaming Foundations"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v39.1-v40.0.md`
- 達成宣言: `MILESTONE.md`
