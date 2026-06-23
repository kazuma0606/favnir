# v22.3.0 仕様書 — Pipeline State Rune（分散状態管理）

## 概要

複数の Worker をまたぐ状態を型安全に管理する `State` Rune を追加する。
キー・バリュー形式の分散状態ストアを `!PipelineState` エフェクトで型システムに統合し、
ステートフルなパイプライン処理（重複排除・カウンター・セッション管理など）を安全に記述できるようにする。

v22.3.0 はバックエンドとして**インメモリ HashMap（スレッドローカル）**を使用するスタブ実装。
Redis / DynamoDB / PostgreSQL への対応は v22.4+ で行う。

**テーマ**: 「分散パイプラインの型安全な状態管理」

---

## ロードマップ完了条件との対応

v22.3.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第三弾。
v23.0 の完了条件②「`par_distributed [A, B, C]` が 3 台の Worker で並列実行できる」を支える
状態共有基盤の第一段階（構文・型システム・インメモリスタブ）。

---

## 機能仕様

### `State` Rune — API

```favnir
import rune "state"

// キーに値をセット（!PipelineState 必須）
State.set(key: String, value: String) -> Unit

// キーの値を取得（存在しない場合は None）
State.get(key: String) -> Option<String>

// キーが存在するかチェック
State.has(key: String) -> Bool

// キーを削除
State.delete(key: String) -> Unit
```

### 使用例

```favnir
import rune "state"

// 重複排除 stage
stage DeduplicateRows: List<Row> -> List<Row> !PipelineState = |rows| {
  List.filter(rows, |r| {
    bind already <- State.has("seen:" ++ r.id)
    if already {
      false
    } else {
      bind _ <- State.set("seen:" ++ r.id, "1")
      true
    }
  })
}

seq DeduplicatedPipeline = LoadRows |> DeduplicateRows |> Save
```

### `!PipelineState` エフェクト

- `State.*` 呼び出しには `!PipelineState` エフェクトが必要
- エフェクトなしで呼び出した場合は **E0338** エラー
- エフェクト伝播ルール: `!PipelineState` を持つ fn を呼ぶ fn も `!PipelineState` が必要

### `fav.toml` の `[state]` セクション

```toml
[state]
backend = "memory"   # v22.3.0 は "memory" のみ対応
                     # v22.4+ で "redis" | "dynamodb" | "postgres" を追加予定
```

### 実行フロー（v22.3.0 スコープ）

```
fav run pipeline.fav
  └─ fav.toml の [state].backend を読む
     └─ "memory"（デフォルト） → インメモリ HashMap を使用
        （State.set/get/has/delete はスレッドローカル HashMap に直接アクセス）
```

> **注意**: v22.3.0 では実際の分散状態（Redis/DynamoDB）は実装しない。
> Worker 間で状態を共有するための外部バックエンド接続は v22.4+ で対応。

---

## アーキテクチャ

### Effect 追加（`ast.rs`）

```rust
pub enum Effect {
    // ... 既存 ...
    Checkpoint,
    Trace,
    /// v22.3.0: Pipeline distributed state (`!PipelineState`)
    PipelineState,
    // ...
}
```

### パーサー変更（`frontend/parser.rs`）

`parse_effect` の match に `"PipelineState"` アームを追加:

```rust
"PipelineState" => {
    self.advance();
    Effect::PipelineState
}
```

### 型チェッカー変更（`middle/checker.rs`）

3 箇所に `"State"` を追加:

1. **namespace env 登録**（L1530 付近）— `"State"` を `Type::Named` として環境に登録
2. **既知エフェクトリスト**（L2450 付近）— `"PipelineState"` を valid effects リストに追加
3. **フィールドアクセス型解決**（L5470 付近）— `"State"` を `Type::Unknown` を返す namespace リストに追加

`require_state_effect` 関数を追加（E0338）:

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

`check_call_on_namespace` の `("State", method)` アームを追加（Checkpoint パターンに倣う）:

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

### VM スレッドローカル（`backend/vm.rs`）

