# v18.6.0 — 共変・反変アノテーション（Variance）タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/ast.rs` — 型追加・構造体拡張

- [x] `Variance` enum を `GenericParam` の直前に追加（`Covariant / Contravariant / Invariant`）
- [x] `GenericParam` に `pub variance: Variance` フィールドを追加
- [x] `GenericParam::unbounded()` の初期化に `variance: Variance::Invariant` を追加
- [x] `InterfaceDecl` に `pub type_params: Vec<GenericParam>` フィールドを追加

### T2: 波及ファイル修正

- [x] `fav/src/frontend/parser.rs` — `InterfaceDecl { ..., type_params: vec![], ... }` 追加
- [x] `fav/src/frontend/parser.rs` — `GenericParam { name, bounds, variance: Variance::Invariant }` 修正（2箇所）
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

### T3: `fav/src/frontend/parser.rs` — `+T` / `-T` パース

- [x] `parse_variance_type_params` 新メソッドを追加
  - `LAngle` (`<`) → 通常開始
  - `LArrow` (`<-`) → 最初のパラメータが Contravariant（`<-T` は `<-` として一体レキシング）
  - `Plus` → Covariant、`Minus` → Contravariant（2番目以降）
- [x] `parse_interface_decl` で `parse_variance_type_params()` を呼び出し
- [x] パースした型パラメータを `InterfaceDecl.type_params` にセット

**重要な技術知見**: `<-T` は lexer によって `LArrow`（`<-`）+ `Ident("T")` + `RAngle` としてトークン化される。
`parse_variance_type_params` で `LArrow` を先頭トークンとして認識し、
`first_is_contravariant = true` フラグで最初のパラメータの分散を設定する。

### T4: `fav/src/middle/checker.rs` — 分散チェック実装

- [x] `check_interface_variance` 関数を追加（`check_interface_decl` から呼び出し）
- [x] `type_expr_contains_in_input` フリー関数（`Arrow/LinearArrow` の左辺に名前が現れるか）
- [x] `type_expr_contains_in_output` フリー関数（`Arrow/LinearArrow` の右辺 + `Named` に名前が現れるか）
- [x] `type_expr_contains` フリー関数（型式中のどこかに名前が現れるか）
- [x] E0334 のエラーメッセージ + ヒント

### T5: `fav/src/driver.rs` — `v186000_tests` 追加

- [x] `v185000_tests::version_is_18_5_0` に `#[ignore]` を追加
- [x] `v186000_tests` モジュールを追加（5件）:
  - [x] `version_is_18_6_0`
  - [x] `variance_covariant_parses`
  - [x] `variance_contravariant_parses`
  - [x] `variance_subtype_covariant`
  - [x] `variance_violation_error`

### T6: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.5.0` → `18.6.0` に更新

### T7: `site/content/docs/language/variance.mdx` 作成

- [x] 共変・反変・不変の説明と分散ルール表
- [x] `+T` / `-T` 構文の例（Source/Sink パターン）
- [x] E0334 エラーの説明と例（共変違反・反変違反）
- [x] 複数型パラメータの例 `Mapper<-T, +U>`

---

## テスト結果

| テスト名 | 結果 |
|---|---|
| `version_is_18_6_0` | PASS |
| `variance_covariant_parses` | PASS |
| `variance_contravariant_parses` | PASS |
| `variance_subtype_covariant` | PASS |
| `variance_violation_error` | PASS |

**5/5 PASS。全体 1683 tests pass（リグレッションなし）。**

---

## 実装ノート

- **`<-T` のレキシング**: `<-` は `LArrow` 単一トークンになる。`parse_variance_type_params` の先頭で `LArrow` を検出して `first_is_contravariant = true` を設定することで対処。2番目以降のパラメータでは `, -U` の `-` は `Minus` としてレキシングされるので問題なし。
- **`TypeExpr` のバリアント**: `App / List / Option / Result / Tuple / Record` は存在しない。実際のバリアントは `Named / Optional / Fallible / Arrow / TrfFn / Intersection / RecordType / Schema / LinearArrow`。
- **`check_interface_variance` の呼び出し位置**: `check_interface_decl` の `interface_registry.register_interface` より前に実行（エラーが出ても登録は続行）。
- **サブタイピングの完全実装は v19.x**: v18.6.0 では E0334 検出（宣言の整合性チェック）のみ。実際のサブタイプ代入チェックは将来版。
