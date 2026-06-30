# v25.4.0 実装計画 — mysql Rune 実質化

## 実装順序

```
Step 0  Cargo.toml: version bump + mysql crate 追加
Step 1  ast.rs: Effect::MySQL 追加
Step 2  error_catalog.rs: E0321 追加
Step 3  fmt.rs / lineage.rs / emit_python.rs / lint.rs /
        reachability.rs / ast_lower_checker.rs: Effect::MySQL 対応（6 ファイル）
Step 4  checker.rs: require_mysql_effect / ns_to_inferred_effect / MySQL builtin fns 追加
Step 5  parser.rs: "MySQL" => Effect::MySQL アーム追加
Step 6  driver.rs: format_effects / effect_json_name に MySQL アーム追加
Step 7  vm.rs: MySQL.*_raw 6 件追加
Step 8  runes/mysql/mysql.fav: type MySqlConn + 6 関数 全面更新
Step 9  examples/mysql_orders_etl.fav: 新規作成
Step 10 site/content/docs/runes/mysql.mdx: 新規作成
Step 11 CHANGELOG.md: [v25.4.0] エントリ追加
Step 12 benchmarks/v25.4.0.json: 新規作成（test_count: 2000）
Step 13 driver.rs: v254000_tests 6 件追加
Step 14 cargo test v254000: 6 件 PASS 確認
Step 15 cargo test: 総テスト数 ≥ 2000 件 確認
Step 16 spec-reviewer レビュー実施
```

---

## 詳細実装手順

### Step 0 — Cargo.toml

```toml
# [target.'cfg(not(target_arch = "wasm32"))'.dependencies] セクションに追加
# redis = ... の直後に配置
# 実装時に crates.io で最新安定版を確認すること（v24 が存在しない場合は v23 にダウングレード）
mysql = { version = "24", default-features = false }
```

バージョンを `25.4.0` に bump:

```toml
version = "25.4.0"
```

### Step 1 — ast.rs: Effect::MySQL

```rust
// Redis, の後、AzureDb の前に挿入
Redis,
/// v25.4.0: MySQL Rune effect（外部 MySQL 専用。!Postgres とは独立）
MySQL,
AzureDb,
```

### Step 2 — error_catalog.rs: E0321

E0315（Postgres）/ E0320（Redis）の後（E0316〜E0319 は未割当の空き番号）、連番で E0321 を追加:

```rust
ErrorEntry {
    code: "E0321",
    title: "undeclared !MySQL effect",
    // ...
}
```

### Step 3 — 6 ファイル一括更新（Effect::MySQL 追加）

各ファイルの `Effect::Redis` アームの直後に `MySQL` を追加:

| ファイル | 追加場所 | 追加内容 |
|---|---|---|
| `fmt.rs` | `Effect::Redis =>` の後 | `Effect::MySQL => Some("!MySQL".to_string())` |
| `lineage.rs` | `ast::Effect::Redis =>` の後 | `ast::Effect::MySQL => { return ("write".into(), Some("DbWrite".into())) }` |
| `emit_python.rs` | `Effect::Redis =>` の後 | `Effect::MySQL => "MySQL"` |
| `lint.rs` | `Effect::Redis =>` の後 | `Effect::MySQL => "MySQL"` |
| `reachability.rs` | `Effect::Redis =>` の後 | `Effect::MySQL => effects_required.insert("MySQL".to_string())` |
| `ast_lower_checker.rs` | `ast::Effect::Redis =>` の後 | `ast::Effect::MySQL => "MySQL".to_string()` |

> **lineage.rs の分類**: MySQL は主に read/write の DB 操作。
> `("write", Some("DbWrite"))` に分類する（Redis の `CacheWrite` とは別）。

### Step 4 — checker.rs: require_mysql_effect + builtin fns

```rust
fn require_mysql_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::MySQL)) {
        self.type_error(
            "E0321",
            "MySQL.* call requires `!MySQL` effect on enclosing fn/stage",
            span,
        );
    }
}
```

`ns_to_inferred_effect` に追加:
```rust
"MySQL" => Some(Effect::MySQL),
```

