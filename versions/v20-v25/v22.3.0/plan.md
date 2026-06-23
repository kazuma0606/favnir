# v22.3.0 実装計画 — Pipeline State Rune（分散状態管理）

## 実装順序

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

## T1: `fav/src/ast.rs` — `Effect::PipelineState` 追加

### 事前確認コマンド

```bash
grep -n "Checkpoint\|Trace\|pub enum Effect" fav/src/ast.rs | head -20
```

### 実装

`Effect::Trace` の直後に `PipelineState` バリアントを追加:

```rust
    Trace,
    /// v22.3.0: Pipeline distributed state (`!PipelineState`)
    PipelineState,
    /// `Emit<EventType>`
```

### 確認

```bash
cargo check --bin fav
# exhaustive match 破損箇所を cargo check のエラーで確認し T3 で修正
```

---

## T2: `fav/src/frontend/parser.rs` — `!PipelineState` エフェクトパース

### 事前確認コマンド

```bash
grep -n "\"Checkpoint\"\|\"Trace\"\|parse_effect\|fn parse_one_effect" fav/src/frontend/parser.rs | head -10
```

### 実装

`"Checkpoint"` アームの直後に `"PipelineState"` アームを追加:

```rust
"Checkpoint" => {
    self.advance();
    Effect::Checkpoint
}
"PipelineState" => {
    self.advance();
    Effect::PipelineState
}
```

### 確認

```bash
cargo check --bin fav
```

---

## T3: `fav/src/middle/checker.rs` — State namespace / effect / method 型

### 事前確認コマンド

```bash
grep -n "\"Checkpoint\"\|\"State\"\|require_checkpoint_effect\|PipelineState" fav/src/middle/checker.rs | head -20
```

### 3-1: namespace env 登録（L1539 付近）

`"Checkpoint"` の直後に `"State"` を追加:

```rust
"Checkpoint",
"State",   // v22.3.0
```

### 3-2: 既知エフェクトリスト（L2455 付近）

`"Checkpoint"` の直後に `"PipelineState"` を追加（Effect::Unknown 検証リスト）:

```rust
"Checkpoint",
"PipelineState",  // v22.3.0
```

### 3-3: フィールドアクセス型解決（L5477 付近）

`"Checkpoint"` の直後に `"State"` を追加（`Type::Unknown` を返す namespace group）:

```rust
| "Checkpoint"
| "State"       // v22.3.0
| "Parquet"
```

### 3-4: `require_state_effect` 関数を追加

`require_checkpoint_effect` の直後に追加:

```rust
fn require_state_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::PipelineState)) {
        self.type_error(
            "E0338",
            "State.* call requires `!PipelineState` effect on enclosing fn/trf",
            span,
        );
    }
}
```

### 3-5: State メソッド型返却（L6386 の Checkpoint アームの直後）

```rust
("State", "get") => {
    self.require_state_effect(span);
    Some(Type::Option(Box::new(Type::String)))
}
("State", "set") | ("State", "delete") => {
    self.require_state_effect(span);
    Some(Type::Unit)
}
("State", "has") => {
    self.require_state_effect(span);
    Some(Type::Bool)
}
```

### 確認

```bash
cargo check --bin fav
```

---

## T4: `fav/src/backend/vm.rs` — STATE_STORE + builtins

### 事前確認コマンド

```bash
grep -n "WORKER_ENDPOINTS\|set_worker_endpoints\|is_known_builtin_namespace" fav/src/backend/vm.rs | head -10
```

### 4-1: STATE_STORE thread-local（WORKER_ENDPOINTS の直後）

```rust
// v22.3.0: Pipeline State — in-memory backend (default)
thread_local! {
    static STATE_STORE: std::cell::RefCell<std::collections::HashMap<String, String>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
    static STATE_BACKEND: std::cell::RefCell<String>
        = std::cell::RefCell::new("memory".to_string());
}

pub fn set_state_backend(backend: &str) {
    STATE_BACKEND.with(|c| *c.borrow_mut() = backend.to_string());
}

/// テスト用: STATE_STORE からキーを直接読む
pub fn get_state_value(key: &str) -> Option<String> {
    STATE_STORE.with(|c| c.borrow().get(key).cloned())
}

/// テスト用: STATE_STORE にキーを直接書き込む
pub fn set_state_value(key: &str, val: &str) {
    STATE_STORE.with(|c| c.borrow_mut().insert(key.to_string(), val.to_string()));
}
```

