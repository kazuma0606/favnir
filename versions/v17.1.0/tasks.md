# v17.1.0 Tasks — 境界付きジェネリクス（Bounded Generics）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"17.1.0"` に変更
- [ ] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — AST 拡張（GenericParam）

- [ ] B-1: `fav/src/ast.rs` に `GenericParam { name: String, bounds: Vec<String> }` 構造体追加
- [ ] B-2: `FnDef.type_params: Vec<String>` → `Vec<GenericParam>` に変更
- [ ] B-3: `StageDef.type_params: Vec<String>` → `Vec<GenericParam>` に変更
- [ ] B-4: その他 `type_params: Vec<String>` を持つ定義を同様に変更
- [ ] B-5: `cargo build` でコンパイルエラー一覧を確認（Phase F で修正）

---

## Phase C — パーサー拡張（`with` キーワード）

- [ ] C-1: `fav/src/frontend/parser.rs` の `parse_generic_params` を拡張
  - `<T>` → `GenericParam { name: "T", bounds: [] }` （既存動作）
  - `<T with Ord>` → `GenericParam { name: "T", bounds: ["Ord"] }`
  - `<T with Ord with Serialize>` → `GenericParam { name: "T", bounds: ["Ord", "Serialize"] }`
  - `<A, B with Eq>` → 複数パラメータの正しい解析
- [ ] C-2: `with` をソフトキーワード（Ident）として処理（新 TokenKind 不要）
- [ ] C-3: パース結果が正しい `GenericParam` になることをスモークテストで確認

---

## Phase D — 型チェッカー拡張（bound 検査）

- [ ] D-1: `fav/src/middle/checker.rs` に `type_implements_bound(ty, bound)` 関数追加
  - `"Ord"` → Int / Float / String のみ true
  - `"Eq"` → 全型 true
  - `"Serialize"` → 全型 true（簡略）
  - `"Display"` → String / Int / Float / Bool のみ true
  - `"Hash"` → Int / String のみ true
  - `"Clone"` → 全型 true
  - その他 → カスタム interface の既存機構で確認
- [ ] D-2: `check_bounded_call` 関数追加（ジェネリクス関数呼び出し時に bound 検査）
- [ ] D-3: E0325 エラー追加（`Type does not implement Interface`）
- [ ] D-4: `check_fn_def` で `GenericParam.bounds` を型環境に登録

---

## Phase E — コンパイラ対応

- [ ] E-1: `fav/src/middle/compiler.rs` の `GenericParam` 参照箇所を更新
  - `compile_fn_def` / `compile_stage_def` の型パラメータ処理
  - bounds は型消去（コンパイル時に除去）
- [ ] E-2: `self/checker.fav` に `check_bounded_generics` 関数追加（Favnir 側の bound チェック）

---

## Phase F — exhaustive match 対応

- [ ] F-1: `fav/src/middle/compiler.rs` の `type_params` 参照を `GenericParam` 対応に更新
- [ ] F-2: `fav/src/middle/checker.rs` の `type_params` 参照を `GenericParam` 対応に更新
- [ ] F-3: `fav/src/driver.rs` の `type_params` 参照を `GenericParam` 対応に更新
- [ ] F-4: その他 exhaustive match エラーを全件解消
- [ ] F-5: `cargo build` → コンパイルエラーなし確認

---

## Phase G — テスト追加（v171000_tests）

- [ ] G-1: `fav/src/driver.rs` に `v171000_tests` モジュール追加
- [ ] G-2: `version_is_17_1_0` — `Cargo.toml` に `"17.1.0"` が含まれる
- [ ] G-3: `bounded_generic_ord` — `fn max<T with Ord>(a: T, b: T) -> T` が Int / Float / String で動作
- [ ] G-4: `bounded_generic_serialize` — `fn serialize<T with Serialize>(val: T) -> String` が動作
- [ ] G-5: `bounded_generic_violation` — `Ord` を実装しないレコード型で E0325 が出る
- [ ] G-6: `bounded_generic_multi` — `T with Ord with Serialize` 複数 bound が動作
- [ ] G-7: `cargo test v171000` → 5/5 PASS 確認

---

## Phase H — サイトドキュメント作成

- [ ] H-1: `site/content/docs/language/generics.mdx` を新規作成
  - 境界付きジェネリクスの構文説明
  - 組み込み Interface 一覧表（Ord / Eq / Serialize / Display / Hash / Clone）
  - カスタム `interface` との組み合わせ例
  - Before / After 比較（制約なし vs 制約あり）
  - E0325 エラーの説明と修正方法

---

## Phase I — 最終確認 + コミット

- [ ] I-1: `cargo test v171000` → 5/5 PASS 最終確認
- [ ] I-2: `cargo test` → 全件 PASS（リグレッションなし）
  - 旧版 version_is_xxx テストは `#[ignore]` 済みのため除外
- [ ] I-3: コミット: `feat: v17.1.0 — 境界付きジェネリクス（Bounded Generics）`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "17.1.0"` | [ ] |
| `fn max<T with Ord>(a: T, b: T) -> T` が Int / Float / String で動作する | [ ] |
| `fn f<T with Serialize>(v: T)` が全レコード型で動作する | [ ] |
| `T with Ord with Serialize` の複数制約が動作する | [ ] |
| bound を満たさない型を渡すと E0325 でエラーになる | [ ] |
| カスタム `interface` との組み合わせが動作する | [ ] |
| `cargo test v171000` → 5/5 PASS | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 技術メモ

- **`with` はソフトキーワード**: `Ident("with")` として解析。新 `TokenKind` は不要。`parse_generic_params` 内でのみ特別扱いする。
- **型消去方式**: bound は checker が検証済み → compiler は bound を無視。実行時ディスパッチなし（静的解決済み）。
- **後方互換**: 既存の `<T>` は `GenericParam { name: "T", bounds: [] }` として表現。bounds 空 = 従来と同じ動作。
- **`FnDef.type_params` 変更の影響範囲**: `compiler.rs`・`checker.rs`・`driver.rs`・`emit_python.rs`・`fmt.rs` など多数の箇所が `type_params` を参照している。Phase F で一括対応。
- **`include_str!` パス**: driver.rs → Cargo.toml は `"../Cargo.toml"`（`fav/src/` からの相対パス）。
- **E0325 の発行タイミング**: `check_bounded_call` での型引数解決時。型推論が完了してから bound チェックを走らせる。
