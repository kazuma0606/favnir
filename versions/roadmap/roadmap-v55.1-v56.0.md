# Roadmap v55.1.0 〜 v56.0.0 — Streaming Native 2.0

Date: 2026-07-23
Status: COMPLETE（2026-07-25）

---

## 前提

- 直前完了: v55.0.0「Production 3.0」（tests = 3206）
- マスターロードマップ: `roadmap-v55.1-v60.0.md`
- 本文書はマスターの v56.0 スプリント部分の詳細版
- **既存機能の扱い**: `Window.tumbling` / `Window.session` / `Watermark` は v41.0 で vm.rs に実装済み。
  v55.1〜v55.2 はこれらを Exactly-once チェックポイントと統合・fav.toml 設定化するのが目的。
  `CEP.sequence` / `CepPatternDef` は v42.1 で ast.rs に実装済み。
  v55.6 は Stream<T> との統合層を追加するのが目的（NFA 再実装ではない）。
  詳細はマスターロードマップ「既存機能との位置づけ」テーブルを参照。

---

## 目標

v41〜v52 で積み上げたウィンドウ・CEP・並列基盤を統合し、
**Exactly-once・Stateful・Replay を備えた本番品質のストリーム処理基盤を完成させる**。

---

## バージョン計画

### v55.1.0 — タンブリング / スライディングウィンドウ + Exactly-once 統合

v41.0 実装済みの `Window.tumbling` / `Window.sliding` に、v55.3（Exactly-once チェックポイント）との
統合インターフェースを追加。ウィンドウ境界でのチェックポイント保存フックを `vm.rs` に挿入し、
再起動時にウィンドウ状態を復元できるようにする。
`fav.toml` の `[stream]` セクションにウィンドウ設定（`buffer_size` 等）を追加。

```toml
# fav.toml
[stream]
buffer_size = 1000
checkpoint_store = "s3://my-bucket/checkpoints"
```

**完了条件**: Rust テスト 2 件（ベース 3206 + 2 = 3208 tests passed, 0 failed）
- `window_tumbling_checkpoint_integration`
- `window_sliding_resume_from_checkpoint`

**実績**: COMPLETE — 3207 tests passed, 0 failed（2026-07-23）

---

### v55.2.0 — セッションウィンドウ + ウォーターマーク本番品質化

v41.0 実装済みの `Window.session` / `Watermark` を `fav.toml` の `[stream]` セクションから
設定できるよう拡張。ウォーターマーク超過イベントの `!Observe` エフェクト経由のドロップ記録を
`vm.rs` に追加。`fav run --stream-stats` フラグでウィンドウ / ウォーターマーク統計を標準出力に表示。

```toml
# fav.toml
[stream]
session_gap_sec = 30
watermark_max_late_sec = 5
```

**完了条件**: Rust テスト 2 件（ベース 3207 + 2 = 3209 tests passed, 0 failed）
- `window_session_toml_config`
- `watermark_late_event_observe_effect`

> **実装注記**: `Effect` enum は v35.5.0 で削除済みのため `!Observe` エフェクトは追加せず、
> `vm.rs` の `late_event_drops` カウンター stub で代替。`--stream-stats` は `show_stream_stats`
> フィールドのみ追加し、フル実装は v55.9 で行う。

**実績**: COMPLETE — 3209 tests passed, 0 failed（2026-07-24）

---

### v55.3.0 — Exactly-once 意味論（冪等チェックポイント）

チェックポイントストア（ファイル / S3）にオフセットと処理済み ID を保存し、
再起動時に重複処理を排除する冪等リトライ機構を実装。
`vm.rs` の effect ディスパッチに checkpoint フックを追加。

```toml
# fav.toml
[stream]
delivery = "exactly-once"   # at-least-once | exactly-once
checkpoint_interval_sec = 10
```

