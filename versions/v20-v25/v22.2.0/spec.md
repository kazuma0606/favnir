# v22.2.0 仕様書 — Distributed `par`（複数 Worker への分散）

## 概要

既存のシングルマシン `par [A, B]` を複数マシンに分散する `par_distributed [A, B, C]` 構文を追加する。
`fav.toml` の `[workers]` セクションで Worker エンドポイントを宣言し、
分散実行が利用できない場合はローカル並列実行にフォールバックする。

**テーマ**: 「単一マシンを越えたパイプライン並列実行」

---

## ロードマップ完了条件との対応

v22.2 は Distributed Scale ロードマップ（v22.1〜v23.0）の第二弾。
v23.0 の完了条件②「`par_distributed [A, B, C]` が 3 台の Worker で並列実行できる」に向けた第一段階
（構文・TOML・スレッドローカル・スタブのみ。実ネットワーク gRPC 実行は v22.3+）。

---

## 機能仕様

### `par_distributed` 構文

```favnir
// Worker プールを使った分散並列実行
seq DistributedReport = par_distributed [FetchOrders, FetchPrices, FetchInventory] |> Merge
```

- `par_distributed` はソフトキーワード（Ident トークンとして解析）
- `FlwStep::ParDistributed(Vec<String>)` として AST に格納される
- 既存の `par [A, B]`（`FlwStep::Par`）と並立する別バリアント
- Worker が未設定の場合は `IO.par_execute_raw` にフォールバック（ローカル並列）

### `fav.toml` の `[workers]` セクション

```toml
[workers]
endpoints = [
  "grpc://worker-1:9090",
  "grpc://worker-2:9090",
  "grpc://worker-3:9090",
]
```

| フィールド | 型 | 説明 |
|---|---|---|
| `endpoints` | `Vec<String>` | Worker gRPC エンドポイントのリスト |

### 実行フロー（v22.2.0 スコープ）

```
fav run pipeline.fav
  └─ fav.toml の [workers].endpoints を読む
     ├─ endpoints が空 → IO.par_execute_raw（ローカル並列）
     └─ endpoints あり → IO.par_distributed_raw（スタブ：ログ出力 + ローカルフォールバック）
```

> **注意**: v22.2.0 では実際の gRPC ネットワーク転送は実装しない（スタブ）。
> 実際の Worker ノードへの stage バイトコード転送は v22.3/v23.x で対応。

---

## アーキテクチャ

### 追加する AST バリアント（`ast.rs`）

`FlwStep` enum に `ParDistributed` を追加:

```rust
pub enum FlwStep {
    Stage(String),
    Par(Vec<String>),
    /// v22.2.0: `par_distributed [A, B, ...]` — distributed parallel execution
    ParDistributed(Vec<String>),
    Tap(Box<Expr>),
    Inspect,
}
```

`stage_names()` / `display_str()` の match も更新。

### パーサー変更（`frontend/parser.rs`）

`parse_flw_step` に `par_distributed` ソフトキーワードのハンドラを追加:

```rust
} else if self.peek_ident_text("par_distributed") {
    self.advance(); // consume "par_distributed"
    self.expect(&TokenKind::LBracket)?;
    // ... parse names list
    Ok(FlwStep::ParDistributed(names))
}
```

`parse_seq_def` の先頭チェック（`if self.peek() == &TokenKind::Par`）も
`par_distributed` ソフトキーワードのケースを追加。

### コンパイラ変更（`middle/compiler.rs`）

`build_step_call` の `FlwStep::Par` と同様のパターンで `ParDistributed` を処理:

```rust
FlwStep::ParDistributed(names) => {
    // IO.par_distributed_raw(["A","B","C"], input)
    // エンドポイントは vm.rs スレッドローカルから取得
    ...
}
```

`display_str_for_step` / デバッグ IR 出力も更新。

### TOML 変更（`toml.rs`）

```rust
#[derive(Debug, Clone, Default)]
pub struct WorkersConfig {
    /// gRPC エンドポイントのリスト（例: "grpc://worker-1:9090"）
    pub endpoints: Vec<String>,
}
```

`FavToml` に `pub workers: Option<WorkersConfig>` フィールドを追加。
`parse_fav_toml` の `[workers]` セクション解析を追加。

### VM スレッドローカル（`backend/vm.rs`）

```rust
// v22.2.0: Distributed Worker endpoints
thread_local! {
    static WORKER_ENDPOINTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn set_worker_endpoints(endpoints: Vec<String>)
pub fn get_worker_endpoints() -> Vec<String>
```

### `IO.par_distributed_raw` builtin（`backend/vm.rs`）

