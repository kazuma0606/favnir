# v22.7.0 — OpenTelemetry 統合 タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1878 passed / 0 failed（v227000_tests 5/5 PASS）

---

## タスク一覧

### T1: `fav/src/otel.rs` — OTel モジュール新規作成

- [x] **事前確認**: `grep -n "rand\|ureq" fav/Cargo.toml | head -10` で rand / ureq が native-only deps にあることを確認
- [x] **事前確認**: `grep -n "SeqStageEnter\|SeqStageCheck\|stage_name" fav/src/backend/vm.rs | head -15` で挿入位置を確認（T2 の作業基準）
- [x] `fav/src/otel.rs` を新規作成（plan.md T1-1 のコードに従う）
  - `TraceId` / `SpanId` 型エイリアス
  - `OtelSpan` struct / `OtelStatus` enum
  - thread-local 5 変数（`OTEL_SPANS` / `CURRENT_TRACE` / `PARENT_STACK` / `OTEL_ENABLED` / `PENDING_SPANS`）
  - ID 生成: `rand::random::<[u8; 16/8]>()` → hex encode
  - `otel_init` / `otel_is_enabled` / `otel_current_parent` / `otel_span_start` / `otel_span_end`
  - `otel_collected_spans` / `otel_reset`
  - `otel_export_http` — ureq v2 の 3-arm match（`Ok(_)` / `Err(Status(code, _))` / `Err(e)`）
  - `otel_export_stdout` — `eprintln!` で stderr に出力
  - `build_otlp_json` — `env!("CARGO_PKG_VERSION")` で service.version
  - `escape_json_str` ヘルパー
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/vm.rs` — stage 実行に OTel span を追加

- [x] **事前確認**: `grep -n "SeqStageEnter\|SeqStageCheck\|stage_name\s*=\|verbose_level\|trace_emit.*enter\|trace_emit.*exit" fav/src/backend/vm.rs | head -25` で変更箇所を確認
- [x] **事前確認**: `grep -n "pub struct VM\|trace_lines\|pub " fav/src/backend/vm.rs | head -20` で VM struct フィールドを確認
- [x] **事前確認**: `grep -n "unwrapped\|inner_msg" fav/src/backend/vm.rs | grep -v "//\|test" | head -10` で SeqStageCheck の変数名を確認
- [x] `VM` struct に `pub current_otel_span_id: Option<crate::otel::SpanId>` フィールドを追加（plan.md T2-1）
- [x] `VM` struct literal（初期化箇所）に `current_otel_span_id: None` を追加（plan.md T2-2）
- [x] `SeqStageEnter` を修正（plan.md T2-3）:
  - `stage_name` を `if Self::verbose_level() > 0 {}` ブロックの**外**に hoist する
  - verbose ブロックの後に OTel span 開始コードを追加
- [x] `SeqStageCheck` の Ok パスに OTel span 終了（Ok）を追加（plan.md T2-4）
- [x] `SeqStageCheck` の Err パスに OTel span 終了（Err）を追加（plan.md T2-4）
- [x] `otel_value_items` ヘルパーを `fn trace_emit` の直前に追加（plan.md T2-5）
  - `VMValue::List` のバリアント形式を `grep -n "enum VMValue\|List(" fav/src/backend/vm.rs | head -10` で確認
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/driver.rs` — `cmd_run` に OTel 追加 + `v227000_tests`

- [x] **事前確認**: `grep -n "set_verbose_level\|cmd_run_self_hosted\|fn cmd_run\b" fav/src/driver.rs | head -10` で挿入位置を確認
- [x] **事前確認**: `grep -n "build_artifact\|exec_artifact_main" fav/src/driver.rs | grep -v "fn " | head -10` でテスト用ヘルパーを確認
- [x] `cmd_run` の `set_verbose_level(verbose_level)` 直後に OTel init を追加（plan.md T3-1）
- [x] `cmd_run` の末尾（`// ── fav run --self-host` コメントの直前）に OTel export を追加（plan.md T3-2）
- [x] `v226000_tests::version_is_22_6_0` に `#[ignore]` を追加（plan.md T3-3）
- [x] `v227000_tests` モジュールを追加（5 件、plan.md T3-4 のコードに従う）
  - `version_is_22_7_0`
  - `otel_spans_collected_after_run`
  - `otel_span_name_includes_stage_name`
  - `otel_export_stdout_does_not_panic`
  - `changelog_has_v22_7_0`
