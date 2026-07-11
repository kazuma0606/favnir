# v36.4.0 実装計画 — `fav validate` コマンド

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.4.0]` エントリを追加（`## [v36.3.0]` の直後） |
| S2 | `fav/src/driver.rs` | `validate_schema_against_headers` / `read_csv_headers` / `cmd_validate` を追加 |
| S3 | `fav/src/main.rs` | `cmd_validate` を `use driver::{ ... }` に追加、`Some("validate") =>` アームを追加 |
| S4 | `fav/src/driver.rs` | `v36300_tests::cargo_toml_version_is_36_3_0` をスタブ化 |
| S5 | `fav/src/driver.rs` | `v36400_tests` モジュール（5 件）を追加 |
| S6 | `fav/Cargo.toml` | バージョンを `36.3.0` → `36.4.0` に更新（必ず **S2〜S5 すべて完了後**） |
| S7 | `cargo test` | 全通過確認（≥ 2676 件） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v36.3.0]` の `---` セパレータの直後に挿入:

```markdown
## [v36.4.0] — 2026-07-08

### Added
- `fav validate --schema <schema.fav> <data.csv>` コマンド — CSV ヘッダーとスキーマフィールドの整合性を検証
- `cmd_validate` — driver.rs に追加
- `validate_schema_against_headers` — スキーマ照合の純粋関数（テスト可能）

---
```

### S2: driver.rs — `cmd_validate` 実装

driver.rs の末尾（`// ── fav lint ──` セクションの直後）に追加する。
実装の詳細は spec.md §1 を参照。

**挿入位置の目安**: `cmd_lint` 関数の定義末尾の `}` の後に `// ── fav validate ──` セクションを追加する。

**注意**:
- `validate_schema_against_headers` は `pub fn` にする（テストモジュールからの呼び出しに必要）
- `read_csv_headers` は `fn`（プライベート）でよい
- `cmd_validate` は `pub fn`

### S3: main.rs — ルーティング追加

#### import 更新

main.rs の `use driver::{ ... }` ブロックに `cmd_validate` を追加する。
`cmd_lint` と同じ行近辺に追加するのが読みやすい:

```rust
cmd_lint, cmd_validate,
```

#### ルーティング

`Some("lint") =>` アームを見つけ、その閉じ `}` の後に `Some("validate") =>` アームを追加する。
spec.md §2 のコードを参照。

### S4: driver.rs — スタブ化

`v36300_tests::cargo_toml_version_is_36_3_0` のアサーションを空実装に置き換え:

```rust
#[test]
fn cargo_toml_version_is_36_3_0() {
    // stubbed: version bumped to 36.4.0
}
```

### S5: driver.rs — v36400_tests モジュール追加

`v36300_tests` の閉じ `}` の後に追加。spec.md §3 のコードを参照。
5 件構成（ロードマップ最小要件 2 件を上回る — spec.md §ロードマップ整合を参照）。

### S6: Cargo.toml バージョン更新

**必ず S2〜S5 すべて完了後に実行すること**。

`version = "36.3.0"` → `version = "36.4.0"`

### S7: cargo test

期待値: T0 で実測した件数 + 5（v36400_tests）= 目標件数 pass、0 failures
（T0 実測値が 2671 の場合: 2671 + 5 = **2676 件**）

## 実装上の重要チェックポイント

### `cmd_lint` アームの場所を探す方法

```bash
grep -n "Some(\"lint\")" fav/src/main.rs | head -5
```

### `validate_schema_against_headers` の公開確認

`pub fn validate_schema_against_headers` として定義されているか確認:

```bash
grep -n "validate_schema_against_headers" fav/src/driver.rs | head -5
```

### `v36300_tests` の閉じ `}` の行番号を確認

```bash
grep -n "v36300_tests\|v36400_tests" fav/src/driver.rs | head -10
```

## `fav validate` の動作フロー

```
fav validate --schema orders.fav data.csv
    ↓
[1] orders.fav をパース → SchemaDef { name: "Orders", fields: [("id", Int), ("amount", Float)] }
    ↓
[2] data.csv のヘッダー行を読み取る → ["id", "amount", "status"]
    ↓
[3] 照合: "id" ✓, "amount" ✓  → エラーなし
    ↓
[4] 出力: "data.csv: schema `Orders`: ok"  →  exit 0

--- エラーケース ---
[2] data.csv のヘッダー行 → ["id", "status"]  (amount が欠損)
[3] 照合: "id" ✓, "amount" ✗  → errors = ["missing column: `amount`"]
[4] 出力: "data.csv: schema `Orders`: missing column: `amount`"  →  exit 1
```