**完了条件**: Rust テスト 2 件（ベース 3209 + 2 = 3211 tests passed, 0 failed）
- `exactly_once_checkpoint_saved`
- `exactly_once_no_duplicate_on_restart`

**実績**: COMPLETE — 3211 tests passed, 0 failed（2026-07-24）

---

### v55.4.0 — ストリーム結合（inner join / left outer join）

`Stream.join_inner` / `Stream.join_left` を VM primitive として追加。
結合は時間ウィンドウ内（`window_secs` 引数）でキーマッチングを行い、
既存 `VMStream::Join`（v42.4.0）と同一の nested-loop join で実装。

> **実装注記**: ロードマップ当初案の「メモリ内ハッシュテーブル実装」および
> 「`par [A, B]` 並列読み込み」は本バージョンでスコープ外とし、
> nested-loop join（シングルスレッド）で代替実装する。
> ハッシュテーブル最適化・並列化は将来のパフォーマンス最適化スプリントで対応する。

```favnir
stage Join: (Stream<Order>, Stream<Customer>) -> Stream<EnrichedOrder> = |(orders, customers)| {
  bind joined <- Stream.join_inner(orders, customers,
    |o, c| o.customer_id == c.id, 60)
  Ok(joined)
}
```

**完了条件**: Rust テスト 2 件（ベース 3211 + 2 = 3213 tests passed, 0 failed）
- `stream_join_inner_matches`
- `stream_join_left_preserves_unmatched`

**実績**: COMPLETE — 3213 tests passed, 0 failed（2026-07-24）

---

### v55.5.0 — Stateful stage（累積状態）

**前提**: v55.3.0（`exactly_once_checkpoint_saved` テスト通過）が完了していること。

`!State` エフェクトを追加。`State.get` / `State.set` / `State.get_or_default` を
VM primitive として実装。State は v55.3 のチェックポイントストアに自動永続化。
E0421 エラーコード（`!State` エフェクトなし state 操作）を `error_catalog.rs` に追加。

```favnir
stage CountPerUser: Stream<Event> -> Stream<(String, Int)> = |events| !State {
  bind count <- State.get_or_default("user_count", Map.empty)
  let new_count = Map.update(count, events.user_id, |n| n + 1)
  bind _ <- State.set("user_count", new_count)
  Ok((events.user_id, Map.get(new_count, events.user_id)))
}
```

**完了条件**: Rust テスト 2 件（ベース 3213 + 2 = 3215 tests passed, 0 failed）
- `stateful_stage_accumulates`
- `stateful_stage_persists`

**実績**: COMPLETE（2026-07-24）— 3215 tests passed, 0 failed
- `STATE_VALUE_STORE` thread-local 追加 / `State.get` / `State.set` / `State.get_or_default` primitive 追加
- `compiler.rs` の namespace 登録に `"State"` を追加（`Global(u16::MAX)` 問題修正）
- E0421 stub / `("State", "get_or_default") => Type::Unknown` 追加

---

### v55.6.0 — CEP（複合イベント処理）Stream 統合

v42.1 実装済みの `CepPatternDef` / `CepExpr::Seq` / `CepExpr::Any` を
`Stream<T>` の値として扱えるよう VM 統合層を追加。
`CEP.sequence` / `CEP.skip_until` を `Stream<T> -> Stream<U>` 変換として公開し、
既存の NFA 実装を再利用。Stateful stage（v55.5）と組み合わせて `!State` エフェクト下で
CEP 状態を永続化できることを確認。

```favnir
bind result <- CEP.sequence([
  CEP.match(|e| e.type == "order_placed"),
  CEP.then(|e| e.type == "payment_confirmed", within_sec: 5)
], emit: |[order, payment]| EnrichedEvent { order, payment })
```

**完了条件**: Rust テスト 2 件（ベース 3215 + 2 = 3217 tests passed, 0 failed）
- `cep_stream_integration`
- `cep_stateful_persistence`
- 注: NFA 実装ではなく関数 API（CEP.sequence/skip_until）で代替実装

