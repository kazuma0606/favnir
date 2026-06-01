# Favnir v9.6.0 Implementation Plan

Date: 2026-06-01
Theme: LLM Rune — `!Llm` エフェクト + Claude / OpenAI 統合

---

## Phase A: Effect::Llm 追加（Rust 8 ファイル）

v9.5.0 の `Effect::Http` と完全に同じパターン。

1. `src/ast.rs` — `Llm` variant を `Http` の直後に追加
2. `src/frontend/parser.rs` — `"Llm" => Effect::Llm`
3. `src/fmt.rs` — `Effect::Llm => Some("!Llm".to_string())`
4. `src/lineage.rs` — `Llm => "!Llm".into()`
5. `src/driver.rs` — `ast::Effect::Llm => "Llm".into()`（2 箇所）
6. `src/middle/ast_lower_checker.rs` — `ast::Effect::Llm => "Llm".to_string()`
7. `src/middle/checker.rs`
   - `BUILTIN_EFFECTS` に `"Llm"` 追加
   - `Llm.*` 関数の型シグネチャ追加
8. `src/middle/reachability.rs` — `Effect::Llm` arm 追加
9. `cargo build` — exhaustive match エラーなし確認

---

## Phase B: vm.rs — Llm primitives 追加

### Llm.complete_raw
```rust
"Llm.complete_raw" => {
    // args[0]: prompt: String
    // 環境変数 LLM_PROVIDER で分岐（デフォルト: anthropic）
    // Anthropic: POST https://api.anthropic.com/v1/messages
    //   x-api-key: ANTHROPIC_API_KEY
    //   body: { model, max_tokens, messages: [{role:"user", content: prompt}] }
    //   → content[0].text を返す
    // OpenAI: POST https://api.openai.com/v1/chat/completions
    //   Authorization: Bearer OPENAI_API_KEY
    //   body: { model, messages: [{role:"user", content: prompt}] }
    //   → choices[0].message.content を返す
    // エラー時: Err(msg)
}
```

### Llm.chat_raw
```rust
"Llm.chat_raw" => {
    // args[0]: messages_json: String（[{role, content}, ...] の JSON）
    // プロバイダに応じてチャット API を呼ぶ
    // 最後のアシスタントメッセージのテキストを返す
}
```

### Llm.extract_raw
```rust
"Llm.extract_raw" => {
    // args[0]: schema_name: String
    // args[1]: prompt: String
    // args[2]: data: String
    // "Extract as JSON matching schema {schema_name}: {prompt}\n\nData: {data}"
    // → complete_raw で取得 → Schema.adapt_one で T にマッピング
    //   （vm.rs 内では String として返し、Favnir 側で Schema.adapt_one を呼ぶ）
}
```

---

## Phase C: checker.fav 更新

```favnir
fn llm_fn(fname: String) -> String {
    if fname == "complete_raw"  { "Result" }
    else if fname == "chat_raw" { "Result" }
    else if fname == "extract_raw" { "Result" }
    else { "Result" }
}
// builtin_ret_ty に追加:
else if ns == "Llm" { llm_fn(fname) }
// ns_to_effect に追加:
else if ns == "Llm" { "Llm" }
```

self-check 通過確認。

---

## Phase D: llm Rune 作成（`runes/llm/`）

### rune.toml
```toml
name = "llm"
version = "0.1.0"
entry = "llm.fav"
```

### client.fav
```favnir
public fn complete(prompt: String) -> Result<String, String> !Llm {
    Llm.complete_raw(prompt)
}

public fn chat(messages: List<Map<String, String>>) -> Result<String, String> !Llm {
    bind json <- Json.encode_raw(messages)
    Llm.chat_raw(json)
}

public fn extract<T>(prompt: String, data: String) -> Result<T, String> !Llm {
    match Llm.extract_raw(type_name_of<T>(), prompt, data) {
        Err(e) => Result.err(e)
        Ok(raw) =>
            match Json.parse_raw(raw) {
                Err(e) => Result.err(e)
                Ok(parsed) =>
                    match Schema.adapt_one(parsed, type_name_of<T>()) {
                        Err(_) => Result.err("llm.extract: schema error")
                        Ok(v)  => Result.ok(v)
                    }
            }
    }
}
```

### llm.fav（エントリポイント）
```favnir
import "client"
namespace llm
use client.{ complete, chat, extract }
```

### llm.test.fav（3 テスト）
- `llm_complete_no_key_is_err` — API キーなしでエラーを返す
- `llm_chat_no_key_is_err` — 同上
- `llm_extract_no_key_is_err` — 同上

---

## Phase E: 統合テスト（driver.rs）

- `llm_effect_llm_accepted` — `!Llm` 宣言で E0003 が出ないこと
- `llm_effect_missing_errors` — 未宣言で E0003 が出ること
- `lineage_llm_effect_in_sources` — `!Llm` が lineage Sources に表示される
- `llm_rune_test_file_passes` — llm.test.fav 全テスト通過
- `cargo test v960` — 全件通過確認

---

## Phase F: self-check + Bootstrap 検証

- `cargo test checker_fav_wire_self_check` — self-check 通過
- `cargo test bootstrap` — bytecode_A == bytecode_B 維持
- `cargo test` — 全件通過

---

## Phase G: ドキュメント・バージョン更新

- `fav/Cargo.toml` version → `"9.6.0"`
- `fav/self/cli.fav` バージョン文字列 → `"9.6.0"`
- `versions/v9.6.0/tasks.md` 完了チェック
- `memory/MEMORY.md` v9.6.0 完了を記録
- commit

---

## 実装上の注意点

1. **`Effect::Llm` の位置**: `ast.rs` で `Http` の直後に挿入（既存 match の網羅性エラーで漏れを検出）
2. **HTTP 実装を Llm primitive 内に隠蔽**: `!Http` は不要 — vm.rs 内で直接 `ureq` を使う
3. **JSON エンコード**: `serde_json` を直接使用（vm.rs 内）
4. **API キー不在時**: `Ok("")` ではなく `Err("LLM_PROVIDER key not set: ...")` を返す
5. **`extract_raw` の設計**: vm.rs は raw JSON 文字列を返し、Schema.adapt_one は Favnir 側（client.fav）で呼ぶ — スタックオーバーフロー回避のため generic 呼び出しチェーンを避ける
6. **モデル名**: Anthropic は `claude-opus-4-6`、OpenAI は `gpt-4o` をデフォルト（env `LLM_MODEL` で上書き可）
