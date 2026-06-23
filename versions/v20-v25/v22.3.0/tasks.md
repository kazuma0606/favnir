# v22.3.0 — Pipeline State Rune（分散状態管理）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/ast.rs` — `Effect::PipelineState` 追加

- [x] **事前確認**: `grep -n "Checkpoint\|Trace\|pub enum Effect" fav/src/ast.rs | head -20` で `Effect` enum の末尾を確認（コマンドはリポジトリルート `C:\Users\yoshi\favnir` から実行）
- [x] `Effect::Trace` の直後に `/// v22.3.0: Pipeline distributed state` コメント付きで `PipelineState,` を追加
- [x] `cargo check --bin fav` でコンパイルエラー箇所を確認（exhaustive match 破損箇所を特定する）

---

### T2: `fav/src/frontend/parser.rs` — `!PipelineState` エフェクトパース

- [x] **事前確認**: `grep -n "\"Checkpoint\"\|\"Trace\"\|fn parse_one_effect\|fn parse_effects" fav/src/frontend/parser.rs | head -10` でエフェクト解析の位置を確認
- [x] `"Checkpoint"` アームの直後に `"PipelineState"` アームを追加
  ```rust
  "PipelineState" => {
      self.advance();
      Effect::PipelineState
  }
  ```
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/middle/checker.rs` — State namespace / effect / method 型

- [x] **事前確認**: `grep -n "\"Checkpoint\"\|require_checkpoint_effect\|E0308" fav/src/middle/checker.rs | head -20` で 3 箇所の Checkpoint パターン位置を確認

#### 3-1: namespace env 登録（L1539 付近）
- [x] `"Checkpoint"` の直後に `"State",   // v22.3.0` を追加

#### 3-2: 既知エフェクトリスト（L2455 付近）
- [x] `"Checkpoint"` の直後に `"PipelineState",  // v22.3.0` を追加

#### 3-3: フィールドアクセス型解決（L5477 付近）
- [x] `"Checkpoint"` の直後に `| "State"       // v22.3.0` を追加

#### 3-4: `require_state_effect` 関数
- [x] `require_checkpoint_effect` の直後に `require_state_effect(span: &Span)` 関数を追加（E0338）

#### 3-5: State メソッド型返却
- [x] Checkpoint アーム（L6386 付近）の直後に State メソッド 3 アームを追加
  - `("State", "get")` → `Type::Option(Box::new(Type::String))`
  - `("State", "set") | ("State", "delete")` → `Type::Unit`
  - `("State", "has")` → `Type::Bool`
  - 各アームで `self.require_state_effect(span)` を呼ぶ

- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/backend/vm.rs` — STATE_STORE + set_state_backend + builtins

- [x] **事前確認**: `grep -n "WORKER_ENDPOINTS\|set_worker_endpoints\|get_worker_endpoints" fav/src/backend/vm.rs | head -5` で WORKER_ENDPOINTS の位置を確認

#### 4-1: thread-local + 公開関数
- [x] `WORKER_ENDPOINTS` block の直後に `STATE_STORE` / `STATE_BACKEND` thread-local を追加
- [x] `set_state_backend(backend: &str)` 公開関数を追加
- [x] `get_state_value(key: &str) -> Option<String>` 公開関数を追加（テスト用）

#### 4-2: `is_known_builtin_namespace` に `"State"` を追加
- [x] `"Arena"` の直後に `| "State"   // v22.3.0` を追加

#### 4-3: 4 VM ビルトイン（`vm_call_builtin` 自由関数に追加、`Cache.*_raw` と同じ場所）
- [x] `Cache.delete_raw` アームの直後（または近傍）に `State.get_raw` / `State.set_raw` / `State.has_raw` / `State.delete_raw` を追加
  - **配置先**: `vm_call_builtin`（自由関数、エラー型 `Err(String)`）— `call_builtin`（method）ではない
  - `State.get_raw`: args[0]=key → `VMValue::Variant("some", Some(Box::new(VMValue::Str(v))))` / `VMValue::Variant("none", None)`
    - **注意**: `VMValue::Option` は存在しない → `Cache.get_raw`（vm.rs L16213）の `VMValue::Variant` パターンを参照
  - `State.set_raw`: args[0]=key, args[1]=value → `VMValue::Unit`（STATE_STORE に insert）
  - `State.has_raw`: args[0]=key → `VMValue::Bool(exists)`
  - `State.delete_raw`: args[0]=key → `VMValue::Unit`（STATE_STORE から remove）
  - **注意**: `VMValue::Option` の構築は `Cache.get_raw` の実装を参照すること

- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/toml.rs` — `StateConfig` + `FavToml.state`

- [x] **事前確認**: `grep -n "WorkersConfig\|pub workers\|workers_cfg\|\[workers\]" fav/src/toml.rs | head -10` で WorkersConfig パターンを確認

#### 5-1: `StateConfig` struct
- [x] `WorkersConfig` ブロックの直後に `StateConfig { backend: String }` struct を追加（`#[derive(Debug, Clone, Default)]`）
  - `Default` では `backend` が空文字列になるため、`cmd_run` 側で空の場合は `"memory"` にフォールバックする

#### 5-2: `FavToml` に `state` フィールドを追加
- [x] `pub workers: Option<WorkersConfig>` の直後に `pub state: Option<StateConfig>,` を追加（コメント付き）

#### 5-3: `parse_fav_toml` に state 変数 + section + handler を追加
- [x] `workers_cfg` 変数宣言の直後に `let mut state_cfg: Option<StateConfig> = None;` を追加
- [x] `if trimmed == "[workers]"` ブロックの直後に `if trimmed == "[state]"` ブロックを追加
- [x] `"workers"` アームの直後に `"state"` アームを追加（`backend` キーの解析）
- [x] `FavToml` return に `state: state_cfg,` を追加

#### 5-4: `FavToml { ... }` 直接初期化の全箇所に `state: None` を追加
- [x] `cargo check --bin fav` でコンパイルエラーを出し、全箇所（checker.rs / resolver.rs / driver.rs 等）に `state: None` を追加

- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `cmd_run` state 設定 + `v223000_tests`

- [x] **事前確認**: `grep -n "set_worker_endpoints\|v22.2.0: Worker" fav/src/driver.rs | head -5` で workers setup の位置を確認

#### 6-1: `cmd_run` state setup
- [x] workers setup ブロックの直後に state setup コードを追加（plan.md T6 のコードに従う）
  - **注意**: `.map(|s| if s.backend.is_empty() { "memory".to_string() } else { s.backend })` で空文字列にも "memory" フォールバック

#### 6-2: `v222000_tests::version_is_22_2_0` に `#[ignore]` を追加

#### 6-3: `v223000_tests` モジュールを `v222000_tests` の直後に追加（5 テスト）
  - `version_is_22_3_0`
  - `pipeline_state_effect_parsed`
  - `state_config_parsed`（`parse_fav_toml_pub` を使用）
  - `state_get_set_in_memory`
  - `changelog_has_v22_3_0`

- [x] **注意**: `state_get_set_in_memory` テストは set→get ラウンドトリップを検証する（`get_state_value` だけでなく）。
  そのため vm.rs に `pub fn set_state_value(key: &str, val: &str)` ヘルパーを T4-1 で追加すること（plan.md T4-1 参照）
- [x] `cargo test v223000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1851 件以上合格）を確認

---

### T7: `runes/state/state.fav` + `Cargo.toml` + `CHANGELOG.md` + MDX

- [x] **事前確認**: `grep "\[v22.2.0\]" CHANGELOG.md` で現在の先頭エントリを確認
- [x] `runes/state/state.fav` を新規作成（plan.md T7 のコードに従う）
  - `get` / `set` / `has` / `delete` の 4 関数
  - 各関数に `!PipelineState` エフェクトと Usage コメント
- [x] `fav/Cargo.toml` の `version = "22.2.0"` → `"22.3.0"` に変更
- [x] v22.3.0 エントリを `CHANGELOG.md` の先頭（v22.2.0 エントリの上）に追加
- [x] `grep "\[v22.3.0\]" CHANGELOG.md` で追加確認
- [x] `site/content/docs/runes/state.mdx` を新規作成
  - `State` Rune 概要 / `!PipelineState` エフェクト説明
  - API リファレンス（get/set/has/delete）
  - 使用例（重複排除パイプライン）
  - `fav.toml` の `[state]` 設定
  - 将来のバックエンド（v22.4+）への言及

---

## テスト一覧（v223000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_3_0` | Cargo.toml に `version = "22.3.0"` が含まれる |
| `pipeline_state_effect_parsed` | `stage Foo: Int -> Int !PipelineState` が `Effect::PipelineState` として `TrfDef.effects` に格納される（`fn` ではなく `stage` を使うこと） |
| `state_config_parsed` | `[state]\nbackend = "redis"` が `StateConfig.backend == "redis"` に格納される |
| `state_get_set_in_memory` | `set_state_value(key, val)` → `get_state_value(key)` のラウンドトリップ + 存在しないキーは `None` |
| `changelog_has_v22_3_0` | CHANGELOG.md に `[v22.3.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `Effect::PipelineState` が AST に追加される
- [x] `!PipelineState` エフェクトがパースされる
- [x] `State.*` 呼び出しに `!PipelineState` が必要（E0338）
- [x] `STATE_STORE` thread-local が `backend/vm.rs` に存在する
- [x] `State.get_raw` / `State.set_raw` / `State.has_raw` / `State.delete_raw` が VM builtin に登録される
- [x] `fav.toml` の `[state].backend` が `StateConfig.backend` に格納される
- [x] `cmd_run` が `[state].backend` を読み取り VM に設定する
- [x] `runes/state/state.fav` が作成される
- [x] `cargo test v223000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1851 件以上合格）
- [x] `CHANGELOG.md` に v22.3.0 エントリ
- [x] `site/content/docs/runes/state.mdx` 作成済み

