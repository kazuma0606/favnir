# v38.7.0 spec — Llm Rune 強化（stream / function_call / embed）

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.7.0 |
| テーマ | Llm Rune に `stream` / `function_call` / `embed` の 3 関数を追加 |
| 前提 | v38.6.0 COMPLETE — `fav new --template rag-pipeline` 実装済み |
| 完了条件 | `v38700_tests` 全テスト pass・`cargo test` 0 failures（≥ 2773 件） |

## 背景と目的

v9.6.0 で実装した Llm Rune（`complete` / `chat` / `extract`）を強化する。
RAG パイプライン（v38.6.0）が使用する `llm.embed` を実際に動作させるほか、
ストリーミングレスポンスとツール呼び出しを追加することで AI エージェント型パイプラインの基盤を整える。

**ロードマップとのシグネチャ差分**:
ロードマップは `Llm.stream(ctx, prompt)` / `Llm.function_call(ctx, prompt, tools)` / `Llm.embed(ctx, text)` と
記載しているが、実装では `ctx` 引数を省略する（LLM 設定は環境変数 `LLM_PROVIDER` / `LLM_MODEL` から取得するため不要）。
また `tools` 引数は型付きオブジェクトではなく `tools_json: String`（JSON 文字列）として受け取る。

**想定動作**:
```bash
# embed（ベクトル化）
$ fav run src/rag.fav
# LLM_PROVIDER=openai の場合: OpenAI Embeddings API を呼び出し
# LLM_PROVIDER=anthropic（デフォルト）: Err("embedding not supported for anthropic provider...") を返す（API キー不要）

# stream（ストリーミング）
# v38.7.0 スコープ: collect-all 実装（ureq 同期）、true SSE は v39.x
$ fav run src/stream.fav

# function_call（ツール呼び出し）
$ fav run src/agent.fav
```

## 実装スコープ

### 1. `fav/src/backend/vm.rs` — 3 VM primitive 追加

既存の `Llm.extract_raw` ブロック（line 12891 付近）の末尾 `}` の直後、
`// ── Snowflake` セクションコメントの前に追加:

```rust
// ── Llm.stream_raw / Llm.function_call_raw / Llm.embed_raw (v38.7.0) ─────
"Llm.stream_raw" => {
    let prompt = vm_string(
        args.into_iter()
            .next()
            .ok_or_else(|| "Llm.stream_raw requires a prompt argument".to_string())?,
        "Llm.stream_raw",
    )?;
    // v38.7.0: collect-all 実装（ureq 同期）。true SSE streaming は v39.x で実装予定。
    Ok(llm_call_complete(&prompt))
}
"Llm.function_call_raw" => {
    if args.len() != 2 {
        return Err("Llm.function_call_raw requires 2 arguments (prompt, tools_json)".to_string());
    }
    let mut it = args.into_iter();
    let prompt = vm_string(it.next().unwrap(), "Llm.function_call_raw prompt")?;
    let tools_json = vm_string(it.next().unwrap(), "Llm.function_call_raw tools_json")?;
    let full_prompt = format!(
        "{}\n\nAvailable tools (JSON): {}\n\nRespond with JSON: {{\"name\": \"<tool>\", \"arguments\": {{...}}}}",
        prompt, tools_json
    );
    Ok(llm_call_complete(&full_prompt))
}
"Llm.embed_raw" => {
    let text = vm_string(
        args.into_iter()
            .next()
            .ok_or_else(|| "Llm.embed_raw requires a text argument".to_string())?,
        "Llm.embed_raw",
    )?;
    Ok(llm_embed(&text))
}
```

`llm_embed` ヘルパー関数を `llm_call_chat` 終端 `}` の直後（line 6677 付近、`// ── Snowflake helpers (v10.2.0)` コメントの前）に追加:

```rust
/// テキストをベクトル化して JSON 文字列を返す。
/// OpenAI provider の場合は /v1/embeddings を呼び出す。
/// Anthropic provider の場合は Err を返す（Anthropic は embedding API を提供しない）。
#[cfg(not(target_arch = "wasm32"))]
fn llm_embed(text: &str) -> VMValue {
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
    match provider.as_str() {
        "openai" => {
            let api_key = match std::env::var("OPENAI_API_KEY") {
                Ok(k) => k,
                Err(_) => return err_vm(VMValue::Str("OPENAI_API_KEY is not set".to_string())),
            };
            let model = std::env::var("LLM_EMBED_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string());
            let body = serde_json::json!({ "model": model, "input": text });
            match ureq::post("https://api.openai.com/v1/embeddings")
                .set("Authorization", &format!("Bearer {}", api_key))
                .set("Content-Type", "application/json")
                .send_string(&body.to_string())
            {
                Ok(resp) => {
                    let t = match resp.into_string() {
                        Ok(s) => s,
                        Err(e) => return err_vm(VMValue::Str(e.to_string())),
                    };
                    match serde_json::from_str::<serde_json::Value>(&t) {
                        Ok(v) => {
                            let vec_val = &v["data"][0]["embedding"];
                            ok_vm(VMValue::Str(vec_val.to_string()))
                        }
                        Err(e) => err_vm(VMValue::Str(e.to_string())),
                    }
                }
                Err(ureq::Error::Status(_, resp)) => {
                    err_vm(VMValue::Str(resp.into_string().unwrap_or_default()))
                }
                Err(ureq::Error::Transport(e)) => err_vm(VMValue::Str(e.to_string())),
            }
        }
        _ => err_vm(VMValue::Str(
            "Llm.embed_raw: embedding not supported for anthropic provider — set LLM_PROVIDER=openai".to_string(),
        )),
    }
}
```

