# v22.6.0 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1872 passed / 0 failed（v226000_tests 8/8 PASS）

---

## タスク一覧

### T1: `fav/src/ast.rs` — SLA struct 追加 + `TrfDef` フィールド追加

- [x] **事前確認**: `grep -n "TriggerAnnotation\|pub struct TrfDef\|pub checkpoint\|pub span: Span" fav/src/ast.rs | head -15` で挿入位置を確認
- [x] `TriggerAnnotation` ブロックの直後（`// ── FnDef` コメントの前）に `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` struct を追加（plan.md T1-1 のコードに従う）
- [x] `TrfDef` の `pub checkpoint: bool,` の直後（`pub span: Span,` の前）に `timeout` / `retry_ann` / `circuit_breaker` フィールドを追加（plan.md T1-2 のコードに従う）
- [x] `cargo check --bin fav` で `TrfDef` struct literal の壊れた箇所をリストアップ（T2 と合わせて修正）

---

### T2: `fav/src/frontend/parser.rs` — 3 アノテーションパーサー追加 + `parse_item` 統合

- [x] **事前確認**: `grep -n "parse_trigger_annotation\|trigger_ann\|checkpoint_ann" fav/src/frontend/parser.rs | head -15` で `parse_trigger_annotation` の後ろと `trigger_ann` の行を確認
- [x] `parse_trigger_annotation` の直後に `parse_timeout_annotation` メソッドを追加（plan.md T2-1 のコードに従う）
  - `# [ timeout` の 3 トークン lookahead
  - `[` は `self.expect(&TokenKind::LBracket)?`（`advance()` ではなく `expect()` を使うこと）
  - `seconds` は `Int` / `Float` 両対応（`n as f64`）
- [x] `parse_timeout_annotation` の直後に `parse_retry_annotation` メソッドを追加（plan.md T2-2 のコードに従う）
  - `[` は `self.expect(&TokenKind::LBracket)?`
  - `max` は `Int` のみ（`n as u32`）、`backoff` は `self.expect_str()` で取得
- [x] `parse_retry_annotation` の直後に `parse_circuit_breaker_annotation` メソッドを追加（plan.md T2-3 のコードに従う）
  - `[` は `self.expect(&TokenKind::LBracket)?`
  - `threshold` は `Int` / `Float` 両対応、`window` は `Int` のみ（`n as u64`）
- [x] `parse_item()` 内の `let trigger_ann = self.parse_trigger_annotation()?;` の直後に 3 行追加（plan.md T2-4 のコードに従う）
- [x] `TokenKind::Stage` ブランチの `td.trigger = trigger_ann;` 直後に 3 行追加（plan.md T2-5 のコードに従う）**2 箇所**（通常 stage と async stage）
- [x] `cargo check --bin fav` でエラーが出た `TrfDef { ... }` 各所に `timeout: None, retry_ann: None, circuit_breaker: None` を追加（plan.md T2-6 のコードに従う）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/middle/checker.rs` — E0401 / E0402 / E0403 追加

- [x] **事前確認**: `grep -n "fn check_trf_def\|self\.type_error\|E0314\|E0335" fav/src/middle/checker.rs | head -10` で `check_trf_def` の位置と `type_error` の引数形式を確認
- [x] `check_trf_def()` 末尾に SLA バリデーションコードを追加（plan.md T3-1 のコードに従う）
  - **`check_item` ではなく `check_trf_def` に追加すること**
  - E0401: `td.timeout.seconds <= 0.0`
  - E0402: `td.retry_ann.max == 0` または backoff 文字列が不正
  - E0403: `td.circuit_breaker.threshold` が範囲外または `window == 0`
  - `self.type_error(code, msg, &span)` を使う（`CheckError::new` は不正）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/error_catalog.rs` — E0401〜E0403 エントリ追加

- [x] **事前確認**: `grep -n "E03[0-9][0-9]\|pub const ERROR_CATALOG" fav/src/error_catalog.rs | tail -10` で現在の最大エラーコードを確認
- [x] 既存の最大エラーコード（E03xx）の直後に E0401〜E0403 エントリを追加（plan.md T4-1 のコードに従う）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/driver.rs` — `cmd_explain_sla` + `v226000_tests`

- [x] **事前確認**: `grep -n "pub fn cmd_explain_lineage\|// ── v22.5.0\|v225000_tests" fav/src/driver.rs | head -10` で挿入位置を確認

#### 5-1: `cmd_explain_sla` を追加

- [x] `cmd_explain_lineage` の直後に `cmd_explain_sla` を追加（plan.md T5-1 のコードに従う）
  - `file: Option<&str>` — `None` の場合 exit
  - セパレーター付き表形式出力
  - `worst = timeout_secs * retry.max`（timeout のみの場合は `timeout_secs`）
  - アノテーションなし stage は `—` 表示、`total_worst_secs` に加算しない
  - 最後に `Total worst-case (SLA-annotated stages only): Xs` を表示

#### 5-2: `v225000_tests::version_is_22_5_0` に `#[ignore]` を追加