builtin fns:
```rust
// MySQL (v25.4.0) — require !MySQL effect
// connect_raw の戻り型は Result<String, String>（checker レベル）。
// runes/mysql/mysql.fav では Result<MySqlConn, String> として公開しているが、
// MySqlConn(String) は名目型ラッパーであり checker は String として扱う
// （PgConn / RedisConn と同じパターン — 意図的な簡略化）。
("MySQL", "connect_raw") => {
    self.require_mysql_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("MySQL", "query_raw") => {
    self.require_mysql_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("MySQL", "execute_raw") => {
    self.require_mysql_effect(span);
    Some(Type::Result(Box::new(Type::Int), Box::new(Type::String)))
}
("MySQL", "transaction_begin_raw") | ("MySQL", "transaction_commit_raw") | ("MySQL", "transaction_rollback_raw") => {
    self.require_mysql_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
("MySQL", _) => {
    self.require_mysql_effect(span);
    Some(Type::Unknown)
}
```

### Step 5 — parser.rs

`"Redis"` アームの直後に追加:
```rust
"MySQL" => {
    self.advance();
    Effect::MySQL
}
```

### Step 6 — driver.rs: format_effects / effect_json_name

`format_effects` 関数の `Redis` アームの後:
```rust
ast::Effect::MySQL => "!MySQL".into(),
```

`effect_json_name` 関数の `Redis` アームの後:
```rust
ast::Effect::MySQL => "MySQL",
```

### Step 7 — vm.rs: MySQL.*_raw 6 件

Redis primitives セクションの直後に追加:

```
// ── MySQL (v25.4.0) ───────────────────────────────────────────────
// NOTE: 接続モデルは RedisConn / PgConn パターンと同様。
// 各 primitive は毎回 mysql::Conn::new(url) で接続を確立する（接続プールは v26.x 以降）。

"MySQL.connect_raw" => { ... PING 確認後 url を返す }
"MySQL.query_raw"   => { ... exec::<mysql::Row, _, _> → JSON 配列文字列 }
"MySQL.execute_raw" => { ... exec_drop → affected_rows() }
"MySQL.transaction_begin_raw"    => { ... exec_drop("BEGIN") }
"MySQL.transaction_commit_raw"   => { ... exec_drop("COMMIT") }
"MySQL.transaction_rollback_raw" => { ... exec_drop("ROLLBACK") }
```

**query_raw の実装要点**:
- `params_json` を `serde_json::Value` で解析し、`Vec<mysql::Value>` に変換
- `conn.exec::<mysql::Row, _, _>(sql, params)` で行を取得
- 各 `Row` を `{"col_name": "col_value", ...}` の JSON オブジェクトに変換
- 全行を JSON 配列としてシリアライズして返す

**mysql::Params の構築**:
- `mysql::Params::Positional(vec![mysql::Value::Bytes(b"val".to_vec()), ...])`
- JSON 文字列から `mysql::Value` への変換:
  - `Number` (整数) → `mysql::Value::Int(n)`
  - `Number` (浮動小数) → `mysql::Value::Float(f)`
  - `String` → `mysql::Value::Bytes(s.as_bytes().to_vec())`
  - `Bool` → `mysql::Value::Int(if b { 1 } else { 0 })`
  - `Null` → `mysql::Value::NULL`

**Row の JSON シリアライズ**:
- `mysql::Row` は `columns()` でカラム名取得、`get::<mysql::Value, _>(i)` で値取得
- `mysql::Value` → JSON: `Bytes(b)` → `String`, `Int(n)` → `Number`, etc.

### Step 8 — runes/mysql/mysql.fav

```favnir
// runes/mysql/mysql.fav — MySQL Rune (v25.4.0)
// 使い方: import rune "mysql"

type MySqlConn(String)

public fn connect(url: String) -> Result<MySqlConn, String> !MySQL {
    MySQL.connect_raw(url)
}

public fn query(conn: MySqlConn, sql: String, params: String) -> Result<String, String> !MySQL {
    MySQL.query_raw(conn, sql, params)
}

public fn execute(conn: MySqlConn, sql: String, params: String) -> Result<Int, String> !MySQL {
    MySQL.execute_raw(conn, sql, params)
}

public fn transaction_begin(conn: MySqlConn) -> Result<Unit, String> !MySQL {
    MySQL.transaction_begin_raw(conn)
}

public fn transaction_commit(conn: MySqlConn) -> Result<Unit, String> !MySQL {
    MySQL.transaction_commit_raw(conn)
}

public fn transaction_rollback(conn: MySqlConn) -> Result<Unit, String> !MySQL {
    MySQL.transaction_rollback_raw(conn)
}
```

### Step 9 — examples/mysql_orders_etl.fav

spec.md の Example をそのまま作成。

### Step 10 — site/content/docs/runes/mysql.mdx

- タイトル: MySQL Rune
- `!MySQL` エフェクトの説明
- 全 6 関数の API リファレンス（シグネチャ・説明・例）
- `!Postgres` との比較表（API 統一方針）
- transaction の使用例