**注意**:
- `#[cfg(not(target_arch = "wasm32"))]` は **`fn llm_embed` の直前のみ**付与する。ディスパッチアーム（`"Llm.embed_raw" => { ... }` ブロック）自体には付与しない（付与すると WASM ビルドで match アーム不在エラーになる）。これは既存の `llm_call_complete` 呼び出しパターンと同様。

### 2. `fav/src/driver.rs` — primitives テーブルに 3 エントリ追加

`Llm.extract_raw` エントリ（line 10437 付近）の直後に追加:

```rust
p!("Llm","Llm.stream_raw","(prompt: String) -> Result<String, String>",[""],true,
   "LLM にプロンプトを送り、ストリーミングレスポンスを収集して返す（v38.7.0: collect-all）。"),
p!("Llm","Llm.function_call_raw","(prompt: String, tools_json: String) -> Result<String, String>",[""],true,
   "LLM にツール定義を渡してツール呼び出し結果を JSON 文字列で返す。"),
p!("Llm","Llm.embed_raw","(text: String) -> Result<String, String>",[""],true,
   "テキストをベクトル化して JSON 配列文字列を返す（LLM_PROVIDER=openai 専用）。"),
```

### 3. `runes/llm/client.fav` — 3 公開関数追加

既存の `extract<T>` 関数の直後に追加:

```favnir
// stream — ストリーミングレスポンス（v38.7.0: collect-all 実装）
public fn stream(prompt: String) -> Result<String, String> {
    Llm.stream_raw(prompt)
}

// function_call — ツール呼び出し
// tools_json は [{"name": "...", "description": "...", "parameters": {...}}] の JSON 文字列
public fn function_call(prompt: String, tools_json: String) -> Result<String, String> {
    Llm.function_call_raw(prompt, tools_json)
}

// embed — テキストをベクトル化（LLM_PROVIDER=openai 専用）
// 戻り値は JSON 配列文字列 "[0.1, 0.2, ...]"
public fn embed(text: String) -> Result<String, String> {
    Llm.embed_raw(text)
}
```

### 4. `runes/llm/llm.fav` — use 宣言を更新

```favnir
use client.{ complete, chat, extract, stream, function_call, embed }
```

### 5. `runes/llm/llm.test.fav` — 3 テスト追加（Favnir テスト）

既存 3 テストの直後に追加:

```favnir
test "llm_stream_no_key_is_err" {
    bind result <- llm.stream("Hello")
    assert(Result.is_err(result))
}

test "llm_function_call_no_key_is_err" {
    bind result <- llm.function_call("Call a tool", "[]")
    assert(Result.is_err(result))
}

test "llm_embed_no_provider_is_err" {
    bind result <- llm.embed("Hello world")
    assert(Result.is_err(result))
}
```

**注意**: `llm_embed_no_provider_is_err` は `LLM_PROVIDER` が未設定（デフォルト `"anthropic"`）の場合、**API キーの有無に関わらず** `Err("Llm.embed_raw: embedding not supported for anthropic provider...")` を返すことを検証する。このテストは `ANTHROPIC_API_KEY` の unset に依存しない。

### 6. `driver.rs` — テストモジュール

#### `v38600_tests::cargo_toml_version_is_38_6_0` のスタブ化

```rust
// Stubbed: version bumped to 38.7.0 — assertion intentionally removed
```

#### `v38700_tests` モジュール新規追加（3 テスト）

```rust
// ── v38700_tests (v38.7.0) — Llm Rune 強化 ───────────────────────────────────
#[cfg(test)]
mod v38700_tests {
    #[test]
    fn cargo_toml_version_is_38_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("38.7.0"), "Cargo.toml must contain version 38.7.0");
    }

    #[test]
    fn changelog_has_v38_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v38.7.0]"), "CHANGELOG.md must contain [v38.7.0]");
    }

    #[test]
    fn llm_rune_enhanced_primitives_exist() {
        let src = include_str!("../backend/vm.rs");
        assert!(
            src.contains("Llm.stream_raw") && src.contains("Llm.function_call_raw") && src.contains("Llm.embed_raw"),
            "vm.rs must contain Llm.stream_raw, Llm.function_call_raw, Llm.embed_raw"
        );
    }
}
```