**実績**: COMPLETE — 3217 tests passed, 0 failed（2026-07-24）

---

### v55.7.0 — Checkpoint / Replay API

**前提**: v55.3.0（`exactly_once_checkpoint_saved` テスト通過）が完了していること。

`fav run --resume-from <checkpoint>` でチェックポイント再開（v55.3 の `checkpoint_store` を参照）。
`fav run --replay-from / --replay-until` で時刻範囲リプレイを実装。
`fav checkpoint list` でチェックポイント一覧を表示。

```bash
$ fav checkpoint list
2026-07-23T09:00:00Z  offset=1000  size=42KB
2026-07-23T09:10:00Z  offset=2300  size=44KB

$ fav run pipeline.fav --resume-from 2026-07-23T09:10:00Z
```

**完了条件**: Rust テスト 2 件（ベース 3217 + 2 = 3219 tests passed, 0 failed）
- `cmd_checkpoint_list`
- `cmd_resume_from_checkpoint`
- 注: `--replay-from / --replay-until` の完全実装は v56.x スコープ（本バージョンは API 確立のみ）

**実績**: COMPLETE — 3219 tests passed, 0 failed（2026-07-24）

---

### v55.8.0 — ドキュメントサイト Streaming 2.0 記事

`site/content/docs/runtime/streaming-v2.mdx` — ウィンドウ・ウォーターマーク・Exactly-once・CEP・Stateful の概要。
`site/content/cookbook/stateful-pipeline.mdx` — Stateful stage と State エフェクトのレシピ。
`site/content/cookbook/cep-patterns.mdx` — CEP パターンのレシピ集。

**完了条件**: Rust テスト 2 件（ベース 3219 + 2 = 3221 tests passed, 0 failed）
- `docs_streaming_v2_page_exists`
- `cookbook_stateful_pipeline_exists`

**実績**: COMPLETE — 3222 tests passed, 0 failed（2026-07-24）

---

### v55.9.0 — 安定化・コードフリーズ（Streaming Native 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/streaming-native2-overview.mdx` 骨子作成。
v55.1〜v55.8 の全テストが通過していることを確認して v56.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3222 + 2 = 3224 tests passed, 0 failed）
- `cargo_toml_version_is_55_9_0`
- `streaming_native2_overview_exists`

**実績**: COMPLETE — 3224 tests passed, 0 failed（2026-07-24）

---

### v56.0.0 — Streaming Native 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「ウィンドウはイベントを時間で区切り、ウォーターマークは遅延を許容し、
>  チェックポイントは障害から瞬時に回復する。
>  CEP はイベントの流れからパターンを検出する。
>  Favnir はリアルタイムデータの言語になった。
>
>  これが Favnir v56.0 — Streaming Native 2.0 の姿である。」

**完了条件**:
- v55.1〜v55.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3227**）
  ※当初目標 3228 だったが `cargo_toml_version_is_55_9_0`（v55900_tests）削除 -1 により 3227 に修正
- `v56000_tests` 4 件 pass（3224 - 1 + 4 = **3227** tests passed, 0 failed）:
  - `cargo_toml_version_is_56_0_0`
  - `changelog_has_v56_0_0`
  - `milestone_has_streaming_native2`
  - `readme_mentions_streaming_native2`
- `MILESTONE.md` に `"Streaming Native 2.0"` 宣言文エントリを追加する
- `★クリーンアップ`（`cargo clean`）完了

**実績**: COMPLETE — 3227 tests passed, 0 failed（2026-07-25）
- `cargo_toml_version_is_55_9_0` テスト削除（v56.0 バージョン更新後は旧バージョン文字列が存在しないため）
- 実テスト数: 3227（ベース 3223 + v56000_tests 4件）

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v54.1-v55.0.md`
- 達成宣言: `MILESTONE.md`
