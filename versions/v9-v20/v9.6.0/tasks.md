# Favnir v9.6.0 Tasks

Date: 2026-06-01
Theme: LLM Rune — `!Llm` エフェクト + Claude / OpenAI 統合

---

## Phase A: Effect::Llm 追加（Rust 8 ファイル）

- [x] A-1: `src/ast.rs` — `Effect::Llm` variant を追加（Http の直後）
- [x] A-2: `src/frontend/parser.rs` — `"Llm" => Effect::Llm` を parse_effect_ann に追加
- [x] A-3: `src/fmt.rs` — `Effect::Llm => Some("!Llm".to_string())` を追加
- [x] A-4: `src/lineage.rs` — `Llm => "!Llm".into()` を追加
- [x] A-5: `src/driver.rs` — effect_to_string match に `ast::Effect::Llm => "Llm".into()` 追加（2箇所）
- [x] A-6: `src/middle/ast_lower_checker.rs` — `ast::Effect::Llm => "Llm".to_string()` を追加
- [x] A-7: `src/middle/checker.rs`
  — `BUILTIN_EFFECTS` に `"Llm"` を追加
  — `Llm.*` 関数の型シグネチャを追加
- [x] A-8: `src/middle/reachability.rs` — `Effect::Llm => { ... }` を追加
- [x] A-9: `cargo build` — exhaustive match エラーなし確認

---

## Phase B: vm.rs — 新 primitive 追加

- [x] B-1: `Llm.complete_raw(prompt: String) -> Result<String, String>` を vm.rs に追加
  — `LLM_PROVIDER` 環境変数で anthropic / openai を切り替え
  — `LLM_MODEL` 環境変数でモデル名を上書き可（デフォルト: claude-opus-4-6 / gpt-4o）
  — `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` を読み取り
- [x] B-2: `Llm.chat_raw(messages_json: String) -> Result<String, String>` を vm.rs に追加
  — messages_json は `[{role, content}, ...]` の JSON 文字列
- [x] B-3: `Llm.extract_raw(schema_name: String, prompt: String, data: String) -> Result<String, String>` を vm.rs に追加
  — プロンプトを組み立てて complete_raw 相当のロジックで呼ぶ
  — raw JSON 文字列を返す（Schema.adapt は Favnir 側で呼ぶ）
- [x] B-4: `src/middle/checker.rs` に Llm.* 型シグネチャを追加
- [x] B-5: `src/middle/compiler.rs` に `"Llm"` namespace を追加（2箇所）

---

## Phase C: checker.fav 更新

- [x] C-1: `fn llm_fn(fname: String) -> String` を追加
- [x] C-2: `builtin_ret_ty` に `else if ns == "Llm" { llm_fn(fname) }` を追加
- [x] C-3: `ns_to_effect` に `else if ns == "Llm" { "Llm" }` を追加
- [x] C-4: self-check 通過確認

---

## Phase D: llm Rune 新規作成（`runes/llm/`）

- [x] D-1: `runes/llm/rune.toml` を作成
- [x] D-2: `runes/llm/client.fav` を作成
  — `public fn complete(prompt: String) -> Result<String, String> !Llm`
  — `public fn chat(messages_json: String) -> Result<String, String> !Llm`
  — `public fn extract<T>(prompt: String, data: String) -> Result<T, String> !Llm`
    （Llm.extract_raw → Json.parse_raw → Schema.adapt_one の直接インライン）
- [x] D-3: `runes/llm/llm.fav` を作成（エントリポイント）
- [x] D-4: `runes/llm/llm.test.fav` を作成（3テスト）
  — `llm_complete_no_key_is_err`
  — `llm_chat_no_key_is_err`
  — `llm_extract_no_key_is_err`

---

## Phase E: 統合テスト（`fav/src/driver.rs`）

- [x] E-1: `llm_effect_llm_accepted` — `!Llm` 宣言で E0003 が出ないこと
- [x] E-2: `llm_effect_missing_errors` — 未宣言で E0003 が出ること
- [x] E-3: `lineage_llm_effect_in_sources` — `!Llm` が lineage Sources に表示される
- [x] E-4: `llm_rune_test_file_passes` — llm.test.fav 全テスト通過
- [x] E-5: `cargo test v960` — 4 件全通過確認

---

## Phase F: self-check + Bootstrap 検証

- [x] F-1: `cargo test checker_fav_wire_self_check` — self-check 通過
- [x] F-2: `cargo test bootstrap` — bytecode_A == bytecode_B 維持確認
- [x] F-3: `cargo test` — 1191 件全通過

---

## Phase G: ドキュメント・バージョン更新

- [x] G-1: `fav/Cargo.toml` の version を `"9.6.0"` に更新
- [x] G-2: `fav/self/cli.fav` のバージョン文字列を `"9.6.0"` に更新
- [x] G-3: `versions/v9.6.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] G-4: `memory/MEMORY.md` に v9.6.0 完了を記録
- [ ] G-5: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `!Llm` が `fav check` で有効なエフェクトとして認識される | ✓ |
| `llm.complete(prompt)` が Claude API に接続して結果を返す | ✓ |
| `llm.chat(messages_json)` がマルチターン会話を処理できる | ✓ |
| `llm.extract<T>(prompt, data)` が型付き構造体を返す | ✓ |
| `LLM_PROVIDER=openai` で OpenAI API に切り替えられる | ✓ |
| `fav explain --lineage` が `!Llm` を Sources に表示する | ✓ |
| `checker.fav` が Llm 名前空間を認識する（self-check 通過） | ✓ |
| APIキー不在時に `Err(...)` を返す（テスト 3 件） | ✓ |
| integration テスト 4 件以上通過 | ✓ |
| `cargo test` 全件通過（1191 件） | ✓ |
