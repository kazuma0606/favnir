# Favnir v12.5.0 実装計画

Date: 2026-06-08

---

## Phase A — `fav run --verbose`

### A-1: `RunConfig` にフラグ追加

`fav/src/driver.rs` の `RunConfig` struct に `verbose: bool` と `trace: bool` を追加。

```rust
pub struct RunConfig {
    pub verbose: bool,   // --verbose: 200 文字トランケート
    pub trace:   bool,   // --trace:   フル出力
    // ... 既存フィールド
}
```

`cmd_run` のフラグパースで `--verbose` / `--trace` を読む。

### A-2: VM にトレースフック追加

`fav/src/backend/vm.rs` の `execute_function` / stage dispatch に
`verbose` フラグを受け渡し、以下のタイミングで eprintln する。

```rust
// stage 開始時
if config.verbose {
    eprintln!("[TRACE] stage {}: enter(...)", stage_name);
}

// bind opcode 実行後（スタックトップを読んで表示）
if config.verbose {
    let display = truncate_value(&val, 200);
    eprintln!("[TRACE]   bind {} <- ... → {}", slot_name, display);
}

// stage 終了時
if config.verbose {
    eprintln!("[TRACE] stage {}: exit {}", stage_name, result_display);
}
```

### A-3: `seq` fail-fast 時のトレース

`SeqStageCheck` opcode のエラー escape 時に以下を出力。

```
[TRACE] seq PipelineName: stopped at stage N/M (StageName)
```

### A-4: `fav.toml` の `[run] verbose = true` 対応

`load_run_config` で `[run]` セクションの `verbose` キーを読んで `RunConfig` に反映。

---

## Phase B — `fav check --json`

### B-1: `CheckOutput` 構造体を定義

`fav/src/driver.rs` に checker の出力を保持する構造体を追加。

```rust
#[derive(serde::Serialize)]
struct CheckError {
    code:       String,
    message:    String,
    file:       String,
    line:       u32,
    col:        u32,
    suggestion: String,
}

#[derive(serde::Serialize)]
struct CheckOutput {
    errors:   Vec<CheckError>,
    warnings: Vec<CheckError>,
    ok:       bool,
}
```

### B-2: `cmd_check` の出力分岐

`cmd_check` に `--json` フラグを追加。

```rust
if args.json {
    let output = CheckOutput { ... };
    println!("{}", serde_json::to_string_pretty(&output)?);
} else {
    // 既存のテキスト出力
}
```

### B-3: checker.fav の出力から JSON フィールドを収集

checker.fav が出力するエラー文字列から `code` / `message` / `line` / `col` を抽出。
正規表現または構造化出力（checker.fav 側に `--format json` を追加）で対応。

checker.fav の変更コストが高い場合は、テキスト出力を Rust 側でパースして
`CheckOutput` を構築する。

---

## Phase C — `fav check --show-types`

### C-1: `--show-types` フラグ追加

`cmd_check` のフラグパースに `--show-types` を追加。

### C-2: bind ごとの型収集

checker の推論結果から各 `bind` / `chain` の右辺型を収集する。

選択肢:
- (a) checker.fav に `--show-types` モードを追加し、推論型を出力に含める
- (b) Rust の `checker.rs`（legacy）で bind ごとの型を収集し出力

実装コストの低い (b) を先に行い、self-hosted 移行は後続バージョンで対応。

### C-3: テキスト出力フォーマット

```
pipeline.fav:8   bind rows_json : String
pipeline.fav:10  bind _         : Result<Unit, String>  ← W006
```

列幅を揃えて出力（ファイル名:行番号 を固定幅にする）。

### C-4: `--json --show-types` 組み合わせ

`--json` と同時指定された場合、JSON に `"bindings"` フィールドを追加。

```json
{
  "bindings": [
    { "file": "pipeline.fav", "line": 8, "name": "rows_json", "type": "String" },
    { "file": "pipeline.fav", "line": 10, "name": "_", "type": "Result<Unit, String>", "warning": "W006" }
  ]
}
```

---

## Phase D — テスト追加

`fav/src/driver.rs` の `v12500_tests` モジュールに以下を追加。

```rust
#[cfg(test)]
mod v12500_tests {
    // A系: verbose
    fn verbose_logs_stage_enter()       { ... }
    fn verbose_logs_bind_result()       { ... }
    fn verbose_logs_seq_stopped()       { ... }
    fn verbose_truncates_long_values()  { ... }

    // B系: --json
    fn check_json_output_format()       { ... }
    fn check_json_ok_true_on_success()  { ... }
    fn check_json_includes_suggestion() { ... }

    // C系: --show-types
    fn check_show_types_bind()          { ... }
    fn check_show_types_w006_marked()   { ... }

    // バージョン確認
    fn version_is_12_5_0()              { ... }
}
```

---

## Phase E — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.5.0"`
- `cargo test` 全通過確認
- `git commit -m "feat: v12.5.0 — fav run --verbose + fav check --json / --show-types"`
- `git push`

---

## 実装上の注意

1. **verbose と stdout の分離**: `IO.println` 等のユーザー出力は stdout、
   トレースはすべて stderr。bootstrap / CI スクリプトが stdout をパースする場合に干渉しない。

2. **`--trace` と `--verbose` の違い**: `--trace` は `--verbose` のスーパーセット。
   `--trace` なら値の長さ制限なし。`--verbose` は 200 文字トランケート。

3. **checker.fav の変更スコープ**: `--show-types` を checker.fav 側に実装する場合、
   checker.fav は JSON 出力モードを持つことになる。
   これは v12.7.0（`fav doc --builtins --format json`）との設計整合も考慮する。

4. **serde_json は既存依存**: `Cargo.toml` に `serde_json = "1"` は既に含まれている。
   追加の依存は不要。
