# v22.2.0 実装計画 — Distributed `par`

## 実装方針

既存の `par [A, B]`（`FlwStep::Par`）のパターンを踏襲して `par_distributed [A, B, C]`（`FlwStep::ParDistributed`）を追加する。
v22.2.0 では構文・TOML パーサー・スレッドローカル・builtin スタブを実装し、
実際の gRPC ネットワーク転送は v22.3+ に委ねる。

**Rust コードへの変更ファイル: 6 ファイル（ast.rs / parser.rs / compiler.rs / toml.rs / vm.rs / driver.rs）**

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `ast.rs` — `FlwStep::ParDistributed` 追加 | なし |
| T2 | `frontend/parser.rs` — `par_distributed` 解析 | T1 |
| T3 | `middle/compiler.rs` — `build_step_call` 更新 | T1 |
| T4 | `toml.rs` — `WorkersConfig` + `FavToml.workers` | なし（T1 と並列可） |
| T5 | `backend/vm.rs` — thread-local + `IO.par_distributed_raw` | T1 |
| T6 | `driver.rs` — `cmd_run` workers 設定 + `v222000_tests` | T1〜T5 |
| T7 | `Cargo.toml` バージョン更新 + CHANGELOG + MDX | T6 |

---

## T1: `fav/src/ast.rs` — `FlwStep::ParDistributed` 追加

### 事前確認
```bash
grep -n "FlwStep\|par_dist" fav/src/ast.rs | head -20
```

### 変更内容

`FlwStep::Par` の直後に `ParDistributed` を追加:

```rust
pub enum FlwStep {
    Stage(String),
    Par(Vec<String>),
    /// v22.2.0: `par_distributed [A, B, ...]` — distributed parallel execution across Worker nodes.
    ParDistributed(Vec<String>),
    Tap(Box<Expr>),
    Inspect,
}
```

`stage_names()` の match を更新:
```rust
FlwStep::ParDistributed(names) => names.iter().map(|s| s.as_str()).collect(),
```

`display_str()` の match を更新:
```rust
FlwStep::ParDistributed(names) => format!("par_distributed [{}]", names.join(", ")),
```

`cargo check --bin fav` でコンパイルエラーを確認（`FlwStep` の exhaustive match が壊れる箇所を特定する）。

---

## T2: `fav/src/frontend/parser.rs` — `par_distributed` 解析

### 事前確認
```bash
grep -n "fn parse_flw_step\|par_distributed\|par_exec\|peek_ident_text" fav/src/frontend/parser.rs | head -20
```
→ `peek_ident_text(text)` メソッドが存在するか確認。

### 変更内容

**`parse_flw_step` に `par_distributed` ブランチを追加**（`par` ブランチの直後）:

```rust
fn parse_flw_step(&mut self) -> Result<FlwStep, ParseError> {
    if self.peek() == &TokenKind::Par {
        // ... 既存の par [A, B] 処理
    } else if self.peek_ident_text("par_distributed") {
        // v22.2.0: par_distributed [A, B, C]
        self.advance(); // consume "par_distributed"
        self.expect(&TokenKind::LBracket)?;
        let (first, _) = self.expect_ident()?;
        let mut names = vec![first];
        while self.peek() == &TokenKind::Comma {
            self.advance();
            let (name, _) = self.expect_ident()?;
            names.push(name);
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(FlwStep::ParDistributed(names))
    } else if self.peek_ident_text("tap") {
        // ... 既存の tap 処理
    }
}
```

**`parse_flw_def_or_binding` の先頭チェックを更新**（L1950 付近、関数名に注意）:

`parse_seq_def` という関数は存在しない。実際のチェックは `parse_flw_def_or_binding` 内（L1950）にある:
```rust
// 変更前:
if self.peek() == &TokenKind::Par {
// 変更後:
if self.peek() == &TokenKind::Par || self.peek_ident_text("par_distributed") {
    let first_step = self.parse_flw_step()?;
    ...
}
```
この修正がないと `par_distributed [A, B, C]` が `FlwStep::Stage("par_distributed")` として誤パースされる。

