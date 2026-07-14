# v43.7.0 仕様書 — 構造体リテラル推論（Structural inference）

## 概要

ロードマップ: "リスト・タプル・レコードリテラルの型を呼び出しコンテキストから決定"

### v43.7.0 スコープ

v43.7.0 は**名前付きレコードリテラル**（`TypeName { field: val ... }`）のみを対象とする。
ロードマップ記載の匿名レコードリテラル（`{ name: "Alice", age: 30 }`、`tname = ""`）および
リスト・タプルリテラルの文脈推論は **v43.8.0 双方向型推論（Bidirectional / top-down）のスコープ**。

```favnir
// 名前付きレコードリテラルを関数に渡す — 型が一致する
type Point = { x: Int  y: Int }
fn make() -> Point { Point { x: 1  y: 2 } }
fn shift(p: Point) -> Point { Point { x: p.x  y: p.y } }
```

---

## 現状と問題

`ERecordLit` の `infer_expr` 処理（checker.fav line 1957）:

```favnir
ERecordLit({ _0: tname, _1: fields }) => Result.ok(tname)
```

これにより:
- **名前付きレコードリテラル**（`Point { x: 1  y: 2 }`）は `tname = "Point"` を返す → 型情報が正確 ✓
- **フィールド型検証**: 現時点では `fields` を評価せず `tname` を返すのみ（フィールド型チェックは将来課題）

`check_fn_def` → `infer_hm` → `ECall` → `infer_call_user` の経路で、レコードリテラルを渡す呼び出しは `infer_arg_tys` が `tname` を評価し、宣言型と `types_compatible` で比較される。

---

## 事前確認（T0）

v43.7.0 実装前に以下を手動確認する:
- `cargo test` → 2920 passed; 0 failed
- `Cargo.toml` version = `43.6.0`
- `driver.rs` に `v43700_tests` モジュールが存在しないこと
- `checker.fav` line 1957 の `ERecordLit({ _0: tname, _1: fields }) => Result.ok(tname)` が存在すること

これにより既存の `ERecordLit` 機構でテストが通ることを事前に保証する。

---

## 解決

v43.7.0 は **実装追加なし** のバリデーションリリース。

名前付きレコードリテラルを使った構造体リテラル推論は既存の仕組みで機能する:
- `ERecordLit` が `tname` を返す
- `types_compatible(inferred, declared)` が `inferred == declared` を検証

```
check_fn_def
  └─ infer_hm(ECall("make_point", args), env)
       └─ infer_call_user → infer_arg_tys → infer_expr(ERecordLit("Point", ...)) → "Point"
            └─ types_compatible("Point", "Point") → true ✓
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v43700_tests` 追加（2 件） |
| `fav/Cargo.toml` | version 43.6.0 → 43.7.0 |
| `CHANGELOG.md` | v43.7.0 エントリ追加 |
| `versions/current.md` | v43.7.0 最新安定版に更新 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.7.0 を COMPLETE に更新 |

**`fav/self/checker.fav` は変更不要**: 既存の `ERecordLit` 機構で名前付きレコードリテラルが正しく型チェックを通過する。

---

## フィールド区切り記法

Favnir レコードリテラルのフィールド区切りは**スペース**（コンマ不要）。
`Point { x: 1  y: 2 }` は二重スペースで区切られており、これが正式な構文。
コンマ区切り（`Point { x: 1, y: 2 }`）との混在は避ける。

---

## テスト設計

### v43.6.0 との差分

v43.6.0 は「多段 `bind` チェーンでの `List` 型伝播」を検証した。
v43.7.0 は「**レコードリテラル** の型情報が関数呼び出しコンテキストで正しく伝播・検証される」ことを検証する。
具体的には `ERecordLit` → `infer_arg_tys` → `types_compatible` の経路を通るテストケース。

### T1 — `v43700_tests`

#### `cargo_toml_version_is_43_7_0`
バージョン確認テスト（次バージョン bump 時にスタブ化）。

#### `structural_record_literal_type_checks`

```rust
let src = r#"
type Point = { x: Int  y: Int }
fn make_point() -> Point { Point { x: 1  y: 2 } }
fn shift(p: Point) -> Point { Point { x: p.x  y: p.y } }
"#;
```

- `Point { x: 1  y: 2 }` の `ERecordLit` が `tname = "Point"` を返し、宣言戻り型 `Point` と一致する
- `fn shift(p: Point) -> Point` に `Point { ... }` を渡す構造も型エラーなし
- `run_checker_fav` → `Ok(())` を返す（`Err` 時はその内容を `{:?}` で表示）

---

## 完了条件

- `cargo test` 2922 tests passed, 0 failed（2920 + 2）
- `v43700_tests` 2 件 pass
- `structural_record_literal_type_checks`: `run_checker_fav` が `Ok(())` を返す

---

## 影響範囲

- **checker.fav 変更なし**: 既存の `ERecordLit` → `tname` 返却で動作する
- **既知制限（スコープ外）**:
  - **匿名レコードリテラル**（`{ name: "Alice", age: 30 }` — 型名なし、`tname = ""`）の文脈推論は非対応。これが v43.8.0 双方向型推論（Bidirectional / top-down）のスコープ
  - **フィールド型検証**: `ERecordLit` の `fields` はフィールド名/型の検証なしに無視される（将来課題）
  - ロードマップ例の `process({ name: "Alice", age: 30 })` は匿名レコードであるため、v43.7.0 では非対応（v43.8.0 以降）

---

## 前提条件

- v43.6.0 COMPLETE（2920 tests）
- `ERecordLit({ _0: tname, _1: fields }) -> Result.ok(tname)` （checker.fav line 1957）
- `types_compatible(inferred, declared)`: `inferred == declared` または `inferred == "Unknown"` で互換判定
- `infer_arg_tys(args, env)`: `EArgList` を再帰的に評価して型リストを構築