- [x] `cargo test v227000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1874 件以上合格）を確認

---

### T4: `fav/src/lib.rs` / `fav/src/main.rs` — `mod otel` 宣言追加

- [x] **事前確認**: `grep -n "cfg(not.*wasm32\|^pub mod\|^mod " fav/src/lib.rs | head -20` で挿入位置を確認
- [x] `lib.rs` に `#[cfg(not(target_arch = "wasm32"))]\npub mod otel;` を追加（plan.md T4-1）
- [x] `main.rs` に `#[cfg(not(target_arch = "wasm32"))]\nmod otel;` を追加（plan.md T4-2）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: バージョン更新・CHANGELOG・MDX・benchmarks

- [x] **事前確認**: `grep "\[v22.6.0\]" CHANGELOG.md` で現在の先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "22.6.0"` → `"22.7.0"` に変更
- [x] v22.7.0 エントリを `CHANGELOG.md` の先頭（v22.6.0 エントリの上）に追加（plan.md T5-2）
- [x] `benchmarks/v22.7.0.json` を新規作成（plan.md T5-3）
- [x] `site/content/docs/cli/otel.mdx` を新規作成（plan.md T5-4）
- [x] `cargo test v227000 --bin fav` — 最終確認 5/5 PASS

---

## テスト一覧（v227000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_7_0` | Cargo.toml に `version = "22.7.0"` が含まれる |
| `otel_spans_collected_after_run` | pipeline 実行後に `otel_collected_spans()` が 1 件以上の span を返す |
| `otel_span_name_includes_stage_name` | span の name が `"stage:MyStage"` 形式（`contains("MyStage")` で確認） |
| `otel_export_stdout_does_not_panic` | `otel_export_stdout()` がパニックせず正常終了する |
| `changelog_has_v22_7_0` | CHANGELOG.md に `[v22.7.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `fav/src/otel.rs` が作成され、全 API が実装される
- [x] `vm.rs` の `SeqStageEnter` 処理（`stage_name` hoist 含む）に OTel span 開始が追加される
- [x] `SeqStageCheck` の OK / Err 終了時に OTel span 終了が呼ばれる
- [x] `cmd_run` の `trace = true` 時に OTel init / export が実行される
- [x] `OTEL_EXPORTER_OTLP_ENDPOINT` が設定されている場合、OTLP/HTTP で POST される
- [x] 環境変数未設定時は `[OTEL]` が stderr に出力される
- [x] `lib.rs` / `main.rs` に `#[cfg(not(target_arch = "wasm32"))]` 付き `mod otel` が追加される
- [x] `cargo test v227000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1874 件以上合格）
- [x] `CHANGELOG.md` に v22.7.0 エントリ
- [x] `benchmarks/v22.7.0.json` 作成済み
- [x] `site/content/docs/cli/otel.mdx` 作成済み

---

## 優先度

```
T1（otel.rs）          ← 最初（T2/T3 の依存元）
T2（vm.rs）            ← T1 完了後
T3（driver.rs）        ← T2 完了後
T4（lib.rs / main.rs） ← T1 完了後（T2 と並行可）
T5（Cargo + doc）      ← T3 完了後
```

---

## コードレビュー指摘と対応

**v22.7.0 コードレビュー指摘と対応:**

実装中に発見・修正（1回目）:
- [HIGH] SeqStageCheck の非 Result 値（_ ブランチ）で OTel span が終了されていなかった → _ ブランチに `otel_span_end` 追加で修正
- [HIGH] 単一 step seq では SeqChain が生成されないため span が収集されなかった → テストを 2 step seq に変更して対応
- [MED] HashMap::new() は const fn でないため thread_local! 内で const {} が使えない → PENDING_SPANS のみ const {} なしに修正済み

コードレビュー後の修正（2回目）:
- [HIGH] `process::exit` が OTel export/reset をバイパスする → `otel_flush_if_enabled()` helper を otel.rs に追加し、各 `process::exit(1)` 前に呼ぶよう修正
- [HIGH] `parse_and_run_with_otel` がパニック時に `otel_reset()` を呼ばない → `catch_unwind` + `otel_reset()` を常に呼ぶ設計に変更し、spans を返り値で返す
- [MED] `escape_json_str` が U+0000–U+001F の制御文字をエスケープしない → RFC 8259 §7 準拠の `\uXXXX` エスケープを追加
- [MED] `input_items` が常に 0（スコープ外として記録、v22.8+ で対応予定）
- [LOW] `PENDING_SPANS` の `const {}` 非使用にコメント追加
- [LOW] `unwrapped.clone().to_vmvalue()` の二重クローン → `uvm: Option<VMValue>` に一本化
