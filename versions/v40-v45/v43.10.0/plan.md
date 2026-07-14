# v43.10.0 実装計画 — `fav check --explain`

## 前提

- ベース: v43.9.0 COMPLETE（2927 tests）
- `cmd_check` シグネチャ: 11 パラメータ（`show_inference: bool` まで）
- `get_explain_text(code: &str) -> Option<&'static str>` が driver.rs に実装済み（E0001〜E0021）
- `run_checker_fav` の戻り型: `Result<(), Vec<String>>`（`Vec<TypeError>` ではない）
- `msgs_to_type_errors(msgs: Vec<String>) -> Vec<TypeError>` が checker_fav_runner.rs に実装済み
- `TypeError.code: &'static str`（checker.rs）
- `--explain` は `bundle` サブコマンドにのみ存在し、`check` には未追加
- **モジュール名注意**: `v43100_tests` は v43.1.0 のモジュールとして既存 → **`v431000_tests`** を使用する

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_check` に `explain: bool` 追加 / エラー出力ループに explain 出力 / `collect_explain_output` 追加 / `v431000_tests` 追加 / `v430900_tests::cargo_toml_version_is_43_9_0` スタブ化 |
| `fav/src/main.rs` | `--explain` フラグ追加 / `cmd_check` 呼び出し更新 |
| `fav/Cargo.toml` | version `43.9.0` → `43.10.0` |
| `CHANGELOG.md` | v43.10.0 エントリ追加 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.10.0 エントリを「静的解説ベース MVP」に修正 |

---

## 実装ステップ

### Step 1 — driver.rs: `collect_explain_output` 追加

`collect_inference_annotations` の直後（`cmd_check` の前）に追加する。

```rust
/// v43.10.0: --explain 用ヘルパー。正常コードでは空 Vec を返す（テスト用）。
/// run_checker_fav は Result<(), Vec<String>> を返すため、
/// msgs_to_type_errors で Vec<TypeError> に変換してから e.code にアクセスする。
pub fn collect_explain_output(src: &str, filename: &str) -> Vec<String> {
    use crate::checker_fav_runner;
    use crate::middle::ast_lower_checker;
    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let lower = ast_lower_checker::lower_program(&program);
    match checker_fav_runner::run_checker_fav(lower) {
        Ok(_) => vec![],
        Err(msgs) => {
            let errors = checker_fav_runner::msgs_to_type_errors(msgs);
            let mut out = Vec::new();
            for e in &errors {
                if let Some(text) = get_explain_text(e.code) {
                    out.push(format!("  Explain: {}", text));
                }
            }
            out
        }
    }
}
```

### Step 2 — driver.rs: `cmd_check` シグネチャ更新

末尾に `explain: bool` を追加（12 番目のパラメータ）。

```rust
pub fn cmd_check(
    file: Option<&str>,
    // ... 既存 10 パラメータ ...
    show_inference: bool,
    explain: bool,           // v43.10.0 新規追加
) {
```

### Step 3 — driver.rs: エラー出力ループに explain 出力

型エラーを出力するループ内（`println!("{}: {}", path, e.message)` の直後）に追加：

```rust
if explain && !json {
    if let Some(text) = get_explain_text(e.code) {
        println!("  Explain: {}", text);
    }
}
```

`--json` との同時指定では `explain` を無効化する（`!json` 条件による）。
プロジェクトモード（`file = None`）の分岐では追加しない（単一ファイルのみ対応）。

### Step 4 — driver.rs: `v431000_tests` 追加・スタブ化

`v430900_tests` モジュールの直前に `v431000_tests` を挿入。
`v430900_tests::cargo_toml_version_is_43_9_0` をスタブ化:
`// Stubbed: version bumped to 43.10.0 in v43.10.0.`

### Step 5 — main.rs: `--explain` フラグ追加

```rust
let mut explain = false;
// ...（--show-inference の直後）
"--explain" => { explain = true; i += 1; }
// cmd_check(..., show_inference, explain)
```

### Step 6 — Cargo.toml: version 更新

`43.9.0` → `43.10.0`

### Step 7 — ロードマップ更新

`roadmap-v43.1-v44.0.md` の v43.10.0 エントリを以下に修正：
「v39 の Llm Rune を活用」→「静的解説テキスト（`get_explain_text`）ベースの MVP 実装。LLM 統合は将来バージョン。」
完了条件に「実績 2929 / ✅ COMPLETE（日付）」を追記。

---

## T1/T2/T3 アトミック適用

`cmd_check` シグネチャ変更（driver.rs）と main.rs 更新は同時適用する。
コンパイルエラーを防ぐため、Cargo.toml 更新・テスト挿入も同一ステップで完了させる。

---

## テスト設計

```rust
// v431000_tests — v43.10.0 のテストモジュール
// 注意: v43100_tests は v43.1.0 の既存モジュールのため使用不可

#[test]
fn cargo_toml_version_is_43_10_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("43.10.0"), "Cargo.toml must contain version 43.10.0");
}

#[test]
fn explain_output_empty_for_well_typed_code() {
    let src = r#"
fn add(a: Int, b: Int) -> Int { a + b }
"#;
    let out = super::collect_explain_output(src, "v431000_test.fav");
    assert!(out.is_empty(), "well-typed code must produce empty explain output: {:?}", out);
}
```