### Step 11 — CHANGELOG.md

`[v25.4.0]` エントリを先頭に追加（`[v25.3.0]` の前）:

```markdown
## [v25.4.0] — 2026-06-25

### Added
- mysql Rune 実質化（「動く Rune」5 条件達成）
- `MySQL.connect` / `MySQL.query` / `MySQL.execute` / `MySQL.transaction_begin/commit/rollback`（6 関数）
- `Effect::MySQL`（`!MySQL` エフェクト）追加（11 ファイル更新）
- E0321「undeclared !MySQL effect」エラーコード追加
- `examples/mysql_orders_etl.fav`（注文 ETL デモ）
- `site/content/docs/runes/mysql.mdx`（API ドキュメント）
- `mysql` crate v25 を native-only 依存に追加
```

### Step 12 — benchmarks/v25.4.0.json

```json
{
  "version": "25.4.0",
  "timestamp": "2026-06-25T00:00:00Z",
  "metrics": {
    "test_count": 2000,
    "compile_hello_ms": 12,
    "compile_etl_ms": 45
  }
}
```

### Step 13 — driver.rs: v254000_tests 6 件

```rust
mod v254000_tests {
    fn mysql_rune_has_connect_fn()        // mysql.fav に "fn connect" を確認
    fn mysql_rune_has_query_execute()     // mysql.fav に "fn query" / "fn execute" を確認
    fn mysql_rune_has_transaction_fns()   // mysql.fav に "fn transaction_begin" を確認
    fn mysql_primitives_exist_in_vm()     // vm.rs に "MySQL.connect_raw" 等を確認
    fn mysql_orders_etl_example_exists()  // examples/mysql_orders_etl.fav を確認
    fn changelog_has_v25_4_0()            // CHANGELOG.md に v25.4.0 を確認
}
```

---

## 注意事項・既知リスク

| リスク | 対策 |
|---|---|
| `mysql` crate v24 がビルド失敗する場合 | `v23` にダウングレードして試す。`Cargo.lock` で実際のバージョンを確認 |
| `mysql::Conn::new(url)` の URL 形式 | `mysql://user:pass@host:port/db` 形式。ユーザー/パスなしは `mysql://host:port/db` |
| `exec::<mysql::Row>` の型推論 | ターボフィッシュ `conn.exec::<mysql::Row, _, _>(sql, params)` が必要 |
| `Row` のカラム名取得 | `row.columns()` でカラム情報を取得、`row.get::<mysql::Value, usize>(i)` で値取得 |
| transaction_begin/commit/rollback の VM 制約 | BEGIN/COMMIT/ROLLBACK を都度新規接続で実行するため、同一接続上のトランザクションではない（擬似実装）。コメントで明記すること |
| Effect::MySQL の exhaustive match | `cargo build` で漏れを確認。v25.3.0 の実績では 11 ファイル更新が必要 |

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version bump + mysql crate 追加 |
| `fav/src/ast.rs` | 更新 | `Effect::MySQL` 追加 |
| `fav/src/error_catalog.rs` | 更新 | E0321 追加 |
| `fav/src/fmt.rs` | 更新 | `Effect::MySQL` 表示文字列 |
| `fav/src/lineage.rs` | 更新 | `Effect::MySQL` リネージ分類 |
| `fav/src/emit_python.rs` | 更新 | `Effect::MySQL` アーム |
| `fav/src/lint.rs` | 更新 | `Effect::MySQL` アーム |
| `fav/src/middle/reachability.rs` | 更新 | `Effect::MySQL` アーム |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `Effect::MySQL` アーム |
| `fav/src/middle/checker.rs` | 更新 | `require_mysql_effect` / builtin fns |
| `fav/src/frontend/parser.rs` | 更新 | `"MySQL" => Effect::MySQL` |
| `fav/src/driver.rs` | 更新 | `format_effects` / `effect_json_name` + v254000_tests |
| `fav/src/backend/vm.rs` | 更新 | MySQL.*_raw 6 件追加 |
| `runes/mysql/mysql.fav` | 更新 | 全面更新（type MySqlConn + 6 関数） |
| `examples/mysql_orders_etl.fav` | 新規 | 注文 ETL デモ |
| `CHANGELOG.md` | 更新 | `[v25.4.0]` エントリ |
| `site/content/docs/runes/mysql.mdx` | 新規 | API ドキュメント |
| `benchmarks/v25.4.0.json` | 新規 | test_count: 2000 |
