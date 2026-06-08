# Favnir v12.5.0 仕様書

Date: 2026-06-08
Theme: `fav run --verbose` + `fav check --json` / `--show-types`

---

## 概要

v12.4.0 までで言語セマンティクスの Critical 修正（bind / seq）が完了した。
v12.5.0 はデバッグ可視性の強化フェーズ。

「ECS に上げてみて初めてわかる」バグを減らすことと、
Claude Code / Codex 等の AI ツールが `fav check` の出力を機械的に読んで自己修正ループを回せることの
両方を目的とする。

---

## 機能 1: `fav run --verbose`

### 目的

実行中のステージ・bind 操作の流れを stderr に出力し、
CloudWatch や CI ログから失敗箇所を特定できるようにする。

現状、ECS Fargate 上でのデバッグは 1 サイクル 6〜8 分かかる。
`--verbose` があれば CloudWatch に詳細が残り、原因特定が大幅に短縮できる。

### 出力形式

```
[TRACE] stage LoadAndInsert: enter(path="/app/sample.csv")
[TRACE]   bind rows_json <- load_csv_rows_json(...) → Ok("..."[312 chars])
[TRACE]   bind _ <- Postgres.execute_raw("CREATE TABLE...") → Err("db error: SSL required")
[TRACE] stage LoadAndInsert: exit Err("db error: SSL required")
[TRACE] seq Pipeline: stopped at stage 1/3 (LoadAndInsert)
```

### 仕様

- `fav run --verbose <file>` ですべてのトレースを stderr に出力
- `--trace` でフル出力（値の長さ制限なし）
- デフォルト（`--verbose`）は値を最大 200 文字でトランケート、末尾に `[N chars]` を付与
- `fav.toml` の `[run] verbose = true` でも有効化できる
- stdout（`IO.println` 等の出力）には影響しない

### トレース出力の対象

| イベント | 出力例 |
|---|---|
| stage 開始 | `[TRACE] stage StageName: enter(arg=...)` |
| bind / chain 結果 | `[TRACE]   bind x <- expr → Ok("value")` |
| stage 終了 | `[TRACE] stage StageName: exit Ok(...)` / `exit Err(...)` |
| seq 停止 | `[TRACE] seq PipelineName: stopped at stage N/M (StageName)` |

---

## 機能 2: `fav check --json`

### 目的

AI ツールが `fav check` の出力を JSON パースして自己修正ループを回せるようにする。

現状の `fav check` はテキスト出力のみで、AI が解析して次のアクションを決めることができない。

### 出力形式

```json
{
  "errors": [
    {
      "code": "E0018",
      "message": "variable 'x' is already bound in this scope",
      "file": "pipeline.fav",
      "line": 12,
      "col": 3,
      "suggestion": "rename to 'x2' or use 'bind _'"
    }
  ],
  "warnings": [
    {
      "code": "W006",
      "message": "discarding Result value with bind _",
      "file": "pipeline.fav",
      "line": 10,
      "col": 3,
      "suggestion": "use 'chain _' to propagate errors"
    }
  ],
  "ok": false
}
```

### 仕様

- `fav check --json <file>` で JSON を stdout に出力
- エラーなしの場合: `{ "errors": [], "warnings": [], "ok": true }`
- exit code は既存と同じ（エラーあり → 1、なし → 0）
- `--json` は `--show-types` と組み合わせ可能

---

## 機能 3: `fav check --show-types`

### 目的

各 `bind` / `chain` の右辺型を表示し、
「自分が書いたコードの型が何か」を確認できるようにする。

AI が `Result` を捨てているかどうかを一目で確認できる。

### 出力形式

```
pipeline.fav:8   bind rows_json : String
pipeline.fav:10  bind _         : Result<Unit, String>  ← W006
pipeline.fav:12  bind _         : Result<Unit, String>  ← W006
pipeline.fav:15  bind result    : List<Map<String, String>>
```

### 仕様

- `fav check --show-types <file>` で各 bind/chain の推論型を表示
- W006 対象の bind には `← W006` を末尾に付与
- `--json` と組み合わせた場合は JSON の各エントリに `"inferred_type"` フィールドを追加:

```json
{
  "bindings": [
    { "file": "pipeline.fav", "line": 8,  "name": "rows_json", "type": "String" },
    { "file": "pipeline.fav", "line": 10, "name": "_",         "type": "Result<Unit, String>", "warning": "W006" }
  ]
}
```

---

## 実装方針

### `--verbose` の実装箇所

- `fav/src/driver.rs` または `fav/src/backend/vm.rs`
- VM の stage dispatch / bind opcode ハンドラにフック
- `--verbose` フラグを `RunConfig` に追加
- stderr への出力は `eprintln!` で直接行う（ロギングライブラリ不使用）

### `--json` / `--show-types` の実装箇所

- `fav/src/driver.rs` の `cmd_check` 関数
- 既存のテキスト出力を `CheckOutput` 構造体に集約してから出力形式を切り替える
- `--json` フラグ → `serde_json::to_string` で出力
- `--show-types` フラグ → checker の推論結果を走査して bind ごとの型を収集

### checker.fav への影響

- `--show-types` は Rust checker（`checker.rs`）または checker.fav どちらでも実装可能
- checker.fav の出力 JSON に `bindings` フィールドを追加する方が self-hosted 設計に合う
- ただし checker.fav の変更コストが高い場合は Rust 側で後処理として実装

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `verbose_logs_stage_enter` | `--verbose` で `[TRACE] stage X: enter` が stderr に出る |
| `verbose_logs_bind_result` | bind の結果が `→ Ok(...)` / `→ Err(...)` 形式で出る |
| `verbose_logs_seq_stopped` | seq fail-fast 時に `stopped at stage N/M` が出る |
| `verbose_truncates_long_values` | 200 文字超の値がトランケートされる |
| `check_json_output_format` | `--json` で正しい JSON 構造が出る |
| `check_json_ok_true_on_success` | エラーなしで `"ok": true` |
| `check_json_includes_suggestion` | エラーの `suggestion` フィールドが存在する |
| `check_show_types_bind` | `--show-types` で bind ごとの型が表示される |
| `check_show_types_w006_marked` | W006 対象の bind に `← W006` が付く |
| `version_is_12_5_0` | `CARGO_PKG_VERSION == "12.5.0"` |

---

## 完了条件

- [ ] `fav run --verbose` で stage / bind のトレースが stderr に出る
- [ ] `fav check --json` で JSON 形式のエラー出力が得られる
- [ ] `fav check --show-types` で bind ごとの推論型が表示される
- [ ] 全テストケース通過
- [ ] `cargo test` 全通過

---

## 非目標

- ロギングライブラリの導入（`tracing` / `log` crate 等）
- `--verbose` の出力を JSON にする（テキスト形式で十分）
- `fav run --json`（実行結果の JSON 出力）— 本バージョンのスコープ外