```rust
// v22.3.0: Pipeline State — in-memory backend
thread_local! {
    static STATE_STORE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static STATE_BACKEND: RefCell<String> = RefCell::new("memory".to_string());
}

pub fn set_state_backend(backend: &str)
pub fn get_state_value(key: &str) -> Option<String>  // テスト用
```

### VM ビルトイン（`backend/vm.rs`）

`is_known_builtin_namespace` に `"State"` を追加。

`call_builtin` に以下の 4 アームを追加:

```
"State.get_raw"    → args[0]: String → Option<String>
"State.set_raw"    → args[0]: String, args[1]: String → Unit
"State.has_raw"    → args[0]: String → Bool
"State.delete_raw" → args[0]: String → Unit
```

### TOML 変更（`toml.rs`）

```rust
#[derive(Debug, Clone, Default)]
pub struct StateConfig {
    /// バックエンド種別（"memory" | "redis" | "dynamodb" | "postgres"）
    pub backend: String,
}
```

`FavToml` に `pub state: Option<StateConfig>` フィールドを追加（`workers` フィールドの直後）。

`parse_fav_toml` の `[state]` セクション解析を追加。

### `cmd_run` 拡張（`driver.rs`）

```rust
// v22.3.0: State backend from fav.toml
let state_backend = file
    .and_then(|f| std::path::Path::new(f).parent())
    .and_then(|dir| FavToml::find_root(dir))
    .and_then(|root| FavToml::load(&root))
    .and_then(|toml| toml.state)
    .map(|s| s.backend)
    .unwrap_or_else(|| "memory".to_string());
vm::set_state_backend(&state_backend);
```

### `runes/state/state.fav` — Rune 実装

```favnir
// runes/state/state.fav — Pipeline State Rune (v22.3.0)
// In-memory backend (stub). Redis/DynamoDB/PostgreSQL in v22.4+.

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

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `Effect::PipelineState` 追加 |
| `fav/src/frontend/parser.rs` | 更新 | `"PipelineState"` エフェクトパース追加 |
| `fav/src/middle/checker.rs` | 更新 | `"State"` namespace 登録 / `"PipelineState"` 既知エフェクト追加 / `require_state_effect`（E0338）/ State メソッド型返却 |
| `fav/src/backend/vm.rs` | 更新 | `STATE_STORE` thread-local / `set_state_backend` / `get_state_value` / `"State"` in `is_known_builtin_namespace` / 4 VM ビルトイン |
| `fav/src/toml.rs` | 更新 | `StateConfig` struct / `FavToml.state` フィールド / `[state]` 解析 |
| `fav/src/driver.rs` | 更新 | `cmd_run` state 設定 / `v223000_tests`（5 件） |
| `runes/state/state.fav` | 新規 | State Rune 実装（get/set/has/delete） |
| `fav/Cargo.toml` | 更新 | `version = "22.2.0"` → `"22.3.0"` |
| `CHANGELOG.md` | 更新 | v22.3.0 エントリ追加 |
| `site/content/docs/runes/state.mdx` | 新規 | State Rune ドキュメント |

---

## テスト一覧（v223000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_3_0` | Cargo.toml に `version = "22.3.0"` が含まれる |
| `pipeline_state_effect_parsed` | `stage Foo: Int -> Int !PipelineState` が `Effect::PipelineState` として `TrfDef.effects` に格納される |
| `state_config_parsed` | `[state]\nbackend = "redis"` が `StateConfig.backend == "redis"` に格納される |
| `state_get_set_in_memory` | `set_state_value(key, val)` → `get_state_value(key)` のラウンドトリップが正しく動作する |
| `changelog_has_v22_3_0` | CHANGELOG.md に `[v22.3.0]` が含まれる |

---

## スコープ外（v22.3.0 では実装しない）

- Redis / DynamoDB / PostgreSQL バックエンド接続（v22.4+）
- TTL 付き状態（`State.set_ttl`）
- `State.Set`（分散セット型）— ロードマップの `State.get_set<String>` 構文
- Worker 間での状態同期（外部バックエンドなしでは不可能）
- 状態のシリアライズ・デシリアライズ（String のみ対応）
- トランザクション / アトミック操作

---

## 完了条件

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
