# Favnir v9.5.0 Tasks

Date: 2026-06-01
Theme: HTTP / gRPC / GraphQL — 型付き API アクセス層の整備 + `!Http` エフェクト追加

---

## Phase A: Effect::Http 追加（Rust 8 ファイル）

- [x] A-1: `src/ast.rs` — `Effect::Http` variant を追加（Network と Rpc の間）
- [x] A-2: `src/frontend/parser.rs` — `"Http" => Effect::Http` を parse_effect_ann に追加
- [x] A-3: `src/fmt.rs` — `Effect::Http => Some("!Http".to_string())` を追加
- [x] A-4: `src/lineage.rs` — `Http => "!Http".into()` を追加
- [x] A-5: `src/driver.rs` — effect_to_string match に `ast::Effect::Http => "Http".into()` 追加（2箇所）
- [x] A-6: `src/middle/ast_lower_checker.rs` — `ast::Effect::Http => "Http".to_string()` を追加
- [x] A-7: `src/middle/checker.rs`
  — `BUILTIN_EFFECTS` に `"Http"` を追加
  — `require_network_effect` を `Effect::Network | Effect::Http` に緩和
- [x] A-8: `src/middle/reachability.rs` — `Effect::Http => { ... }` を追加
- [x] A-9: `cargo build` — exhaustive match エラーなし確認

---

## Phase B: vm.rs — 新 primitive 追加

- [x] B-1: `Http.get_body_raw(url: String) -> Result<String, String>` を vm.rs に追加
- [x] B-2: `Http.post_body_raw(url: String, body: String, ct: String) -> Result<String, String>` を vm.rs に追加
- [x] B-3: `src/middle/checker.rs` に型シグネチャを追加

---

## Phase C: checker.fav 更新

- [x] C-1: `fn http_fn(fname: String) -> String` を追加
- [x] C-2: `fn grpc_fn(fname: String) -> String` を追加
- [x] C-3: `builtin_ret_ty` に `Http` / `Grpc` を追加（else if 構文）
- [x] C-4: `ns_to_effect` に `Http => "Http"` / `Grpc => "Rpc"` を追加
- [x] C-5: self-check 通過確認

---

## Phase D: http Rune 拡張（`runes/http/request.fav`）

- [x] D-1: `public fn get_text(url: String) -> Result<String, String> !Http` を追加
- [x] D-2: `public fn get_json<T>(url: String) -> Result<T, String> !Http` を追加
  — `Http.get_body_raw` → `Json.parse_raw` → `Schema.adapt_one` 直接インライン
- [x] D-3: `public fn post_json_typed<T, R>(url: String, body: T) -> Result<R, String> !Http` を追加
  — `Http.post_body_raw` → `Json.parse_raw` → `Schema.adapt_one` 直接インライン
- [x] D-4: `runes/http/http.fav` の use 文を更新
- [x] D-5: 既存 http rune テスト通過確認

---

## Phase E: grpc Rune 拡張（`runes/grpc/client.fav`）

- [x] E-1: `public fn call_json<T>(...)` を追加
  — `Grpc.call_raw` → `Grpc.encode_raw` → `Json.parse_raw` → `Schema.adapt_one`
- [x] E-2: `public fn call_list<T>(...)` を追加
- [x] E-3: `runes/grpc/grpc.fav` の use 文を更新
- [x] E-4: 既存 grpc rune テスト通過確認

---

## Phase F: graphql Rune 新規作成（`runes/graphql/`）

- [x] F-1: `runes/graphql/rune.toml` を作成
- [x] F-2: `runes/graphql/client.fav` を作成（query<T> / mutate<T>）
- [x] F-3: `runes/graphql/graphql.fav` を作成（エントリポイント）
- [x] F-4: `runes/graphql/graphql.test.fav` を作成（3テスト）

---

## Phase G: 統合テスト（`fav/src/driver.rs`）

- [x] G-1: `http_effect_http_accepted` — !Http 宣言で E0003 が出ないこと
- [x] G-2: `http_effect_missing_errors` — 未宣言で E0003 が出ること
- [x] G-3: `http_get_body_raw_err_on_bad_url` — 不正 URL でエラーを返すこと
- [x] G-4: `lineage_http_effect_in_sources` — !Http が lineage Sources に表示される
- [x] G-5: `graphql_rune_test_file_passes` — graphql.test.fav 全テスト通過
- [x] G-6: `cargo test v950` — 5 件全通過確認

---

## Phase H: self-check + Bootstrap 検証

- [x] H-1: `cargo test checker_fav_wire_self_check` — self-check 通過
- [x] H-2: `cargo test bootstrap` — bytecode_A == bytecode_B 維持確認
- [x] H-3: `cargo test` — 1187 件全通過

---

## Phase I: ドキュメント・バージョン更新

- [x] I-1: `fav/Cargo.toml` の version を `"9.5.0"` に更新
- [x] I-2: `fav/self/cli.fav` のバージョン文字列を `"9.5.0"` に更新
- [x] I-3: `versions/v9.5.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] I-4: `memory/MEMORY.md` に v9.5.0 完了を記録
- [x] I-5: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `!Http` が `fav check` で有効なエフェクトとして認識される | ✓ |
| `Http.*` 呼び出しが `!Http` / `!Network` どちらでもコンパイル可 | ✓ |
| `!Network` 宣言の既存コードが引き続き動作する | ✓ |
| `http.get_text(url)` が `Result<String, String> !Http` で動作する | ✓ |
| `http.get_json<T>(url)` が型付きレスポンスを返す | ✓ |
| `http.post_json_typed<T, R>(url, body)` が動作する | ✓ |
| `grpc.call_json<T>(host, method, payload)` が動作する | ✓ |
| `grpc.call_list<T>(host, method, payload)` が動作する | ✓ |
| `graphql.query<T>(url, gql)` が動作する | ✓ |
| `graphql.mutate<T>(url, gql)` が動作する | ✓ |
| `fav explain --lineage` が `!Http` を Sources に表示する | ✓ |
| `checker.fav` が Http / Grpc 名前空間を認識する | ✓ |
| 既存 http / grpc テストが引き続き通過する | ✓ |
| `cargo test` 全件通過（1187 件） | ✓ |
