# v22.2.0 — Distributed `par`（複数 Worker への分散）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — `FlwStep::ParDistributed` 追加

- [x] **事前確認**: `grep -rn "FlwStep" fav/src/` で `FlwStep` を match しているファイルを全て特定する（compiler.rs / checker.rs / ast_lower_checker.rs / emit_python.rs 等が対象）
- [x] **事前確認**: `grep -n "FlwStep\|Par(" fav/src/ast.rs` で `FlwStep` enum の構造を確認
- [x] `FlwStep::Par(Vec<String>)` の直後に `ParDistributed(Vec<String>)` バリアントを追加（コメント付き）
- [x] `stage_names()` の match に `FlwStep::ParDistributed(names)` を追加（Par と or-pattern でまとめ）
- [x] `display_str()` の match に `FlwStep::ParDistributed(names) => format!("par_distributed [{}]", names.join(", "))` を追加
- [x] `cargo check --bin fav` でコンパイルエラー箇所を確認（exhaustive match 破損箇所を特定する）

---

### T2: `fav/src/frontend/parser.rs` — `par_distributed` ソフトキーワード解析

- [x] **事前確認**: `parse_flw_step` / `parse_flw_def_or_binding` の位置を確認
- [x] `parse_flw_step` の `par` ブランチ直後に `par_distributed` ブランチを追加
  - `self.peek_ident_text("par_distributed")` でソフトキーワードを検出
  - `par` と同じ `[A, B, C]` リスト解析
  - `FlwStep::ParDistributed(names)` を返す
- [x] `parse_flw_def_or_binding` の先頭 par チェック（L1950 付近）を `|| self.peek_ident_text("par_distributed")` に拡張
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/middle/compiler.rs` + `checker.rs` + `ast_lower_checker.rs` + `emit_python.rs` — exhaustive match 更新

- [x] **事前確認**: `grep -n "FlwStep::Par\b" fav/src/middle/compiler.rs` で全 `FlwStep::Par` マッチ箇所を確認
- [x] `build_step_call` に `FlwStep::ParDistributed(names)` アームを追加（`IO.par_distributed_raw` を呼ぶ IR を生成）
- [x] `flw_step_name` に `ParDistributed` アームを追加
- [x] `build_step_call_ctx` の `FlwStep::Par(_)` を `Par(_) | ParDistributed(_)` に更新
- [x] `fav/src/middle/checker.rs` の `FlwStep::Par` match 箇所に `ParDistributed` を追加（型チェック・エフェクト伝播は `Par` と同等）
- [x] `first_stage` / `last_stage` helpers に `ParDistributed` を追加（or-pattern）
- [x] `fav/src/middle/ast_lower_checker.rs` の `lower_flw_step` に `ParDistributed` アーム追加
- [x] `fav/src/emit_python.rs` の `has_par` フラグを `Par(_) | ParDistributed(_)` に更新
- [x] `emit_python.rs` の `build_chain_expr` と `emit_flw_with_par` の `Par` マッチを or-pattern に更新
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/toml.rs` — `WorkersConfig` + `FavToml.workers` 追加

- [x] `WorkersConfig { endpoints: Vec<String> }` struct を追加（`#[derive(Debug, Clone, Default)]`）
- [x] `FavToml` に `pub workers: Option<WorkersConfig>` フィールドを追加（`registry_url` フィールドの直後）
- [x] `parse_fav_toml` に `"[workers]"` セクションハンドラを追加
  - `"endpoints"` キーを `Vec<String>` として解析
- [x] `FavToml` の `workers: workers_cfg` を FavToml return に追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/backend/vm.rs` — `WORKER_ENDPOINTS` thread-local + `IO.par_distributed_raw`

- [x] `WORKER_ENDPOINTS: RefCell<Vec<String>>` thread-local を追加
- [x] `set_worker_endpoints(endpoints: Vec<String>)` 公開関数を追加
- [x] `get_worker_endpoints() -> Vec<String>` 公開関数を追加
- [x] `"IO.par_distributed_raw"` を `call_builtin` に追加（`"IO.par_execute_raw"` アームの直後）
  - Worker エンドポイントが設定されている場合 `eprintln!` でログ出力
  - `IO.par_execute_raw` のロジックを直接コピー（再帰呼び出しは使用不可）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `cmd_run` workers 設定 + `v222000_tests`

- [x] **事前確認**: `grep -n "set_checkpoint_stages\|set_checkpoint_dir" fav/src/driver.rs` で checkpoint setup の位置を確認
- [x] `cmd_run` の checkpoint setup の直後に workers setup コードを追加
- [x] `v221000_tests::version_is_22_1_0` に `#[ignore]` を追加
- [x] `v222000_tests` モジュールを `v221000_tests` の直後に追加（5 テスト）
  - `version_is_22_2_0`
  - `par_distributed_parsed`
  - `workers_config_parsed`
  - `set_and_get_worker_endpoints`
  - `changelog_has_v22_2_0`
- [x] `cargo test v222000 --bin fav` — 5/5 PASS 確認
- [x] `cargo test --bin fav` — リグレッションなし（1850 件合格）確認
- [x] `FavToml { ... }` 初期化箇所（checker.rs / resolver.rs / driver.rs）に `workers: None` を追加

---

### T7: `Cargo.toml` + `CHANGELOG.md` + `site/content/docs/cli/par-distributed.mdx`

- [x] `fav/Cargo.toml` の `version = "22.1.0"` → `"22.2.0"` に変更
- [x] v22.2.0 エントリを `CHANGELOG.md` の先頭（v22.1.0 エントリの上）に追加
- [x] `grep "[v22.2.0]" CHANGELOG.md` で追加確認
- [x] `site/content/docs/cli/par-distributed.mdx` を新規作成

---

## テスト一覧（v222000_tests、5 件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_22_2_0` | Cargo.toml に `version = "22.2.0"` が含まれる | PASS |
| `par_distributed_parsed` | `seq Foo = par_distributed [A, B, C]` が `FlwStep::ParDistributed` としてパースされる | PASS |
| `workers_config_parsed` | `[workers]\nendpoints = [...]` が `WorkersConfig.endpoints` に格納される | PASS |
| `set_and_get_worker_endpoints` | `set_worker_endpoints` → `get_worker_endpoints` でエンドポイントが一致する | PASS |
| `changelog_has_v22_2_0` | CHANGELOG.md に `[v22.2.0]` が含まれる | PASS |

---

## 完了条件チェックリスト

- [x] `FlwStep::ParDistributed` が AST に追加され `stage_names()`/`display_str()` で処理される
- [x] `seq Foo = par_distributed [A, B, C]` がパースできる
- [x] `build_step_call` が `ParDistributed` を `IO.par_distributed_raw` に変換する
- [x] `fav.toml` の `[workers].endpoints` が `WorkersConfig` に格納される
- [x] `WORKER_ENDPOINTS` thread-local と `set/get_worker_endpoints` が vm.rs に存在する
- [x] `IO.par_distributed_raw` が `call_builtin` に登録されている
- [x] `cmd_run` が fav.toml の `[workers]` を読み取り vm に設定する
- [x] `cargo test v222000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1850 件合格）
- [x] `CHANGELOG.md` に v22.2.0 エントリ
- [x] `site/content/docs/cli/par-distributed.mdx` 作成済み
