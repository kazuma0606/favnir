# v37.2.0 実装計画 — 行多相実用強化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 変更 | `v37100_tests` スタブ化 / `v37200_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.1.0"` → `"37.2.0"` |
| `CHANGELOG.md` | 追記 | `[v37.2.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.2.0 の完了条件をスコープ縮小後の内容に更新 + ✅ マーク |
| `versions/current.md` | 更新 | 最新安定版 v37.2.0、次バージョン v37.3.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.2.0] エントリ追加

`## [v37.1.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

```markdown
## [v37.2.0] - 2026-07-09

### Added
- 複数フィールド行制約 `R with { id: Int, name: String }` が call-site 型チェックを通ることをテストで保証
- ネスト行型 `R with { address: { city: String } }` がパースを通ることを確認
- `v37200_tests` 4 テスト追加

---
```

### Step 2: 事前確認 — 複数フィールド行制約とネスト型の動作確認

```bash
# パーサーが record type constraint をどう処理するか確認
grep -n "parse_type_expr\|RecordType\|LBrace\|HasField" fav/src/frontend/parser.rs | head -20

# 複数フィールド制約の call-site check が通るか事前確認
# (v37200_tests 追加後 cargo test v37200 で確認)
```

`R with { id: Int, name: String }` のパースにより複数の `TypeConstraint::HasField` が生成されるか確認。
もし未対応の場合は parser.rs の `parse_type_bounds` を調整する。

### Step 3: driver.rs — `v37100_tests::cargo_toml_version_is_37_1_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.2.0` に変更。

### Step 4: driver.rs — `v37200_tests` モジュール追加

`v37100_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する。

追加内容は spec.md の `v37200_tests` コードブロックに従う。

**重要:** `row_poly_multi_field_checks` は `type UserRow = { ... }` の定義と
`display(UserRow { ... })` の call-site を含めること。
関数宣言のみでは `HasField` の call-site check がトリガーされないため。

構成:
- imports: `use crate::frontend::parser::Parser;` / `use crate::middle::checker::Checker;`
- ローカル `check_errors()` ヘルパー
- `use super::*` 不要（`run()` は使わない）

```bash
# テスト追加後に個別確認
cargo test v37200 2>&1 | grep "v37200"
```

### Step 5: Cargo.toml バージョン更新

Step 1〜4 完了・コンパイルエラー解消後に `37.1.0` → `37.2.0` に更新。

## 依存関係

- `row_poly_multi_field_checks` は call-site を含むため、`TypeConstraint::HasField` の call-site check が必要 → Step 2 で動作確認
- `nested_row_type_parseable` は `Parser::parse_str(...).is_ok()` のみ → チェッカー依存なし
- `v37200_tests` は `use super::*` 不要（`include_str!` と `Parser` / `Checker` のみ使用）

## リスク

| リスク | 対処 |
|---|---|
| `R with { id: Int, name: String }` の複数フィールド制約が未対応の場合 | Step 2 で確認後、parser.rs の `parse_type_bounds` に record type constraint 対応を追加 |
| ネスト record type `{ city: String }` がパーサーで型として未対応の場合 | `nested_row_type_parseable` を `is_ok()` での確認にとどめており、型チェックを要求しない |
| `v37100_tests` の閉じ `}` 行番号が T0 の記録と異なる | T0 で Read して実際の行番号を確認してから Edit |
