# Favnir v9.6.0 Spec

Date: 2026-06-01
Theme: LLM Rune — `!Llm` エフェクト + Claude / OpenAI 統合

---

## 概要

v9.6.0 では Favnir に LLM（大規模言語モデル）統合を追加する。
専用エフェクト `!Llm` を導入し、Claude（Anthropic）および OpenAI の API を
`llm.complete` / `llm.chat` / `llm.extract<T>` の統一インターフェースで呼び出せるようにする。

v9.5.0 で完成した `!Http` パターンと同じ 8 ファイル追加手順でエフェクトを登録する。

---

## 設計方針

### エフェクト
- `!Llm` を新規追加（AST, parser, fmt, lineage, driver, ast_lower_checker, checker, reachability）
- `Http.*` 呼び出し（内部実装）は `!Llm` で隠蔽 — ユーザーは `!Http` を宣言しなくてよい

### プロバイダ切替
| 環境変数 | 値 | 備考 |
|---|---|---|
| `LLM_PROVIDER` | `anthropic`（デフォルト）/ `openai` | |
| `ANTHROPIC_API_KEY` | Anthropic API キー | |
| `OPENAI_API_KEY` | OpenAI API キー | |

### vm.rs Primitives
| Primitive | 引数 | 戻り値 |
|---|---|---|
| `Llm.complete_raw` | `prompt: String` | `Result<String, String>` |
| `Llm.chat_raw` | `messages: String`（JSON エンコード済み） | `Result<String, String>` |
| `Llm.extract_raw` | `schema_name: String, prompt: String, data: String` | `Result<String, String>` |

### Rune API（`runes/llm/`）
```favnir
public fn complete(prompt: String) -> Result<String, String> !Llm
public fn chat(messages: List<Map<String, String>>) -> Result<String, String> !Llm
public fn extract<T>(prompt: String, data: String) -> Result<T, String> !Llm
```

### checker.fav 追加
- `llm_fn(fname: String) -> String`
- `builtin_ret_ty` に `else if ns == "Llm" { llm_fn(fname) }`
- `ns_to_effect` に `else if ns == "Llm" { "Llm" }`

---

## ファイル構成

### 新規作成
```
runes/llm/
  rune.toml
  llm.fav          # エントリポイント（use client.*）
  client.fav       # complete / chat / extract<T>
  llm.test.fav     # 3 テスト
```

### 変更ファイル（Rust 8 ファイル）
1. `src/ast.rs` — `Effect::Llm` variant
2. `src/frontend/parser.rs` — `"Llm" => Effect::Llm`
3. `src/fmt.rs` — `Effect::Llm => Some("!Llm".to_string())`
4. `src/lineage.rs` — `Llm => "!Llm".into()`
5. `src/driver.rs` — `ast::Effect::Llm => "Llm".into()`（2 箇所）
6. `src/middle/ast_lower_checker.rs` — `ast::Effect::Llm => "Llm".to_string()`
7. `src/middle/checker.rs` — `BUILTIN_EFFECTS` + `Llm.*` 型シグネチャ
8. `src/middle/reachability.rs` — `Effect::Llm => { effects_required.insert("Llm".to_string()); }`

### 変更ファイル（vm.rs + checker.fav）
- `src/backend/vm.rs` — `Llm.complete_raw` / `Llm.chat_raw` / `Llm.extract_raw`
- `self/checker.fav` — `llm_fn` + builtin_ret_ty / ns_to_effect 追加

---

## 完了条件

| 条件 | |
|---|---|
| `!Llm` が `fav check` で有効なエフェクトとして認識される | |
| `llm.complete(prompt)` が Claude API に接続して結果を返す | |
| `llm.chat(messages)` がマルチターン会話を処理できる | |
| `llm.extract<T>(prompt, data)` が型付き構造体を返す | |
| `LLM_PROVIDER=openai` で OpenAI API に切り替えられる | |
| `fav explain --lineage` が `!Llm` を Sources に表示する | |
| `checker.fav` が Llm 名前空間を認識する | |
| integration テスト 2 件以上（モック OK）通過 | |
| `cargo test` 全件通過 | |