### 4-2: `is_known_builtin_namespace` に `"State"` 追加

`"Arena"` の直後に追加:

```rust
| "Arena"
| "State"    // v22.3.0
```

### 4-3: VM ビルトイン 4 アーム

**配置先**: `vm_call_builtin`（自由関数、`Cache.*_raw` と同じ場所、L16200 付近）。
`call_builtin`（method）ではなく `vm_call_builtin`（`fn vm_call_builtin(...)` free fn）に追加する。
エラー型は `Err(String)` であることに注意（`self.error(artifact, ...)` ではなく文字列リテラル）。

> **注意**: `VMValue::Option` は存在しない。`Cache.get_raw`（L16213）と同様に
> `VMValue::Variant("some"/"none")` を使うこと。

```rust
// v22.3.0: Pipeline State builtins (in-memory backend stub)
"State.get_raw" => {
    let key = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.get_raw requires a String key".to_string()),
    };
    let val = STATE_STORE.with(|c| c.borrow().get(&key).cloned());
    Ok(match val {
        Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Str(v)))),
        None    => VMValue::Variant("none".to_string(), None),
    })
}
"State.set_raw" => {
    let mut it = args.into_iter();
    let key = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.set_raw: key must be a String".to_string()),
    };
    let val = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.set_raw: value must be a String".to_string()),
    };
    STATE_STORE.with(|c| c.borrow_mut().insert(key, val));
    Ok(VMValue::Unit)
}
"State.has_raw" => {
    let key = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.has_raw: key must be a String".to_string()),
    };
    let exists = STATE_STORE.with(|c| c.borrow().contains_key(&key));
    Ok(VMValue::Bool(exists))
}
"State.delete_raw" => {
    let key = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return Err("State.delete_raw: key must be a String".to_string()),
    };
    STATE_STORE.with(|c| c.borrow_mut().remove(&key));
    Ok(VMValue::Unit)
}
```

### 確認

```bash
cargo check --bin fav
```

---

## T5: `fav/src/toml.rs` — `StateConfig` + `FavToml.state`

### 事前確認コマンド

```bash
grep -n "WorkersConfig\|pub workers\|\"workers\"\|workers_cfg" fav/src/toml.rs | head -10
```

### 5-1: `StateConfig` struct（`WorkersConfig` の直後）

```rust
// ── State config (v22.3.0) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct StateConfig {
    /// バックエンド種別 ("memory" | "redis" | "dynamodb" | "postgres")
    pub backend: String,
}
```

### 5-2: `FavToml` に `state` フィールドを追加（`workers` フィールドの直後）

```rust
/// State backend from `[state]` section (v22.3.0).
pub state: Option<StateConfig>,
```

### 5-3: `parse_fav_toml` に変数・section・handler を追加

変数宣言（`workers_cfg` の直後）:
```rust
let mut state_cfg: Option<StateConfig> = None;
```

section 判定（`"[workers]"` の直後）:
```rust
if trimmed == "[state]" {
    section = "state";
    continue;
}
```

handler（`"workers"` アームの直後）:
```rust
"state" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        if key == "backend" {
            let mut current = state_cfg.take().unwrap_or_default();
            current.backend = val.trim_matches('"').to_string();
            state_cfg = Some(current);
        }
    }
}
```

`FavToml` return（`workers: workers_cfg` の直後）:
```rust
state: state_cfg,
```

### 注意

`FavToml { ... }` を直接初期化している全箇所（checker.rs / resolver.rs / driver.rs など）に
`state: None` を追加する必要がある。`cargo check` でコンパイルエラー箇所を特定して修正。

### 確認