**注意**:
- `llm_rune_enhanced_primitives_exist` は `include_str!("backend/vm.rs")` を使用（`fav/src/driver.rs` から見た `fav/src/backend/vm.rs` への相対パス — `../` は不要）。
- `use super::*;` は不要（`include_str!` のみ使用）。

### 7. `CHANGELOG.md` — `[v38.7.0]` エントリ追加

```
## [v38.7.0] — 2026-07-10

### Added
- `Llm.stream_raw` VM primitive（collect-all 実装、true SSE は v39.x）
- `Llm.function_call_raw` VM primitive（ツール呼び出し JSON レスポンス）
- `Llm.embed_raw` VM primitive（OpenAI Embeddings API、`LLM_PROVIDER=openai` 専用）
- `llm_embed` ヘルパー関数 in `vm.rs`
- `stream` / `function_call` / `embed` 公開関数 in `runes/llm/client.fav`
- `v38700_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 8. その他ドキュメント更新

- `fav/Cargo.toml`: `38.6.0` → `38.7.0`
- `versions/current.md`: 最新安定版 → v38.7.0、次バージョン → v38.8.0
- `versions/roadmap/roadmap-v38.1-v39.0.md`: v38.7.0 を ✅ 完了済みにマーク・テスト件数を 3 件に更新
- `site/content/docs/runes/llm.mdx`（低優先度）: `stream` / `function_call` / `embed` 関数の説明を追記推奨（v38.8.0 cookbook 前に更新すること）

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v38.6.0 | 2770 |
| v38.7.0 追加分（Rust） | +3 |
| v38.7.0 期待値 | 2773 |

**注意**: `runes/llm/llm.test.fav` に 3 テストを追加するが、これは Favnir テストファイルであり `cargo test` のカウントには含まれない（`llm_rune_test_file_passes` テストが `run_fav_test_file_with_runes` で実行するため、個数に変動なし）。

## 注意事項

### `include_str!("../backend/vm.rs")` のパス

`driver.rs` からの相対パスは `"../backend/vm.rs"` となる（`fav/src/driver.rs` → `fav/src/backend/vm.rs`）。
`include_str!` は compile-time マクロのため、パスが誤っているとコンパイルエラーになる。
T2 の事前確認でパスを検証すること。

### `#[cfg(not(target_arch = "wasm32"))]` の付与

`llm_embed` 関数には `llm_call_chat` と同じく `#[cfg(not(target_arch = "wasm32"))]` を付与する。
WASM ビルドは `ureq` を使用しないため、この条件コンパイルが必要。

### `Llm.stream_raw` の実装方針

v38.7.0 では true SSE（Server-Sent Events）ストリーミングは実装しない。
`ureq` は同期 HTTP クライアントのため、ストリーミングは `llm_call_complete` の転用（collect-all）とする。
コメントに `// v38.7.0: collect-all 実装` を明記する。

### `llm.test.fav` の `llm_embed_no_provider_is_err` テスト

`LLM_PROVIDER` が未設定（デフォルト `"anthropic"`）の場合、`llm.embed` は `llm_embed` 内で即座に
`Err("Llm.embed_raw: embedding not supported for anthropic provider...")` を返す。
これは `ANTHROPIC_API_KEY` の有無に関係なく動作するため、環境変数 unset の前処理に依存しない。

一方 `llm_stream_no_key_is_err` / `llm_function_call_no_key_is_err` は `llm_call_complete` を呼び出し、
こちらは `ANTHROPIC_API_KEY` が unset であることが必要。
既存 `llm_rune_test_file_passes` テストが `remove_var("ANTHROPIC_API_KEY")` を実行してから
`run_fav_test_file_with_runes` を呼ぶため、これら 2 テストも確実に pass する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `vm.rs` に `Llm.stream_raw`, `Llm.function_call_raw`, `Llm.embed_raw` が含まれる | `llm_rune_enhanced_primitives_exist` テスト |
| 2 | `CHANGELOG.md` に `[v38.7.0]` が含まれる | `changelog_has_v38_7_0` テスト |
| 3 | `Cargo.toml` バージョンが `38.7.0` | `cargo_toml_version_is_38_7_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2773） | `cargo test` 実行結果 |
| 5 | `roadmap-v38.1-v39.0.md` の v38.7.0 が ✅ かつテスト件数が 3 件 | T9 後に目視確認 |
| 6 | `versions/current.md` が v38.7.0（最新安定版）に更新されている | T9 後に目視確認 |
