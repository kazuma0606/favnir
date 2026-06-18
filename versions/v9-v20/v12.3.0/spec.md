# Favnir v12.3.0 Spec

Date: 2026-06-07
Theme: `bind` を真の monadic bind に修正（`--legacy` モード）

---

## 背景

### 問題: `bind` ≠ monadic bind

関数型言語の `bind`（モナドの `>>=`）は本来以下の動作をする:

- `Ok(v)` → `v` を変数に束縛して続行
- `Err(e)` → そこで短絡し、`Err(e)` を上位に伝播

しかし現状の Favnir `--legacy` モードでは `bind` が**単純代入**として実装されている。
`bind x <- Postgres.execute_raw(...)` を実行すると、`x` には `Result<Unit, String>` の
値そのもの（`Ok(...)` や `Err(...)`）が格納される。
Err でも短絡せず、後続の処理が続行してしまう。

```
実際の動作（v12.2.0 まで）:
  bind x <- Result.err("接続失敗")
  → x = Err("接続失敗")  // 短絡しない！後続が動き続ける
```

### `chain` との乖離

現状では**`chain`** が本来の monadic bind の動作をしている:
- `chain x <- expr` → Ok(v) なら x に v を束縛して続行、Err(e) なら短絡

`chain` と `bind` の意味論的な役割が逆になっており、
AI も人間も「`bind` でいいのか、`chain` を使うべきか」を判断できない。

E2E デモで「Postgres が全エラー無視で exit 0」になった根本原因の一つがこれ。

### Favnir pipeline モード（デフォルト）は影響なし

デフォルトの Favnir pipeline モード（compiler.fav 経由）では:
- `bind x <- expr` = 単純代入（変更なし）
- `chain x <- expr` = monadic bind（既に正しく動作）

ユーザーは Result を扱う箇所では `chain` を使うことが明確になっており、
このモードの動作は v12.3.0 で変更しない。

---

## v12.3.0 の変更

### 対象: `--legacy` モードの Rust VM

`src/backend/vm.rs` の `IRStmt::Bind` 実行時に、
**束縛される値が `Result` ヴァリアント（`ok` / `err`）の場合**に以下を適用する:

| 値 | 現在の動作 | v12.3.0 以降 |
|---|---|---|
| `Value::Variant("ok", Some(inner))` | `x` = `Ok(inner)` 全体 | `x` = `inner`（アンラップ） |
| `Value::Variant("err", Some(msg))` | `x` = `Err(msg)` 全体（続行） | 即座に短絡 → 上位に `Err(msg)` を伝播 |
| その他（Unit, Int, String, List 等） | 変更なし | 変更なし |

### エラー伝播の仕組み

`IRStmt::Bind` で `Err` を検出した場合、`VMError` を返して VM の実行スタックを
即座に巻き戻す。呼び出し元（stage runner / seq runner）はこの `VMError` をキャッチし、
stage を `Err` 扱いにする。

```
Before (v12.2.0):
  stage LoadAndInsert:
    bind _ <- Postgres.execute_raw(...)  // Err("SSL required")
    // ↑ x = Err("SSL required") だが続行
    bind _ <- Postgres.execute_raw(...)  // 2回目も実行される

After (v12.3.0):
  stage LoadAndInsert:
    bind _ <- Postgres.execute_raw(...)  // Err("SSL required")
    // ↑ 短絡: stage が Err("SSL required") で終了
    // 後続の bind は実行されない
```

### 影響範囲と後方互換性

**後方互換性の破壊あり**（`--legacy` モードのみ）:

既存の `--legacy` コードで `bind x <- result_fn()` の後に `match x { Ok(v) => ... }` と
書いているコードは動作が変わる。v12.2.0 の W006 でこのパターンを事前に警告済みのため、
移行コストは最小限。

**Favnir pipeline モード（デフォルト）は一切影響なし**。

---

## エラーメッセージ

`bind` による短絡時のエラー出力例:

```
[ERROR] stage 'LoadAndInsert' stopped at bind: Err("SSL connection required")
  --> pipeline.fav:10
   |
10 | bind _ <- Postgres.execute_raw("CREATE TABLE ...", "[]")
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: bind propagates Err in --legacy mode (v12.3.0+)
   = help: to handle the error explicitly, use: match Postgres.execute_raw(...) { Ok(_) => ... Err(e) => ... }
   = help: to propagate automatically without binding, use: chain _ <- Postgres.execute_raw(...)
```

---

## `bind` vs `chain` セマンティクス整理（v12.3.0 以降）

| キーワード | pipeline モード | legacy モード |
|---|---|---|
| `bind x <- expr` | 単純代入（expr の戻り値をそのまま束縛） | **Result なら unwrap/短絡、非 Result なら単純代入** |
| `chain x <- expr` | monadic bind（Ok → x=v, Err → 短絡） | monadic bind（変更なし） |

---

## テストケース

### 正常系

| テスト名 | 内容 | 期待結果 |
|---|---|---|
| `bind_ok_unwraps_value_legacy` | `bind x <- Result.ok(42)` → `x == 42`（Intとして使える） | OK |
| `bind_non_result_unchanged_legacy` | `bind x <- IO.println("hi")` → `x == Unit` | OK |
| `bind_chain_same_ok_semantics` | `bind x <- Result.ok(42)` と `chain x <- Result.ok(42)` が同じ結果 | OK |

### 短絡系

| テスト名 | 内容 | 期待結果 |
|---|---|---|
| `bind_propagates_err_legacy` | `bind x <- Result.err("fail")` → stage が Err で停止 | Err |
| `bind_err_skips_subsequent_binds` | Err 後の bind は実行されない | 後続 bind の副作用なし |
| `bind_err_stops_seq_pipeline` | seq pipeline で前段が Err → 後段が実行されない | Err + pipeline 停止 |

### 後方互換確認

| テスト名 | 内容 | 期待結果 |
|---|---|---|
| `favnir_pipeline_bind_unchanged` | デフォルト pipeline モードで `bind x <- Result.ok(42)` → x = Ok(42) のまま | OK（変更なし） |

### バージョン確認

| テスト名 | 内容 |
|---|---|
| `version_is_12_3_0` | `CARGO_PKG_VERSION == "12.3.0"` |

---

## スコープ外（v12.3.0 には含めない）

- `seq` pipeline fail-fast — v12.4.0 で別途実装
- `fav run --verbose` によるトレース出力 — v12.5.0
- Favnir pipeline モードの `bind` 変更 — 設計上変更しない
- `--legacy` モードの非推奨化 — 今後の検討事項
