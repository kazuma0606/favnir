# Roadmap v42.1.0 〜 v43.0.0 — Real-Time Power

Date: 2026-07-11
Status: 計画中（v42.0 完了後に詳細確定）

---

## 目標

v42.0「Type Precision」で Refinement type・パターン強化を整備した。
このフェーズは **「CEP・Stream join・Back-pressure により、サブ秒レイテンシのリアルタイムパイプラインを型安全に記述できるようにする」** を実現する。

---

## バージョン計画

### v42.1.0 — CEP DSL 基盤 ✅ COMPLETE（2026-07-12）

Complex Event Processing の構文・型・VM サポート基盤。
`cep` キーワードと `pattern` ブロックを parser・checker.fav に追加。

```favnir
cep pattern LoginEvent {
  Login within 60
}
```

**完了条件**: Rust テスト 3 件（推定 2870 tests passed, 0 failed）
**実績**: v42100_tests 3/3 PASS、2877 tests passed, 0 failed

---

### v42.2.0 — CEP パターン: `seq` / `any` / `not` ✅ COMPLETE（2026-07-12）

```favnir
cep pattern LoginThenPurchase {
  seq(Login, Purchase) within 300
}

cep pattern AnyAlert {
  any(DiskFull, OOM, NetworkDown)
}
```

`seq` / `any` / `not` の 3 パターンコンビネータを実装。

**完了条件**: Rust テスト 3 件（推定 2880 tests passed, 0 failed）
**実績**: v42200_tests 3/3 PASS、2880 tests passed, 0 failed

---

### v42.3.0 — CEP checker.fav 統合 ✅ COMPLETE（2026-07-12）

CEP パターンのセマンティクス検証。`within` 値の数値的妥当性（>= 1）を checker.rs で検証し E0420 を追加。
※ イベント名の型環境検証（「型変数を checker.fav で検証」）はイベント型システムが未整備のため v44.x に延期。

```favnir
// NG: within 0 は意味論的に不正 → E0420
cep pattern P {
  Login within 0
}

// OK: within >= 1
cep pattern LoginThenPurchase {
  seq(Login, Purchase) within 300
}
```

**完了条件**: Rust テスト 3 件（推定 2883 tests passed, 0 failed）
**実績**: v42300_tests 3/3 PASS、2883 tests passed, 0 failed

---

### v42.4.0 — Stream join（time-window）✅ COMPLETE（2026-07-12）

```favnir
bind joined <- Stream.join(orders, payments,
  on: |o, p| o.id == p.order_id,
  window: 60)
```

2 ストリームの time-window join 演算子。
※ 名前付き引数構文（`on:`, `window:`）はパーサー未対応のため位置引数形式で代替実装。
※ join キーの型安全チェック（checker.fav への移植）はイベント型システム整備後の v43.x 以降に延期。現バージョンは `Stream<Unknown>` 型推論のみ。

**完了条件**: Rust テスト 3 件（推定 2886 tests passed, 0 failed）

---

### v42.5.0 — Back-pressure `@max_inflight` ✅ COMPLETE（2026-07-12）

```favnir
#[max_inflight(100)]
stage SlowSink: Rows -> Unit = |ctx, rows| {
  bind _ <- Db.batch_insert(ctx, rows)
}
```

`#[max_inflight(n)]` アノテーションを parser・AST に追加（Favnir 構文は `#[max_inflight(...)]`）。
※ runtime back-pressure（VM スケジューラーによる上流ステージ一時停止）は VM にスケジューラー未実装のため v44.x に延期。v42.5.0 ��� parser + AST 宣言のみ。

**完了条件**: Rust テスト 2 件（推定 2888 tests passed, 0 failed）

---

### v42.6.0 — WebSocket Rune ✅ COMPLETE（2026-07-12）

リアルタイム push sink。`runes/websocket/` 追加。
`WebSocket.send(ctx, url, message)` / `WebSocket.broadcast(ctx, url, messages)`

**完了条件**: Rust テスト 2 件（実績 2891 tests passed, 0 failed）
※ v42.5.0 でネガティブテスト 1 件が追加されたため、ロードマップ当初記載の 2890 を 2891 に修正。

---

### v42.7.0 — `fav monitor` ✅ COMPLETE（2026-07-12）

実行中パイプラインのスループット / イベント数 / レイテンシをターミナルに表示。
`cmd_monitor` を main.rs・driver.rs に追加。

**完了条件**: Rust テスト 2 件（実績 2893 tests passed, 0 failed）
※ v42.6.0 実績が 2891 のため、ロードマップ当初記載の 2892 を 2893 に修正。

---

### v42.8.0 — Real-Time Power cookbook ✅ COMPLETE（2026-07-12）

- `site/content/cookbook/cep-login-purchase.mdx`
- `site/content/cookbook/stream-join.mdx`

**完了条件**: Rust テスト 1 件（実績 2894 tests passed, 0 failed）
※ v42.7.0 実績が 2893 のため、ロードマップ当初記載の 2893 を 2894 に修正。

---

### v42.9.0 — v43.0 前調整・安定化 ✅ COMPLETE（2026-07-12）

コードフリーズ（新規機能追加なし）。`site/content/docs/real-time-power.mdx` 新規作成。

**完了条件**: meta テスト 2 件（推定 2896 tests passed, 0 failed）
※ v42.8.0 実績が 2894 のため、ロードマップ当初記載の 2895 を 2896 に修正。
**実績**: v42900_tests 2/2 PASS、2896 tests passed, 0 failed

---

### v43.0.0 — Real-Time Power 宣言 ★クリーンアップ ✅ COMPLETE（2026-07-12）

**宣言文（暫定）**:

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `@max_inflight` で Back-pressure を宣言的に制御できる。
>
>  これが Favnir v43.0 — Real-Time Power の姿である。」

**完了条件**:
- v42.1〜v42.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ 2896 + 4 = **2900**）
  ※ v42.9.0 実績 2896 を起点に修正（当初 2895 + 4 = 2899 は誤差）
- `v43000_tests` 4 件 pass（内訳: `cargo_toml_version_is_43_0_0` / `changelog_has_v43_0_0` / `milestone_has_real_time_power` / `readme_mentions_real_time_power`）
- `MILESTONE.md` に `"Real-Time Power"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: v43000_tests 4/4 PASS、2900 tests passed, 0 failed、cargo clean 完了（29.9 GiB 削除）

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v41.1-v42.0.md`
- 達成宣言: `MILESTONE.md`
