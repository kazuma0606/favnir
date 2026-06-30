# v25.3.0 タスクリスト — redis Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.3.0"` に bump + `redis` crate 追加（`cargo build` で features 確認） | [x] |
| T1 | `fav/src/ast.rs` 更新（`Effect::Redis` 追加） | [x] |
| T2 | `fav/src/middle/checker.rs` 更新（`ns_to_effect` / `require_redis_effect` / redis builtin fns） | [x] |
| T3 | `fav/src/emit_python.rs` / `fav/src/lineage.rs` / `fav/src/fmt.rs` / `fav/src/middle/reachability.rs` / `fav/src/middle/ast_lower_checker.rs` / `fav/src/lint.rs` 更新（`Effect::Redis` 対応・6 ファイル） | [x] |
| T4 | `fav/src/error_catalog.rs` 更新（E0320 追加） | [x] |
| T5 | `runes/redis/redis.fav` 全面更新（connect / get / set / del / incr / lpush / rpop / publish / subscribe_once） | [x] |
| T6 | `fav/src/backend/vm.rs` 更新（`Redis.*_raw` 9 件追加、subscribe_once はタイムアウト 30 秒付き） | [x] |
| T7 | `examples/redis_rate_limiter.fav` 新規作成（`import rune "redis"` 使用、`String.from_int` で型修正済み） | [x] |
| T8 | `CHANGELOG.md` 更新（`[v25.3.0]` エントリ追加） | [x] |
| T9 | `site/content/docs/runes/redis.mdx` 新規作成（全 API 記載） | [x] |
| T10 | `fav/src/driver.rs` 更新（`v253000_tests` 7 件追加） | [x] |
| T11 | `benchmarks/v25.3.0.json` 新規作成（実測値 1993 件） | [x] |
| T12 | `cargo test v253000` — 7 件 PASS 確認 | [x] |
| T13 | `cargo test` 総テスト数 ≥ 1993 件 確認（実績: 1993 件） | [x] |
| T14 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `Redis.connect` が `runes/redis/redis.fav` に存在する
- [x] `Redis.get / set / del / incr` が `runes/redis/redis.fav` に存在する
- [x] `Redis.lpush / rpop / publish / subscribe_once` が `runes/redis/redis.fav` に存在する
- [x] `Redis.*_raw` 9 件すべてが `fav/src/backend/vm.rs` に存在する
- [x] `Effect::Redis` が `fav/src/ast.rs` に存在する（`cargo build` で exhaustive match 確認済み）
- [x] E0320 が `fav/src/error_catalog.rs` に存在する（E0316〜E0319 は既存のため E0320 を採用）
- [x] `examples/redis_rate_limiter.fav` が存在し `import rune "redis"` / `incr` / `lpush` を含む
- [x] `CHANGELOG.md` に `v25.3.0` が存在する
- [x] `site/content/docs/runes/redis.mdx` が存在し全 API を記載している
- [x] `v253000_tests` 7 件すべて PASS（Effect::Redis テスト含む）
- [x] 総テスト数 ≥ 1993 件（実績: 1993 件）

---

## コードレビュー指摘（spec-reviewer — 実装前に対応済み）

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| HIGH | `as_pubsub()` は redis crate v0.25 に存在しない | `PubSubCommands::subscribe` + `ControlFlow::Break` パターンに変更（`into_pubsub()` も不存在のため）|
| HIGH | `subscribe_once` のタイムアウト仕様が spec.md に未記載 | `set_read_timeout(30s)` 付き。タイムアウト時は `Result.err("timeout: ...")` |
| HIGH | `Effect::Redis` 追加で更新必要な 3 ファイルが未列挙（reachability.rs / ast_lower_checker.rs / lint.rs） | 実装時に 8 ファイル更新（driver.rs / parser.rs も追加で必要だった） |
| HIGH | example の `Redis.set(conn, key, count, 60)` が型エラー | `String.from_int(count)` に修正 |
| MED | ロードマップ逸脱が未記録 | tasks.md メモ欄に意図的逸脱として明記 |
| MED | emit_python.rs 非網羅的状態 | `_ => "Effect"` キャッチオールがあるため `Redis` 明示アームを追加 |
| MED | features 未検証 | `cargo build` で `tcp_nodelay` のみで subscribe が動作することを確認済み |
| MED | redis.mdx 漏れ | T9 として追加・完了 |
| MED | Effect::Redis 存在確認テスト欠落 | 7 件目 `effect_redis_exists_in_ast` テストを追加 |
| LOW | RedisConn 定義場所 | redis.fav に直接定義（コメントで将来方針を明記） |
| LOW | benchmark の Step 順序 | T11（benchmark）を T10（テスト）の後に実施 |
| LOW | 依存バージョン矛盾 | spec.md を v25.1.0 に修正（ロードマップと整合） |

## 実装時に発見した追加修正

| 問題 | 修正 |
|---|---|
| E0316 は AzureDb に既使用 | Redis には E0320 を採用（E0316〜E0319 は Gcp/Stream/Azure 系で使用済み） |
| `driver.rs` にも Effect match があった | `format_effects` と `effect_json_name` の 2 箇所に `Redis` アームを追加 |
| `parser.rs` にも Effect パース処理があった | `"Redis" => Effect::Redis` アームを追加 |
| `PubSubCommands::subscribe` の unwrap_or 型推論 | `let _: () = conn.subscribe(...)` で型注釈を追加 |

---

## メモ

- `!Cache` エフェクト（v7.3.0）は**インメモリキャッシュ**用 → `!Redis`（外部サービス）とは完全独立
- **ロードマップ v25.3 との意図的逸脱**: `Redis.subscribe(conn, channel, fn)` は VM のクロージャ呼び出し制約により `subscribe_once`（1 件受信）として実装。将来バージョンで `subscribe` を別関数として追加予定
- `redis` crate v0.25 の Pub/Sub API: `PubSubCommands::subscribe` トレイト + `ControlFlow::Break`
- `connect_raw` は PING 確認のみ、実接続は各 primitive 内で都度確立（PgConn パターンと同様）
- `Effect::Redis` 追加で更新が必要なファイル（実際): ast.rs / checker.rs / reachability.rs / ast_lower_checker.rs / emit_python.rs / lineage.rs / lint.rs / fmt.rs / driver.rs / parser.rs（計 10 ファイル）
- spec.md / plan.md のエラーコード E0316 は E0320 に修正済み
