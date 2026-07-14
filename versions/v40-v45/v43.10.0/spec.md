# v43.10.0 仕様書 — `fav check --explain`

## 概要

`fav check --explain` フラグを追加する。型チェックエラー発生時、エラーコードに対応する自然言語解説を出力する。

既存の `get_explain_text` 関数（E0001〜E0021 の静的解説テキスト）を活用する。

---

## 背景

v43.9.0 で `--show-inference` を追加し、型推論の可視化を実現した。
次のステップとして、型チェックエラー時に開発者が原因を素早く把握できるよう、
エラーコードと紐づいた解説テキストを `fav check` の出力に統合する。

ロードマップには「v39 の Llm Rune を活用」と記載されているが、本バージョンでは
**静的解説テキスト（`get_explain_text`）を使う MVP 実装**を採用する。
理由: LLM 呼び出しはネットワーク依存・レイテンシ・API キー要件があり、
CI 環境での安定動作が保証できない。v43.10.0 では静的解説で基盤を固め、
LLM 統合は後続バージョンで対応する（ロードマップの Llm Rune 活用は将来版へスライド）。
→ ロードマップ `roadmap-v43.1-v44.0.md` の v43.10.0 エントリも「静的解説ベース MVP」に修正する。

---

## 機能仕様

### `fav check --explain`

```
fav check src/main.fav --explain
```

**動作:**
1. 通常の型チェックを実行する（既存動作と同一）
2. エラーが検出された場合、各エラー出力の直後に `get_explain_text(e.code)` で解説テキストを出力する
3. エラーがない場合は通常通り `no errors found` を表示する（`--explain` は出力を変更しない）
4. `--json` と `--explain` が同時に指定された場合、`--explain` は無効化される（警告なし）
5. プロジェクトモード（`fav.toml` 指定、`file = None`）では `--explain` は非対応。単一ファイルモードのみ。

**出力フォーマット（エラーあり時）:**
```
src/main.fav: E0001 undefined variable: x
  Explain: ...（get_explain_text("E0001") の返値）
```

`e` は `TypeError` 構造体（`TypeError.code: &'static str`）。
解説テキストが存在しない場合（`get_explain_text` が `None` を返す場合）は解説行を省略する。

---

## 実装方針

### driver.rs

- `collect_explain_output(src: &str, filename: &str) -> Vec<String>` を追加（テスト用ヘルパー）
  - `run_checker_fav` は `Result<(), Vec<String>>` を返すため、`Err(msgs)` を `msgs_to_type_errors(msgs)` で `Vec<TypeError>` に変換してから `e.code` にアクセスする
- `cmd_check` シグネチャ末尾に `explain: bool` を追加（12 番目のパラメータ）
- エラー出力ループ内で `explain` が `true` かつ `!json` の場合、`get_explain_text(e.code)` を呼び出し
  `Some(text)` であれば `println!("  Explain: {}", text)` を出力する

### main.rs

- `let mut explain = false;` を追加
- `"--explain" => { explain = true; i += 1; }` を追加（`--show-inference` の直後）
- `cmd_check(...)` 呼び出しに `explain` を末尾引数として追加

### テスト（`v431000_tests`）

> **注意**: `v43100_tests` は v43.1.0 のモジュール名として既存のため使用不可。
> v43.10.0 のモジュール名は **`v431000_tests`** とする（43.10.0 = 43×10000 + 10×100 + 0）。

1. `cargo_toml_version_is_43_10_0` — `Cargo.toml` に `"43.10.0"` が含まれる
2. `explain_output_empty_for_well_typed_code` — 正常コードでは `collect_explain_output` が空 Vec を返す

---

## スコープ外

- LLM 呼び出しによるリアルタイム解説生成（→ 将来バージョン）
- `--json` 出力への `explain` フィールド統合（→ 将来バージョン）
- E0022 以降の新規エラーコードへの解説追加（→ 各エラー追加バージョンで対応）
- プロジェクトモードでの `--explain`（→ 将来バージョン）

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で 2929 tests passed, 0 failed
- `v431000_tests` 2 件 pass
- `fav check <well_typed.fav> --explain` が `no errors found` を表示する
- `Cargo.toml` version = `43.10.0`
