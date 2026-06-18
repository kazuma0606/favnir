# v18.1.0 — エフェクト推論（Effect Inference）仕様

## 概要

関数ボディからエフェクトを自動推論する。これにより `!Db` / `!IO` / `!AWS` 等のエフェクト宣言を手書きしなくてもよくなる。
宣言漏れによる E0314〜E0319 エラーが発生しなくなり、記述量が減る。

---

## 1. エフェクト推論の動作

### 1.1 現状と推論後の比較

```fav
// 現状: エフェクトを手動で宣言（宣言漏れが E0314 等になる）
fn load_users() -> Result<List<User>, String> !Db !IO {
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- IO.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}

// 推論後: エフェクト宣言が不要（型チェッカーが自動推論）
fn load_users() -> Result<List<User>, String> {
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- IO.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}

// stage も同様に推論
stage LoadUsers -> List<User> {  // !Db !IO が自動推論される
  bind rows <- Postgres.query_raw("SELECT * FROM users", [])
  bind _    <- IO.println(f"loaded {List.length(rows)} rows")
  Result.ok(rows)
}
```

### 1.2 推論の仕組み

```
1. 関数ボディを走査してプリミティブ呼び出しを収集
   Postgres.* → !Db
   IO.*       → !IO
   S3.*       → !AWS
   Kafka.*    → !Kafka
   Snowflake.* → !Snowflake
   BigQuery.*  → !BigQuery
   Http.*      → !Http

2. 呼び出す関数のエフェクトを再帰的に収集（推移的推論）

3. 推論結果 = EffectSet を fn のシグネチャに付与

4. 明示宣言がある場合は整合性を検査:
   - 明示 ⊇ 推論  → OK（明示 > 推論なら W010 警告）
   - 明示 ⊂ 推論  → E0330（宣言漏れ）
```

### 1.3 明示宣言を維持するケース

```fav
// interface の実装では明示が必要（外部からの契約として）
interface Loader {
  fn load() -> Result<List<Row>, String> !Db
}

// 純粋関数であることをコンパイル時に保証したい場合
fn pure_transform(row: Row) -> Row {
  { ...row, score: compute(row) }
}
// もし Db 呼び出しが混入していたらコンパイルエラー
```

---

## 2. 実装内容

### 2.1 `fav/src/middle/checker.rs`

#### 新型 `EffectSet`

```rust
pub type EffectSet = std::collections::HashSet<Effect>;
```

#### 新関数 `infer_effects`

```rust
pub fn infer_effects(fn_def: &FnDef, env: &Env) -> EffectSet
```

**動作:**

1. `fn_def.body` を走査
2. `Expr::FieldAccess { obj: Expr::Var(ns), .. }` を検出
3. ネームスペース → Effect マッピング:
   - `"Postgres"` / `"Db"` → `Effect::Db`
   - `"IO"` → `Effect::IO`
   - `"S3"` → `Effect::AWS`
   - `"Kafka"` → `Effect::Kafka`
   - `"Snowflake"` → `Effect::Snowflake`
   - `"BigQuery"` → `Effect::BigQuery`
   - `"Http"` → `Effect::Http`
4. 呼び出す関数が既知のエフェクトを持つ場合（`fn_effects_registry` から参照）は Union
5. 結果 `EffectSet` を返す

#### `fn_effects_registry: HashMap<String, EffectSet>`

`check_program` / `register_item_signatures` で関数ごとの推論済みエフェクトを蓄積。
推移的推論のために後続関数の参照が必要なため、2 パス構成:
1. Pass 1: プリミティブ呼び出しからの直接エフェクト収集
2. Pass 2: 他の fn 呼び出しを経由した推移的エフェクト伝播

#### 明示宣言との整合性検査

`check_fn_def` にて:
- `fn.effects.is_empty()` → 推論結果をそのまま適用
- `fn.effects` が明示されている場合 → `infer_effects` の結果と比較
  - 推論 ⊄ 明示 → E0330（"effect not declared: !Db"）
  - 推論 ⊊ 明示 → W010（"declared effect !IO not required"）

### 2.2 `self/checker.fav`（Favnir 実装）

`infer_effects_fn` 関数を追加:

```fav
fn infer_effects_fn(fn_def: FnDef, env: CheckEnv) -> List<String> {
  bind stmts <- fn_def.body.stmts
  bind effects <- [ns_to_effect(call.ns) | stmt <- stmts, call <- collect_calls(stmt)]
  List.dedup(effects)
}
```

### 2.3 `fav/src/driver.rs`

#### `--show-effects` オプション

`fav check --show-effects` で推論されたエフェクトを表示:

```
fn load_users   inferred: !Db !IO
fn pure_transform inferred: (none)
stage LoadUsers inferred: !Db !IO
```

#### W010 警告

```
W010: declared effect `!IO` is not required by `fn pure_transform`
```

---

## 3. エフェクト → ネームスペース マッピング表

| ネームスペース | エフェクト |
|---|---|
| `Postgres`, `Db` | `!Db` |
| `IO` | `!IO` |
| `S3`, `Sqs`, `Dynamo`, `Aws` | `!AWS` |
| `Kafka`, `Rskafka` | `!Kafka` |
| `Snowflake` | `!Snowflake` |
| `BigQuery` | `!BigQuery` |
| `Http`, `Ureq` | `!Http` |
| `Llm` | `!Llm` |

---

## 4. エラーコード

| コード | 説明 |
|---|---|
| `E0330` | エフェクト宣言漏れ（推論結果 ⊄ 明示宣言） |
| `W010` | 余分なエフェクト宣言（明示 ⊋ 推論結果） |

> E0330 は v17.8.0 から既存コードとの番号重複を避けるため v18.1.0 では E0336 として追加する。

---

## 5. テスト（v181000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_1_0` | Cargo.toml に "18.1.0" が含まれる |
| `effect_inference_db` | `Postgres.query_raw` を含む fn に `!Db` が推論される |
| `effect_inference_multi` | `Postgres.*` と `IO.*` を含む fn に `!Db !IO` が推論される |
| `effect_inference_pure` | 副作用なし fn のエフェクトが空集合になる |
| `effect_inference_transitive` | `!Db` を持つ fn を呼ぶ fn にも `!Db` が推論される |

---

## 6. 完了条件

- [ ] `!Db` を宣言しなくても `Postgres.*` を呼ぶ fn の `fav check` が通る
- [ ] 複数エフェクト（`!Db !IO`）が正しく推論される
- [ ] 純粋関数（副作用なし）のエフェクトが空集合として推論される
- [ ] エフェクトありの fn を呼ぶ fn でもエフェクトが伝播する（推移的推論）
- [ ] `fav check --show-effects` で推論結果を確認できる
- [ ] `cargo test v181000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
