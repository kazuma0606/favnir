# v30.7.0 仕様書 — fav run エラー時スタックトレース改善

## 概要

`fav run` / `fav test` 実行時に発生するランタイムエラーの表示品質を改善する。
`hint_for_runtime_error` 関数を追加し、エラー発生フレームがステージの場合は "in stage X" ラベルを付与する。

---

## 背景

ロードマップ v30.7 より:

**現状**:
```
vm error in <fn> @42: index out of bounds
```
情報が少なく、どの関数・どの行で起きたか分かりにくい。

**v30.7.0 実装後の目標**:
```
runtime error: index out of bounds
  in stage ValidateRows (src/stages.fav:34)
  at parse_csv_row (src/stages.fav:12)
  = ヒント: List.nth は範囲外アクセスで失敗します。List.get を使うと Option<T> で安全に取得できます。
```

> **ロードマップ目標との差異**:
> ロードマップ v30.7 の「目標」には `"called from src/main.fav:12:3 (EtlPipeline)"` 形式、
> ソース行テキスト表示（`|` マーカー）、列情報（`:col`）が含まれるが、
> `FvcArtifact` / `FvcFunction` にファイルパス情報がなく（`source_line: u32` のみ）、
> これらはアーティファクト構造の変更なしに実装できない。
> v30.7.0 はヒントメッセージとステージラベルを先行実装し、ファイル情報付与は後続バージョン（v31.x 以降）に持ち越す。

---

## スコープ

### IN SCOPE

- `hint_for_runtime_error`（新規プライベート関数、`driver.rs`）
  - エラーメッセージパターンマッチで `= ヒント:` を返す（3 パターン、具体 → 汎用の順）
- `format_runtime_error`（`driver.rs` 改善）
  - プレフィックスを `"RuntimeError:"` → `"runtime error:"` に統一
  - スタックトレースが空の場合も `fn_name` を保持した表示に改善
  - スタックフレームで「ステージ」を識別して "in stage X" ラベルを付与
  - ヒントを末尾に付加
- `v307000_tests`（3 件）

### OUT OF SCOPE

- `VMError` / `FvcArtifact` / `FvcFunction` 構造体の変更
- アーティファクトへのファイルパス情報付与
- ソース行テキストの表示（`|` マーカー）
- 列情報（`:col`）の表示
- `"called from"` 形式の呼び出し元チェーン表示
- `--verbose` / `--trace` フラグの変更
- site/ MDX 更新

---

## 実装仕様

### `hint_for_runtime_error`（新規 `pub(crate)` 関数、`driver.rs`）

より具体的なパターンを先に評価し、論理的な重複を避ける:

```rust
pub(crate) fn hint_for_runtime_error(message: &str) -> Option<&'static str> {
    if message.contains("global index out of bounds") || message.contains("constant index out of bounds") {
        Some("モジュールのインポートが不足している可能性があります。import 文を確認してください。")
    } else if message.contains("index out of bounds") {
        Some("List.nth は範囲外アクセスで失敗します。List.get を使うと Option<T> で安全に取得できます。")
    } else if message.contains("type error") {
        Some("型の不一致が発生しています。fav check で型エラーを事前に確認できます。")
    } else {
        None
    }
}
```

> **パターン順序の根拠**:
> `"global index out of bounds"` は `"index out of bounds"` を部分文字列として含むため、
> 汎用パターンを先に評価すると具体パターンへ到達しない。
> 具体 → 汎用の順で評価することで各パターンが独立して機能する。

### `format_runtime_error` 改善（`driver.rs`）

**変更前**:
```rust
fn format_runtime_error(source_file: &str, e: crate::backend::vm::VMError) -> String {
    if e.stack_trace.is_empty() {
        return format!("vm error in {} @{}: {}", e.fn_name, e.ip, e.message);
    }
    let mut msg = format!("RuntimeError: {}", e.message);
    for frame in &e.stack_trace {
        if frame.line == 0 {
            msg.push_str(&format!("\n  at {} ({})", frame.fn_name, source_file));
        } else {
            msg.push_str(&format!(
                "\n  at {} ({}:{})",
                frame.fn_name, source_file, frame.line
            ));
        }
    }
    msg
}
```