---

## 優先度

```
T1（ast.rs）          ← 最初（T2/T3 の依存元）
T2（parser.rs）       ← T1 完了後
T3（checker.rs）      ← T1 完了後（T2 と並列可）
T4（vm.rs）           ← T1 完了後（T2/T3 と並列可）
T5（toml.rs）         ← T1 に非依存（並列可）
T6（driver.rs）       ← T1〜T5 完了後
T7（rune + Cargo + doc） ← T6 完了後
```

---

## コードレビュー指摘と対応

### 実装前レビュー（spec/plan/tasks）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| HIGH-1 | plan.md T4-3 が `VMValue::Option` を使用（存在しない型） | `VMValue::Variant("some"/"none")` に修正（plan.md 修正済み） |
| HIGH-2 | `pipeline_state_effect_parsed` テストが `fn foo: Int -> Int !PipelineState` 構文（無効）を使用 | `stage Foo: Int -> Int !PipelineState = \|n\| { n }` + `Item::TrfDef` に修正 |
| MED-3 | `state_get_set_in_memory` テストが get のみで set→get ラウンドトリップを検証していない | `set_state_value` ヘルパーを vm.rs に追加し、set→get ラウンドトリップを検証するよう修正 |
| MED-4 | `StateConfig.backend` が空文字列の場合に `unwrap_or_else` が保護できない | `.map(|s| if s.backend.is_empty() { "memory" } else { s.backend })` で空文字列対策を追加 |
| LOW-5 | tasks.md T4-3 の配置先（`vm_call_builtin` vs `call_builtin`）が不明瞭 | `vm_call_builtin`（自由関数）に明示。実装も同箇所に配置 |

### 実装後レビュー（コード）

| 優先度 | 指摘 | 対応 |
|---|---|---|
| MED-1 | `STATE_BACKEND` が State ビルトイン内で参照されていない | `set_state_backend` に `backend != "memory"` 時の警告 `eprintln!` を追加 |
| MED-2 | テスト用 `get_state_value` / `set_state_value` が `pub` で公開 | `pub(crate)` に変更 |
| MED-3 | E0338（`!PipelineState` なし呼び出し）のテストが欠落 | `state_call_without_effect_emits_e0338` テストを追加（6 件目）。6/6 PASS |
| LOW-4 | `emit_python.rs` の `map_effect` で `PipelineState` が `_` に落ちる | `Effect::PipelineState => "PipelineState"` を明示追加 |

## 実装メモ

- `Effect::PipelineState` 追加後、`lineage.rs` / `ast_lower_checker.rs` / `reachability.rs` / `driver.rs` / `fmt.rs` / `lint.rs` の 6 箇所で exhaustive match エラーが発生 → 全修正済み
- `FavToml` struct に `state` フィールド追加後、`driver.rs` / `checker.rs`（2 箇所）/ `resolver.rs`（3 箇所）で struct 初期化エラー → `state: None` を追加して全修正済み
- テスト結果: `cargo test v223000 --bin fav` — 5/5 PASS、`cargo test --bin fav` — 1855 PASS（0 failures）