```bash
cargo check --bin fav
```

---

## T6: `fav/src/driver.rs` — `cmd_run` state 設定 + `v223000_tests`

### 6-1: `cmd_run` state setup（workers setup の直後）

```rust
// v22.3.0: State backend from fav.toml [state]
{
    let state_backend = file
        .and_then(|f| std::path::Path::new(f).parent())
        .and_then(|dir| crate::toml::FavToml::find_root(dir))
        .and_then(|root| crate::toml::FavToml::load(&root))
        .and_then(|toml| toml.state)
        // NOTE: backend が空文字列の場合（parse_kv でキーが欠落した等）も "memory" にフォールバック
        .map(|s| if s.backend.is_empty() { "memory".to_string() } else { s.backend })
        .unwrap_or_else(|| "memory".to_string());
    crate::backend::vm::set_state_backend(&state_backend);
}
```

### 6-2: `v222000_tests::version_is_22_2_0` に `#[ignore]` を追加

### 6-3: `v223000_tests` モジュールを `v222000_tests` の直後に追加

```rust
// ── v223000_tests (v22.3.0) — Pipeline State Rune ────────────────────────────
#[cfg(test)]
mod v223000_tests {
    use super::*;

    #[test]
    fn version_is_22_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.3.0\""), "Cargo.toml should have version 22.3.0");
    }

    #[test]
    fn pipeline_state_effect_parsed() {
        // NOTE: `fn` は `(params)` 形式が必須のため、エフェクト付き宣言には `stage` を使う
        let src = "stage Foo: Int -> Int !PipelineState = |n| { n }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::TrfDef(td) = &prog.items[0] {
            assert!(
                td.effects.contains(&crate::ast::Effect::PipelineState),
                "expected Effect::PipelineState in stage effects"
            );
        } else {
            panic!("expected TrfDef (stage) item");
        }
    }

    #[test]
    fn state_config_parsed() {
        let toml_src = "[state]\nbackend = \"redis\"\n";
        let parsed = crate::toml::parse_fav_toml_pub(toml_src);
        let state = parsed.state.expect("state config should be present");
        assert_eq!(state.backend, "redis");
    }

    #[test]
    fn state_get_set_in_memory() {
        crate::backend::vm::set_state_backend("memory");
        // thread-local なのでユニークなキーを使ってテスト間干渉を回避
        let key = "test_key_v223_roundtrip";
        // set → get ラウンドトリップ検証
        crate::backend::vm::set_state_value(key, "hello");
        let got = crate::backend::vm::get_state_value(key);
        assert_eq!(got, Some("hello".to_string()), "set then get should return value");
        // 存在しないキーは None
        let missing = crate::backend::vm::get_state_value("__nonexistent_v223__");
        assert!(missing.is_none(), "missing key should return None");
    }

    #[test]
    fn changelog_has_v22_3_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.3.0]"), "CHANGELOG should have v22.3.0 entry");
    }
}
```

### 注意: `state_get_set_in_memory` テストの詳細

`get_state_value` はテスト用の公開関数。`STATE_STORE` は thread-local なので他テストとの干渉に注意。
set 操作は `vm_call_builtin` を通さず直接 `STATE_STORE` を操作するか、
`"State.set_raw"` を `call_builtin` 経由で呼ぶ。

より確実なテスト方法: `STATE_STORE` に直接アクセスする `pub fn set_state_value(key, val)` ヘルパーを追加する。
ただしこれはテスト専用関数なので `#[cfg(test)]` または doc comment で明記する。

### 確認

```bash
cargo test v223000 --bin fav   # 5/5 PASS を確認
cargo test --bin fav           # リグレッションなし（1851 件以上）確認
```

---

## T7: Rune + Cargo.toml + CHANGELOG.md + MDX

### 7-1: `runes/state/state.fav` を新規作成

