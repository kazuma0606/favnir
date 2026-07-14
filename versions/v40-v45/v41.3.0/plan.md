# v41.3.0 実装計画

## 実装順序

1. **parser.rs** — 式・パターン両方の LParen 分岐を修正（タプルデシュガー）
2. **checker.fav** — デシュガー設計を示すコメント追加
3. **Cargo.toml** — version bump `41.2.0` → `41.3.0`
4. **CHANGELOG.md** — `[v41.3.0]` エントリ追加
5. **driver.rs** — `v41200_tests::cargo_toml_version_is_41_2_0` スタブ化 + `v41300_tests` 追加
6. **cargo test** — 2856 tests passed 確認
7. **versions/current.md** 更新 + roadmap マーク

## 各ステップ詳細

### Step 1: parser.rs

**式側** (`parse_primary` または `parse_atom` の `LParen` 分岐):
- `(` を consume
- `)` が次 → unit `()`
- そうでなければ `parse_expr()` で最初の要素をパース
- `,` が次 → タプルモード: `("_0"〜"_N", expr)` のフィールドリストを構築し `RecordConstruct("__tuple__", fields)` を返す
- `,` でなければ `)` を expect してグルーピング括弧として返す

**パターン側** (`parse_pattern` の `LParen` 分岐):
- `(` を consume
- `)` が次 → unit `()`
- そうでなければ `parse_pattern()` で最初のパターンをパース
- `,` が次 → タプルモード: `PatternField::Alias("_0"〜"_N", pat)` のフィールドリストを構築し `Pattern::Record(fields)` を返す
- `,` でなければ `)` を expect してグルーピング括弧として返す

### Step 2: checker.fav

**ファイル末尾**に spec §3 の文面どおりコメントを追加（`check_item` 付近ではなく末尾）。

### Step 3-4: バージョン管理ファイル

標準手順。

### Step 5: driver.rs

- `v41200_tests::cargo_toml_version_is_41_2_0` スタブ化
- `v41300_tests` 3 件追加（`use super::*` 不要）

## パス確認

| テスト | `include_str!` | 解決先 |
|---|---|---|
| T1 | `include_str!("../Cargo.toml")` | `fav/Cargo.toml` |
| T2 | `include_str!("../../CHANGELOG.md")` | `CHANGELOG.md`（ルート） |
| T3 | `include_str!` 不使用 | `Parser::parse_str` 直接呼び出し |

## 注意事項

- `parse_primary` と `parse_atom` のどちらに LParen 処理があるかを T0 で確認すること
- 式側の LParen 分岐は `TokenKind::LParen` 行番号を T0 で記録してから編集
- パターン側の既存コード（`unit: ()` → `RParen` を expect）を正確に把握してから編集
- `RecordConstruct("__tuple__", ...)` は型チェッカーで E0102 を出す可能性があるが、
  v41.3.0 の検証は `Parser::parse_str` のみのため問題なし