- [x] 完了

#### 5-3: `v226000_tests` モジュールを追加（8 テスト）

- [x] `version_is_22_6_0`
- [x] `timeout_annotation_parsed`（`td.timeout.seconds == 30.0`）
- [x] `retry_annotation_parsed`（`r.max == 3`, `r.backoff == "exponential"`）
- [x] `circuit_breaker_annotation_parsed`（`cb.threshold == 0.5`, `cb.window == 60`）
- [x] `sla_invalid_timeout_checker_err`（E0401 — `seconds = 0`）
- [x] `sla_invalid_retry_checker_err`（E0402 — `max = 0`）
- [x] `sla_invalid_circuit_breaker_checker_err`（E0403 — `threshold = 0.0`）
- [x] `changelog_has_v22_6_0`
- [x] `cargo test v226000 --bin fav` — 8/8 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1872 件合格）を確認

---

### T6: `fav/src/main.rs` — `fav explain --sla` 対応

- [x] **事前確認**: `grep -n "\"--lineage\"\|cmd_explain_lineage\|Some(\"explain\")" fav/src/main.rs | head -10` で `Some("explain")` ブランチを確認
- [x] `Some("explain")` ブランチ内の `--lineage` チェックの直後に `--sla` チェックを追加（plan.md T6-1 のコードに従う）
  - `args.iter().skip(2).find(|a| !a.starts_with('-'))` でファイルパスを取得
  - `crate::driver::cmd_explain_sla(file)` を呼ぶ
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T7: `fav/Cargo.toml` + `CHANGELOG.md` + MDX + benchmarks

- [x] **事前確認**: `grep "\[v22.5.0\]" CHANGELOG.md` で現在の先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "22.5.0"` → `"22.6.0"` に変更
- [x] v22.6.0 エントリを `CHANGELOG.md` の先頭（v22.5.0 エントリの上）に追加（plan.md T7-2 のコードに従う）
- [x] `grep "\[v22.6.0\]" CHANGELOG.md` で追加確認
- [x] `site/content/docs/cli/sla.mdx` を新規作成
- [x] `benchmarks/v22.6.0.json` を新規作成

---

## テスト一覧（v226000_tests、8 件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_22_6_0` | Cargo.toml に `version = "22.6.0"` が含まれる | PASS |
| `timeout_annotation_parsed` | `#[timeout(seconds = 30)]` が正しくパースされ `td.timeout.seconds == 30.0` | PASS |
| `retry_annotation_parsed` | `#[retry(max = 3, backoff = "exponential")]` が正しくパースされる | PASS |
| `circuit_breaker_annotation_parsed` | `#[circuit_breaker(threshold = 0.5, window = 60)]` が正しくパースされる | PASS |
| `sla_invalid_timeout_checker_err` | `seconds = 0` で E0401 が `check_program` に報告される | PASS |
| `sla_invalid_retry_checker_err` | `max = 0` で E0402 が報告される | PASS |
| `sla_invalid_circuit_breaker_checker_err` | `threshold = 0.0` で E0403 が報告される | PASS |
| `changelog_has_v22_6_0` | CHANGELOG.md に `[v22.6.0]` が含まれる | PASS |

---

## 完了条件チェックリスト

- [x] `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` が `ast.rs` に追加される
- [x] `TrfDef` に `timeout` / `retry_ann` / `circuit_breaker` フィールドが追加される
- [x] `#[timeout(seconds = N)]` がパースされ `TrfDef.timeout` に設定される
- [x] `#[retry(max = N, backoff = "...")]` がパースされ `TrfDef.retry_ann` に設定される
- [x] `#[circuit_breaker(threshold = F, window = N)]` がパースされ `TrfDef.circuit_breaker` に設定される
- [x] E0401 / E0402 / E0403 が `check_trf_def()` 経由でコンパイル時に報告される
- [x] E0401〜E0403 が `error_catalog.rs` に登録される
- [x] `fav explain --sla [file]` が SLA 一覧と最悪実行時間（SLA 付き stage のみ合計）を出力する
- [x] `cargo test v226000 --bin fav` — 8/8 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1872 件合格）
- [x] `CHANGELOG.md` に v22.6.0 エントリ
- [x] `benchmarks/v22.6.0.json` 作成済み
- [x] `site/content/docs/cli/sla.mdx` 作成済み

---

## コードレビュー指摘と対応

（コードレビューは完了後に実施予定）

---

## 優先度

```
T1（ast.rs）           ← 最初（T2/T3 の依存元）
T2（parser.rs）        ← T1 完了後
T3（checker.rs）       ← T2 完了後
T4（error_catalog.rs） ← T3 と並行可
T5（driver.rs）        ← T3 完了後
T6（main.rs）          ← T5 完了後
T7（Cargo + doc）      ← T6 完了後
```