```
引数: (names: List<String>, input: VMValue)
動作:
  1. WORKER_ENDPOINTS を読む
  2. 空 → IO.par_execute_raw と同じロジックを直接実行（ローカル並列実行）
  3. 非空 → eprintln!("[par_distributed] distributing to {} workers", endpoints.len())
            → ローカル並列フォールバック（実際のgRPC転送は v22.3+）

※ 実装上の注意: call_builtin の再帰（self.call_builtin("IO.par_execute_raw", args)）は
  args が by-value で消費されるため使えない場合がある。
  IO.par_execute_raw のロジックを直接コピーするか共通 private fn に抽出すること。
```

### `cmd_run` 拡張（`driver.rs`）

`cmd_run` 内で fav.toml の `[workers]` を読み取り、エンドポイントを VM に設定:

```rust
// v22.2.0: Worker endpoints from fav.toml
// find_root はディレクトリを受け取るため .parent() が必要
let worker_endpoints = file
    .and_then(|f| std::path::Path::new(f).parent())
    .and_then(|dir| FavToml::find_root(dir))
    .and_then(|root| FavToml::load(&root))
    .and_then(|toml| toml.workers)
    .map(|w| w.endpoints)
    .unwrap_or_default();
vm::set_worker_endpoints(worker_endpoints);
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `FlwStep::ParDistributed(Vec<String>)` 追加、`stage_names`/`display_str` 更新 |
| `fav/src/frontend/parser.rs` | 更新 | `parse_flw_step` に `par_distributed` ソフトキーワード追加、`parse_flw_def_or_binding` の先頭 par チェック更新 |
| `fav/src/middle/compiler.rs` | 更新 | `build_step_call` / `display_str_for_step` に `ParDistributed` アーム追加 |
| `fav/src/middle/checker.rs` | 更新 | `FlwStep::Par` と同等として `ParDistributed` の exhaustive match を追加（エフェクト伝播・型チェックは Par と共有） |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `FlwStep` exhaustive match に `ParDistributed` アーム追加 |
| `fav/src/emit_python.rs` | 更新 | `FlwStep` exhaustive match 追加、`has_par` フラグを `ParDistributed` でも `true` に更新 |
| `fav/src/toml.rs` | 更新 | `WorkersConfig` struct 追加、`FavToml.workers` フィールド追加、`[workers]` 解析 |
| `fav/src/backend/vm.rs` | 更新 | `WORKER_ENDPOINTS` thread-local、`set/get_worker_endpoints`、`IO.par_distributed_raw` builtin |
| `fav/src/driver.rs` | 更新 | `cmd_run` で fav.toml workers 読み取り、`v222000_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "22.1.0"` → `"22.2.0"` |
| `CHANGELOG.md` | 更新 | v22.2.0 エントリ追加 |
| `site/content/docs/cli/par-distributed.mdx` | 新規 | `par_distributed` / `[workers]` ドキュメント |

---

## テスト一覧（v222000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_2_0` | Cargo.toml に `version = "22.2.0"` が含まれる |
| `par_distributed_parsed` | `seq Foo = par_distributed [A, B, C]` が `FlwStep::ParDistributed(["A","B","C"])` としてパースされる |
| `workers_config_parsed` | `[workers]\nendpoints = [...]` が `WorkersConfig.endpoints` に正しく格納される |
| `set_and_get_worker_endpoints` | `set_worker_endpoints` → `get_worker_endpoints` でエンドポイントが一致する |
| `changelog_has_v22_2_0` | CHANGELOG.md に `[v22.2.0]` が含まれる |

---

## スコープ外（v22.2 では実装しない）

- 実際の gRPC ネットワーク経由での stage バイトコード転送（v22.3/v23.x）
- `fav worker --port N` Worker サーバープロセス（v22.3）
- Worker ノードへのロードバランシング・フェイルオーバー
- stage 出力のシリアライズ（`.favc` 形式での Worker 間転送）
- `[workers: Worker.Pool]` シグネチャ型（ロードマップの構文案）
- Worker 認証・TLS

---

## 完了条件

- [ ] `par_distributed [A, B, C]` が `FlwStep::ParDistributed` としてパースされる
- [ ] `FlwStep::ParDistributed` が `FlwStep::stage_names()` / `display_str()` で正しく処理される
- [ ] `build_step_call` が `ParDistributed` を `IO.par_distributed_raw` に変換する
- [ ] `fav.toml` の `[workers].endpoints` が `WorkersConfig.endpoints` に格納される
- [ ] `WORKER_ENDPOINTS` thread-local と `set/get_worker_endpoints` が vm.rs に存在する
- [ ] `IO.par_distributed_raw` が `call_builtin` に登録されている
- [ ] `cmd_run` が fav.toml の `[workers]` を読み取り vm に設定する
- [ ] `cargo test v222000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1846 件以上合格）
- [ ] `CHANGELOG.md` に v22.2.0 エントリ
- [ ] `site/content/docs/cli/par-distributed.mdx` 作成済み
