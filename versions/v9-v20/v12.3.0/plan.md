# Favnir v12.3.0 Plan

Date: 2026-06-07
Theme: `bind` を真の monadic bind に修正（`--legacy` モード）

---

## 実装対象

### `src/backend/vm.rs` — `IRStmt::Bind` の Result ハンドリング

`--legacy` モードの VM（`src/backend/vm.rs`）で `IRStmt::Bind` を実行するとき、
束縛される値が `Value::Variant` の `"ok"` / `"err"` ヴァリアントの場合に特別処理を追加する。

変更前（v12.2.0 まで）:
```rust
// IRStmt::Bind(name, expr) — 現状
let val = self.eval_expr(expr)?;
self.env.insert(name.clone(), val);
```

変更後（v12.3.0）:
```rust
// IRStmt::Bind(name, expr) — 変更後
let val = self.eval_expr(expr)?;
match &val {
    Value::Variant(tag, Some(inner)) if tag == "ok" => {
        self.env.insert(name.clone(), *inner.clone());
    }
    Value::Variant(tag, Some(msg)) if tag == "err" => {
        return Err(VMError::BindErr {
            name: name.clone(),
            msg: msg.to_string(),
        });
    }
    _ => {
        self.env.insert(name.clone(), val);
    }
}
```

`VMError::BindErr` は `stage runner` / `seq runner` でキャッチし、
stage を `Err` 扱いにしてエラーメッセージを出力する。

---

## 実装フロー

```
Phase A: vm.rs の IRStmt::Bind / IRStmt::Chain 付近のコード確認
    ↓
Phase B: VMError に BindErr ヴァリアント追加
    ↓
Phase C: IRStmt::Bind の Result アンラップ / 短絡実装
    ↓
Phase D: エラーメッセージ出力（stage runner / seq runner）
    ↓
Phase E: Favnir pipeline モード（IRStmt::Bind は変更なし）の確認
    ↓
Phase F: driver.rs に v12300_tests モジュール追加
    ↓
Phase G: cargo test v12300 / cargo test 全通過確認
    ↓
Phase H: Cargo.toml バージョン更新 + コミット
```

---

## 技術的注意点

### `Value::Variant` の構造

`vm.rs` の `Value` 型で `Result` は `Value::Variant(tag, inner)` として表現される。
- `Ok(v)` → `Value::Variant("ok", Some(Box<Value>))`
- `Err(e)` → `Value::Variant("err", Some(Box<Value>))`（e は `Value::Str` 等）

tag の文字列は小文字（`"ok"` / `"err"`）であることを確認してから実装する。

### `IRStmt::Chain` との関係

`IRStmt::Chain` は既に monadic bind（ChainCheck あり）として実装されている。
`IRStmt::Bind` への変更は `--legacy` モードのみが対象。

Favnir pipeline モード（`compiler.fav` 経由）では `bind` → `IRStmt::Bind`、
`chain` → `IRStmt::Chain` と lowered されるが、pipeline モードでは:
- `IRStmt::Bind` = 単純代入のまま（変更なし）
- `IRStmt::Chain` = monadic bind（変更なし）

`--legacy` フラグの有無で `IRStmt::Bind` の挙動を分岐させる。

### `VMError::BindErr` の設計

```rust
pub enum VMError {
    // 既存エラー種別 ...
    BindErr { name: String, msg: String },
}
```

`BindErr` を受け取った stage runner は以下のメッセージを出力:
```
[ERROR] stage 'StageName' stopped at bind: Err("msg")
  --> pipeline.fav:LINE
   |
   = note: bind propagates Err in --legacy mode (v12.3.0+)
   = help: to handle explicitly: match expr { Ok(_) => ... Err(e) => ... }
   = help: to propagate without binding: chain _ <- expr
```

ファイル・行番号の出力は既存エラーフォーマットに合わせる。
行番号情報が取得できない場合はメッセージのみで可。

### Favnir pipeline モードへの影響なし確認

Favnir pipeline モードの `bind x <- expr` は単純代入のまま。
テスト `favnir_pipeline_bind_unchanged` で `bind x <- Result.ok(42)` → `x = Ok(42)` を確認する。

---

## テスト方針

`fav/src/driver.rs` に `v12300_tests` モジュールを追加。

### 正常系（3件）

- `bind_ok_unwraps_value_legacy`: `bind x <- Result.ok(42)` → `x == 42`（Int として使える）
- `bind_non_result_unchanged_legacy`: `bind x <- IO.println("hi")` → `x == Unit`
- `bind_chain_same_ok_semantics`: `bind` と `chain` の Ok 結果が同じ

### 短絡系（3件）

- `bind_propagates_err_legacy`: `bind x <- Result.err("fail")` → stage が Err で停止
- `bind_err_skips_subsequent_binds`: Err 後の bind は実行されない（副作用なし）
- `bind_err_stops_seq_pipeline`: seq pipeline で前段 Err → 後段未実行

### 後方互換確認（1件）

- `favnir_pipeline_bind_unchanged`: デフォルト pipeline モードで `bind x <- Result.ok(42)` → `x = Ok(42)` のまま

### バージョン確認（1件）

- `version_is_12_3_0`

合計 8 件。

---

## 実装優先順位

1. `vm.rs` の `Value::Variant` tag 文字列の確認
2. `VMError::BindErr` 追加
3. `IRStmt::Bind` の `--legacy` モード分岐実装
4. エラーメッセージ出力の整備
5. テスト追加（短絡系から先に実装して動作確認）
6. Favnir pipeline モード不変確認
7. バージョン更新 + コミット