**変更後**:
```rust
fn format_runtime_error(source_file: &str, e: crate::backend::vm::VMError) -> String {
    if e.stack_trace.is_empty() {
        // fn_name / ip 情報を保持しつつプレフィックスを統一
        let mut msg = if e.fn_name == "<none>" {
            format!("runtime error: {}", e.message)
        } else {
            format!("runtime error: {}\n  in {} ({})", e.message, e.fn_name, source_file)
        };
        if let Some(hint) = hint_for_runtime_error(&e.message) {
            msg.push_str(&format!("\n  = ヒント: {}", hint));
        }
        return msg;
    }
    let mut msg = format!("runtime error: {}", e.message);
    for frame in &e.stack_trace {
        // Favnir ではステージ名はアッパーキャメルケース（例: ValidateRows）。
        // fn 名は小文字始まりが言語規約（W003 で推奨）。
        // 先頭文字が大文字の場合は "in stage X" ラベルを付与する。
        // ※ "<unknown>" / "<none>" は '<' 始まりのため誤検知しない。
        let is_stage = frame.fn_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        if is_stage {
            if frame.line == 0 {
                msg.push_str(&format!("\n  in stage {} ({})", frame.fn_name, source_file));
            } else {
                msg.push_str(&format!(
                    "\n  in stage {} ({}:{})",
                    frame.fn_name, source_file, frame.line
                ));
            }
        } else if frame.line == 0 {
            msg.push_str(&format!("\n  at {} ({})", frame.fn_name, source_file));
        } else {
            msg.push_str(&format!(
                "\n  at {} ({}:{})",
                frame.fn_name, source_file, frame.line
            ));
        }
    }
    if let Some(hint) = hint_for_runtime_error(&e.message) {
        msg.push_str(&format!("\n  = ヒント: {}", hint));
    }
    msg
}
```

### ステージ検出ルール

Favnir のステージ名はアッパーキャメルケース（`ValidateRows`、`LoadCsv` など）。
通常関数名は小文字始まり（`validate_row`、`parse_csv_row` など）が言語規約。

先頭文字が大文字 → `"in stage X"` ラベルを付与。
先頭文字が小文字（または `<`）→ 通常関数として `"at X"` 表示。

> **誤検知リスク**:
> ユーザーが大文字始まりの `fn` 名を使った場合（Favnir では非推奨）は誤って "in stage X" と表示される。
> これは将来 `VMError` に `is_stage: bool` フラグを追加することで解決できるが、v30.7.0 では許容範囲とする。

---

## テスト設計（v307000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_30_7_0` | `Cargo.toml` に `version = "30.7.0"` |
| 2 | `hint_for_runtime_error_works` | `hint_for_runtime_error` を直接呼び出し、3 パターンが `Some`・未知が `None` であることを確認 |
| 3 | `benchmark_v30_7_0_exists` | `benchmarks/v30.7.0.json` に `"30.7.0"` |

> テスト 2 は `hint_for_runtime_error` を `pub(crate)` にして実際の返り値を検証する。
> `include_str!` によるテキスト確認では条件分岐の正しさを保証できないため。

---

## 完了条件

- `Cargo.toml` version = `"30.7.0"`
- `hint_for_runtime_error` が `pub(crate)` で実装されている（3 パターン、具体 → 汎用の順）
- `format_runtime_error` が `"runtime error:"` プレフィックスを使用する
- `format_runtime_error` がステージ名を `"in stage X"` 形式で表示する
- `format_runtime_error` がヒントを `"  = ヒント: ..."` で末尾に付加する
- 空スタックトレース時も `fn_name` が保持される（`<none>` 以外の場合）
- `cargo test v307000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.7.0]` セクション
- `benchmarks/v30.7.0.json` 存在
- `versions/current.md` を v30.7.0 に更新（「最新安定版」欄を変更）
- `tasks.md` が COMPLETE