`cargo check --bin fav` でコンパイルエラーが 0 であることを確認。

---

## T3: `fav/src/middle/compiler.rs` — `build_step_call` + `display_str_for_step` 更新

### 事前確認
```bash
grep -n "FlwStep::Par\b\|build_step_call\|display_str_for_step\|par_execute_raw" fav/src/middle/compiler.rs | head -20
```
→ `FlwStep::Par` を処理している全箇所を確認する。

### 変更内容

`build_step_call` の `FlwStep::Par(names)` アームと同じパターンで `ParDistributed` を追加:

```rust
FlwStep::ParDistributed(names) => {
    // IO.par_distributed_raw(["A","B","C"], input)
    // Worker エンドポイントは vm.rs スレッドローカルから取得するため引数不要
    let io_ns_idx = ctx.resolve_global("IO").unwrap_or(u16::MAX);
    let io_ns = || IRExpr::Global(io_ns_idx, Type::Unknown);
    // names リストを IRExpr として構築（Par と同じパターン）
    // 呼び出し先: IO.par_distributed_raw
    let par_fn = IRExpr::FieldAccess(
        Box::new(io_ns()),
        "par_distributed_raw".to_string(),
        Type::Unknown,
    );
    IRExpr::Call(Box::new(par_fn), vec![/* names list */, input], Type::Unknown)
}
```

**注意**: `FlwStep::Par` の実装（`IO.par_execute_raw` を構築するコード）を読んで同じパターンで実装する。
names リストの IRExpr 構築は `List.empty` + `List.push` の連鎖なので、`FlwStep::Par` のコードをコピーして
関数名だけ `"par_distributed_raw"` に変更する。

`display_str_for_step` / デバッグ IR 文字列関数にも `ParDistributed` アームを追加:
```bash
grep -n "fn display_str_for_step\|Par(names)" fav/src/middle/compiler.rs | head -10
```

`cargo check --bin fav` でコンパイルエラーが 0 であることを確認。

---

## T4: `fav/src/toml.rs` — `WorkersConfig` + `FavToml.workers`

### 事前確認
```bash
grep -n "pub struct FavToml\|pub workers\|WorkersConfig\|SnowflakeTomlConfig\|pub snowflake" fav/src/toml.rs | head -10
```
→ `FavToml` の既存フィールドのパターン（`Option<XxxConfig>`）を確認する。

### 変更内容

**`WorkersConfig` struct を追加**（既存の config struct 群の末尾付近）:

```rust
/// v22.2.0: [workers] セクション — 分散 par_distributed の Worker エンドポイント設定
#[derive(Debug, Clone, Default)]
pub struct WorkersConfig {
    /// gRPC Worker エンドポイントのリスト（例: "grpc://worker-1:9090"）
    pub endpoints: Vec<String>,
}
```

**`FavToml` に `workers` フィールドを追加**（`registry_url` フィールドの直後）:

```rust
/// Optional workers configuration (v22.2.0).
pub workers: Option<WorkersConfig>,
```

**`parse_fav_toml` に `[workers]` セクション解析を追加**:

既存の `"[snowflake]"` や `"[workers]"` セクション解析のパターンを確認し、同じ方法で実装する。

```
grep -n "\"\\[snowflake\\]\"\|\"\\[kafka\\]\"\|\"\\[aws\\]\"" fav/src/toml.rs | head -10
```

`WorkersConfig` のフィールド解析:
- `"endpoints"` → `Vec<String>` として解析（カンマ区切りリスト）

`cargo check --bin fav` でコンパイルエラーが 0 であることを確認。

---

## T5: `fav/src/backend/vm.rs` — thread-local + `IO.par_distributed_raw`