```favnir
// runes/state/state.fav — Pipeline State Rune (v22.3.0)
//
// 分散パイプラインで型安全な状態管理を提供する Rune。
// v22.3.0: インメモリ HashMap バックエンド（スタブ）
// v22.4+: Redis / DynamoDB / PostgreSQL バックエンド追加予定
//
// Usage:
//   bind _ <- State.set("seen:" ++ row.id, "1")    // !PipelineState
//   bind v <- State.get("seen:" ++ row.id)          // !PipelineState -> Option<String>
//   bind b <- State.has("seen:" ++ row.id)          // !PipelineState -> Bool
//   bind _ <- State.delete("seen:" ++ row.id)       // !PipelineState

public fn get(key: String) -> Option<String> !PipelineState {
    State.get_raw(key)
}

public fn set(key: String, value: String) -> Unit !PipelineState {
    State.set_raw(key, value)
}

public fn has(key: String) -> Bool !PipelineState {
    State.has_raw(key)
}

public fn delete(key: String) -> Unit !PipelineState {
    State.delete_raw(key)
}
```

### 7-2: `fav/Cargo.toml` バージョン更新

```
version = "22.2.0" → "22.3.0"
```

### 7-3: `CHANGELOG.md` に v22.3.0 エントリを先頭に追加

```markdown
## [v22.3.0] — 2026-06-21 — Pipeline State Rune（分散状態管理）

...
```

### 7-4: `site/content/docs/runes/state.mdx` を新規作成

内容:
- `State` Rune の概要
- `!PipelineState` エフェクト説明
- API リファレンス（get/set/has/delete）
- 使用例（重複排除パイプライン）
- `fav.toml` の `[state]` 設定
- 将来のバックエンド（v22.4+）への言及

---

## 主要な落とし穴・注意事項

1. **`FavToml` struct 初期化箇所**: `state: None` の追加漏れが多発しやすい。
   `cargo check` でコンパイルエラーを出してから一括修正する（checker.rs / resolver.rs / driver.rs 等）。

2. **`state_get_set_in_memory` テストの thread-local 干渉**:
   Rust の test runner はデフォルトでスレッド並列実行する。`STATE_STORE` は thread-local なので
   テスト間で同一スレッドが使われた場合に値が残る可能性がある。
   `set_state_value` ヘルパーを追加して明示的に初期化するか、ユニークなキーを使う。

3. **`Effect::PipelineState` の exhaustive match**:
   `ast.rs` に `Effect` を追加した後、`cargo check` で `Effect` に match している箇所を確認。
   `parser.rs` のエフェクト解析は `other => Effect::Unknown(...)` フォールバックがあるため
   exhaustive match 問題は起きないが、`checker.rs` の `check_effect_declaration` 等では
   `PipelineState` が既知エフェクトとして認識されるよう追加が必要。

4. **`State.*_raw` と WASM**:
   `STATE_STORE` は `std::collections::HashMap`（スレッドローカル）で、WASM でも使用可能。
   `IO.par_execute_raw` と異なり `std::thread::spawn` を使わないため cfg ガードは不要。

5. **`State.get_raw` の `VMValue::Option` 構築**:
   `VMValue::Option(Some(Box::new(v)))` / `VMValue::Option(None)` の型が既存コードと一致しているか確認。
   `Cache.get_raw` の実装パターンを参照すること。

---

## 完了条件チェックリスト

- [ ] `Effect::PipelineState` が AST に追加される
- [ ] `!PipelineState` エフェクトがパースされる
- [ ] `State.*` 呼び出しに `!PipelineState` が必要（E0338）
- [ ] `STATE_STORE` thread-local が `backend/vm.rs` に存在する
- [ ] `State.get_raw` / `State.set_raw` / `State.has_raw` / `State.delete_raw` が VM builtin に登録される
- [ ] `fav.toml` の `[state].backend` が `StateConfig.backend` に格納される
- [ ] `cmd_run` が `[state].backend` を読み取り VM に設定する
- [ ] `runes/state/state.fav` が作成される
- [ ] `cargo test v223000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1851 件以上合格）
- [ ] `CHANGELOG.md` に v22.3.0 エントリ
- [ ] `site/content/docs/runes/state.mdx` 作成済み