### 事前確認
```bash
grep -n "STAGE_CHECKPOINT_DIR\|set_checkpoint_dir\|IO.par_execute_raw\|par_execute_raw" fav/src/backend/vm.rs | head -15
```
→ v22.1.0 の `STAGE_*` thread-local パターンと `IO.par_execute_raw` の実装場所を確認する。

### スレッドローカル追加

`STAGE_CHECKPOINT_DIR` 等の直後に追加:

```rust
// ── v22.2.0: Distributed Worker endpoints ─────────────────────────────────────
thread_local! {
    static WORKER_ENDPOINTS: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

pub fn set_worker_endpoints(endpoints: Vec<String>) {
    WORKER_ENDPOINTS.with(|c| *c.borrow_mut() = endpoints);
}

pub fn get_worker_endpoints() -> Vec<String> {
    WORKER_ENDPOINTS.with(|c| c.borrow().clone())
}
```

### `IO.par_distributed_raw` builtin

`"IO.par_execute_raw"` アームの直後に追加。

**注意**: `self.call_builtin(artifact, "IO.par_execute_raw", args)` は `args` が by-value で消費されるため
再帰呼び出しができない場合がある（コンパイルエラーになるか、args が空になる）。
`IO.par_execute_raw` のロジックを直接コピーするか、共通 private fn（`fn par_execute_core`）に抽出すること。

```rust
// v22.2.0: distributed par execution
// args: [names: List<String>, input: VMValue]
// Falls back to local parallel (IO.par_execute_raw logic) when no Worker endpoints are configured.
"IO.par_distributed_raw" => {
    let endpoints = get_worker_endpoints();
    if !endpoints.is_empty() {
        eprintln!(
            "[par_distributed] {} worker(s) configured; local fallback active (gRPC dispatch in v22.3+)",
            endpoints.len()
        );
    }
    // IO.par_execute_raw と同じロジックを直接実行（再帰 call_builtin は args 消費問題のため不可）
    // → IO.par_execute_raw の実装（L4769〜）をコピーし関数名だけ変更する
    // （またはロジックを par_execute_core private fn に抽出して両方から呼ぶ）
    ... // IO.par_execute_raw のロジックをここに複製
}
```

`cargo check --bin fav` でコンパイルエラーが 0 であることを確認。

---

## T6: `fav/src/driver.rs` — `cmd_run` 更新 + `v222000_tests`

### `cmd_run` への workers 設定追加

checkpoint setup コードの直後に追加:

```rust
// v22.2.0: Worker endpoints from fav.toml [workers] section
{
    let worker_endpoints = file
        .and_then(|f| std::path::Path::new(f).parent())
        .and_then(|dir| crate::toml::FavToml::find_root(dir))
        .and_then(|root| crate::toml::FavToml::load(&root))
        .and_then(|toml| toml.workers)
        .map(|w| w.endpoints)
        .unwrap_or_default();
    crate::backend::vm::set_worker_endpoints(worker_endpoints);
}
```

### `v221000_tests::version_is_22_1_0` に `#[ignore]` を追加

### `v222000_tests` モジュール追加

`v221000_tests` の直後に追加:

```rust
// ── v222000_tests (v22.2.0) — Distributed par ────────────────────────────────
#[cfg(test)]
mod v222000_tests {
    use super::*;

    #[test]
    fn version_is_22_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.2.0\""), "Cargo.toml should have version 22.2.0");
    }

    #[test]
    fn par_distributed_parsed() {
        let src = "seq Foo = par_distributed [A, B, C]";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("lex");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::FlwDef(fd) = &prog.items[0] {
            assert_eq!(fd.steps.len(), 1);
            if let crate::ast::FlwStep::ParDistributed(names) = &fd.steps[0] {
                assert_eq!(names, &vec!["A".to_string(), "B".to_string(), "C".to_string()]);
            } else {
                panic!("expected FlwStep::ParDistributed");
            }
        } else {
            panic!("expected FlwDef");
        }
    }

    #[test]
    fn workers_config_parsed() {
        let toml_src = "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[workers]\nendpoints = [\"grpc://worker-1:9090\", \"grpc://worker-2:9090\"]\n";
        let config = crate::toml::parse_fav_toml_pub(toml_src);
        let workers = config.workers.expect("workers config should be present");
        assert_eq!(workers.endpoints.len(), 2);
        assert_eq!(workers.endpoints[0], "grpc://worker-1:9090");
    }

    #[test]
    fn set_and_get_worker_endpoints() {
        let endpoints = vec!["grpc://w1:9090".to_string(), "grpc://w2:9090".to_string()];
        crate::backend::vm::set_worker_endpoints(endpoints.clone());
        let got = crate::backend::vm::get_worker_endpoints();
        assert_eq!(got, endpoints);
        // reset
        crate::backend::vm::set_worker_endpoints(vec![]);
    }

    #[test]
    fn changelog_has_v22_2_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.2.0]"), "CHANGELOG should have v22.2.0 entry");
    }
}
```

`cargo test v222000 --bin fav` — 5/5 PASS を確認。

---

## T7: `Cargo.toml` + CHANGELOG + MDX

### Cargo.toml

`version = "22.1.0"` → `"22.2.0"`

### CHANGELOG.md（先頭に追加）

```markdown
## [v22.2.0] — 2026-06-21 — Distributed `par`（複数 Worker への分散）

### Added
- `par_distributed [A, B, C]` 構文（`FlwStep::ParDistributed`）— 複数 Worker への分散並列実行
- `fav.toml` の `[workers]` セクション（`WorkersConfig.endpoints`）
- `vm::set_worker_endpoints` / `get_worker_endpoints` スレッドローカル設定
- `IO.par_distributed_raw` VM builtin（Worker 未設定時はローカル並列フォールバック）
- `site/content/docs/cli/par-distributed.mdx` ドキュメント

### Changed
- `fav/Cargo.toml` バージョンを `22.2.0` に更新
```

### `site/content/docs/cli/par-distributed.mdx`

`par_distributed` 構文・`[workers]` 設定・フォールバック動作を説明するドキュメント。

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `FlwStep` の exhaustive match 破損（compiler.rs / checker.rs 等） | T1 完了直後に `cargo check` でエラー箇所を特定 |
| `parse_flw_step` で `par_distributed` がソフトキーワードとして機能しない | `peek_ident_text("par_distributed")` の存在を事前確認 |
| `parse_seq_def` の先頭 par チェックが `par_distributed` を見逃す | T2 で条件を `|| self.peek_ident_text("par_distributed")` に拡張 |
| `IO.par_distributed_raw` の再帰呼び出し（`call_builtin` → `IO.par_execute_raw`） | 無限再帰にならないよう `IO.par_execute_raw` のロジックを直接コピーする方が安全 |
| `WorkersConfig` の `Default` derive が不要なフィールドに副作用を持つ | `Default` は `endpoints: Vec<String>` のみのため問題なし |

---

## 実装上の注意点

### `IO.par_distributed_raw` の実装方針

`call_builtin` 内で `self.call_builtin(artifact, "IO.par_execute_raw", args)` と再帰呼び出しするのは
args が消費されるため不可。代わりに `IO.par_execute_raw` のロジックを直接コピーするか、
共通ヘルパー private fn を抽出する。

### `FlwStep::ParDistributed` の `stage_names()` 更新

`FlwStep::stage_names()` の match は `Par(names)` と同じパターンで追加する。
`display_str()` も同様。

### toml.rs の `[workers]` パース

`parse_fav_toml` の section ハンドラは `if line == "[workers]"` の形で判定している。
`WorkersConfig` の `endpoints` は `Vec<String>` 型なのでリスト解析が必要。
既存の `[kafka]` または `[aws]` セクションで `Vec<String>` を解析している箇所を参考にする:
```bash
grep -n "Vec<String>\|endpoints\|kafka" fav/src/toml.rs | head -20
```
